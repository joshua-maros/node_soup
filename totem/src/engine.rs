use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use bytemuck::Zeroable;
use cranelift::{
    codegen::{ir::Function, Context},
    prelude::*,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, DataId, FuncId, Linkage, Module};
use itertools::Itertools;
use maplit::{hashmap, hashset};
use target_lexicon::Triple;

use crate::util::{self, Id, IdCreator};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FunctionKind {
    /// Uses fastcall convention, has 1 parameter for output plus a number of
    /// parameters equal to the number of parameters the original node has.
    InternalImplementation(NodeId),
    /// Uses systemv convention, has 1 parameter pointing to an output plus
    /// packed parameters after appropriate offsets.
    ExternalWrapper(NodeId),
}

struct CodeGenerationContext {
    builder_c: FunctionBuilderContext,
    codegen_c: Context,
    data_c: DataContext,
    module: JITModule,
    functions: HashMap<FunctionKind, FuncId>,
    constants: HashMap<NodeId, DataId>,
    undefined_functions: HashSet<FunctionKind>,
    previously_defined_functions: HashSet<FunctionKind>,
}

struct NodeDefinitionContext<'x, 'f> {
    func_builder: &'x mut FunctionBuilder<'f>,
    constants: &'x mut HashMap<NodeId, DataId>,
    data_c: &'x mut DataContext,
    module: &'x mut JITModule,
    param_ptrs: &'x HashMap<NodeId, Value>,
    nodes: &'x HashMap<NodeId, Node>,
    functions: &'x mut HashMap<FunctionKind, FuncId>,
    undefined_functions: &'x mut HashSet<FunctionKind>,
    node: NodeId,
}

impl<'x, 'f> NodeDefinitionContext<'x, 'f> {
    pub fn reborrow<'y>(&'y mut self, new_node: NodeId) -> NodeDefinitionContext<'y, 'f>
    where
        'x: 'y,
    {
        NodeDefinitionContext {
            func_builder: &mut *self.func_builder,
            constants: &mut *self.constants,
            data_c: &mut *self.data_c,
            module: &mut *self.module,
            param_ptrs: &*self.param_ptrs,
            nodes: &*self.nodes,
            functions: &mut *self.functions,
            undefined_functions: &mut *self.undefined_functions,
            node: new_node,
        }
    }
}

impl CodeGenerationContext {
    fn make_builder() -> JITBuilder {
        let libcall_names = cranelift_module::default_libcall_names();
        let mut flag_builder = settings::builder();
        // On at least AArch64, "colocated" calls use shorter-range relocations,
        // which might not reach all definitions; we can't handle that here, so
        // we require long-range relocation types.
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "true").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();
        let isa_builder = isa::lookup(Triple::host()).unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        JITBuilder::with_isa(isa, libcall_names)
    }

    fn new() -> Self {
        let mut builder = Self::make_builder();
        builder.hotswap(true);
        let module = JITModule::new(builder);
        Self {
            builder_c: FunctionBuilderContext::new(),
            codegen_c: Context::new(),
            data_c: DataContext::new(),
            module,
            functions: HashMap::new(),
            constants: HashMap::new(),
            undefined_functions: HashSet::new(),
            previously_defined_functions: HashSet::new(),
        }
    }

    fn get_function_signature(
        module: &JITModule,
        nodes: &HashMap<NodeId, Node>,
        function: FunctionKind,
    ) -> Signature {
        let ptr_type = module.target_config().pointer_type();
        if let FunctionKind::InternalImplementation(node) = function {
            let params = nodes[&node].collect_parameter_nodes(node, nodes);
            Signature {
                params: vec![AbiParam::new(ptr_type); 1 + params.len()],
                returns: vec![],
                call_conv: isa::CallConv::Fast,
            }
        } else {
            Signature {
                params: vec![AbiParam::new(ptr_type)],
                returns: vec![],
                call_conv: isa::CallConv::SystemV,
            }
        }
    }

    fn get_function_declaration(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        function: FunctionKind,
    ) -> FuncId {
        Self::get_function_declaration_impl(
            &mut self.functions,
            &mut self.undefined_functions,
            &mut self.module,
            nodes,
            function,
        )
    }

    fn get_function_declaration_impl(
        functions: &mut HashMap<FunctionKind, FuncId>,
        undefined_functions: &mut HashSet<FunctionKind>,
        module: &mut JITModule,
        nodes: &HashMap<NodeId, Node>,
        function: FunctionKind,
    ) -> FuncId {
        *functions.entry(function).or_insert_with(|| {
            let sig = Self::get_function_signature(&*module, nodes, function);
            let id = module
                .declare_function(&format!("{:#?}", function), Linkage::Export, &sig)
                .unwrap();
            undefined_functions.insert(function);
            id
        })
    }

    /// Additionally defines the constant if it has not been defined.
    fn get_constant_declaration(
        constants: &mut HashMap<NodeId, DataId>,
        data_c: &mut DataContext,
        module: &mut JITModule,
        node: NodeId,
        data: &Value2,
    ) -> DataId {
        *constants.entry(node).or_insert_with(|| {
            data_c.define(data.as_ne_bytes().into_boxed_slice());
            let id = module
                .declare_data(
                    &format!("Data For {:?}", node),
                    Linkage::Export,
                    true,
                    false,
                )
                .unwrap();
            module.define_data(id, data_c).unwrap();
            data_c.clear();
            id
        })
    }

    fn write_constant_data(&mut self, node: NodeId, data: &Value2) {
        let buffer_id = self.constants[&node];
        let buffer = self.module.get_finalized_data(buffer_id);
        let slice = unsafe { std::slice::from_raw_parts_mut(buffer.0.cast_mut(), buffer.1) };
        slice.copy_from_slice(&data.as_ne_bytes());
    }

    /// Also defines all nodes this node is dependant on.
    fn define_function_implementation(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        function: FunctionKind,
    ) {
        self.define_function_implementation_impl(nodes, function);
        while self.undefined_functions.len() > 0 {
            for undefined_function in self.undefined_functions.clone() {
                self.define_function_implementation_impl(nodes, undefined_function);
            }
        }
        self.module.finalize_definitions().unwrap();
    }

    fn define_function_implementation_impl(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        function: FunctionKind,
    ) {
        let func_id = self.get_function_declaration(nodes, function);
        if self.undefined_functions.contains(&function) {
            self.undefined_functions.remove(&function);
        } else {
            return;
        }

        self.codegen_c.func = Function::new();
        self.codegen_c.func.signature = Self::get_function_signature(&self.module, nodes, function);
        let mut builder = FunctionBuilder::new(&mut self.codegen_c.func, &mut self.builder_c);
        let root_block = builder.create_block();
        builder.append_block_params_for_function_params(root_block);
        builder.switch_to_block(root_block);
        let output_ptr = builder.block_params(root_block)[0];
        let (node, param_ptrs) = if let FunctionKind::InternalImplementation(node) = function {
            (
                node,
                nodes[&node]
                    .collect_parameter_nodes(node, nodes)
                    .into_iter()
                    .sorted()
                    .zip(builder.block_params(root_block)[1..].iter().copied())
                    .collect(),
            )
        } else if let FunctionKind::ExternalWrapper(node) = function {
            let output_layout = Self::node_output_layout(nodes, node);
            let mut offset = output_layout.len();
            let mut parameter_ptrs = HashMap::new();
            for parameter in nodes[&node]
                .collect_parameter_nodes(node, nodes)
                .into_iter()
                .sorted()
            {
                let parameter_ptr = builder.ins().iadd_imm(output_ptr, offset as i64);
                offset += Self::node_output_layout(nodes, parameter).len();
                parameter_ptrs.insert(parameter, parameter_ptr);
            }
            (node, parameter_ptrs)
        } else {
            todo!()
        };
        let ctx = NodeDefinitionContext {
            func_builder: &mut builder,
            constants: &mut self.constants,
            data_c: &mut self.data_c,
            module: &mut self.module,
            param_ptrs: &param_ptrs,
            functions: &mut self.functions,
            undefined_functions: &mut self.undefined_functions,
            nodes,
            node,
        };
        if let FunctionKind::InternalImplementation(..) = function {
            Self::compile_node_to_instructions(ctx, output_ptr);
        } else if let FunctionKind::ExternalWrapper(..) = function {
            Self::compile_node_wrapper(ctx, output_ptr);
        }
        builder.ins().return_(&[]);
        builder.seal_block(root_block);
        builder.finalize();
        if self.previously_defined_functions.contains(&function) {
            self.module.prepare_for_function_redefine(func_id).unwrap();
        } else {
            self.previously_defined_functions.insert(function);
        }
        println!("{:#?}", self.codegen_c.func);
        self.module
            .define_function(func_id, &mut self.codegen_c)
            .unwrap();
    }

    unsafe fn execute_node_implementation<IO>(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        node: NodeId,
        io: &mut IO,
    ) {
        let id = self.get_function_declaration(nodes, FunctionKind::ExternalWrapper(node));
        let func = self.module.get_finalized_function(id);
        let func = std::mem::transmute::<_, fn(&mut IO)>(func);
        func(io);
    }

    /// Optimized way to execute a node multiple times in a row
    /// (execute_node_implementation has to look up the implementation every
    /// time you invoke it, which contributes significantly to the performance
    /// of very small functions.)
    unsafe fn execute_node_implementation_several_times<IO>(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        node: NodeId,
        io: &mut IO,
        times: usize,
        mut setup: impl FnMut(&mut IO, usize),
        mut teardown: impl FnMut(&mut IO, usize),
    ) {
        let id = self.get_function_declaration(nodes, FunctionKind::ExternalWrapper(node));
        let func = self.module.get_finalized_function(id);
        let func = std::mem::transmute::<_, fn(&mut IO)>(func);
        for time in 0..times {
            setup(io, time);
            func(io);
            teardown(io, time);
        }
    }

    fn load_global_data(c: NodeDefinitionContext, ty: Type, id: DataId, output_ptr: Value) {
        let local_id = c.module.declare_data_in_func(id, c.func_builder.func);
        let ptr_type = c.module.target_config().pointer_type();
        let ptr = c.func_builder.ins().symbol_value(ptr_type, local_id);
        let value = c.func_builder.ins().load(ty, MemFlags::new(), ptr, 0);
        c.func_builder
            .ins()
            .store(MemFlags::new(), value, output_ptr, 0);
    }

    fn node_output_layout(nodes: &HashMap<NodeId, Node>, node: NodeId) -> DataLayout {
        let node = &nodes[&node];
        match &node.operation {
            NodeOperation::Literal(Value2::Float(..)) => DataLayout::Float,
            NodeOperation::Literal(Value2::Integer(..)) => DataLayout::Int,
            NodeOperation::Literal(..) => todo!(),
            NodeOperation::Parameter(_) => Self::node_output_layout(nodes, node.input.unwrap()),
            NodeOperation::Basic(_) => Self::node_output_layout(nodes, node.input.unwrap()),
            NodeOperation::ComposeStruct(_, component_names) => {
                let mut components = Vec::new();
                for (label, &value) in component_names.iter().zip(node.arguments.iter()) {
                    components.push((label.clone(), Self::node_output_layout(nodes, value)));
                }
                DataLayout::Struct { components }
            }
            NodeOperation::ComposeColor => todo!(),
            NodeOperation::GetComponent(name) => {
                let DataLayout::Struct { components }= Self::node_output_layout(nodes, node.input.unwrap()) else { panic!() };
                components.into_iter().find(|x| &x.0 == name).unwrap().1
            }
            NodeOperation::CustomNode { result, .. } => Self::node_output_layout(nodes, *result),
        }
    }

    fn compile_node_to_instructions(mut c: NodeDefinitionContext, output_ptr: Value) {
        let node = &c.nodes[&c.node];
        match &node.operation {
            NodeOperation::Literal(value) => {
                let data =
                    Self::get_constant_declaration(c.constants, c.data_c, c.module, c.node, value);
                match value {
                    Value2::Boolean(_) => todo!(),
                    Value2::Integer(_) => Self::load_global_data(c, types::I32, data, output_ptr),
                    Value2::Float(_) => Self::load_global_data(c, types::F32, data, output_ptr),
                    Value2::String(_) => todo!(),
                    Value2::Struct {
                        name: _,
                        components: _,
                    } => todo!(),
                    Value2::Invalid => todo!(),
                }
            }
            NodeOperation::Parameter(_) => {
                let source_ptr = c.param_ptrs[&c.node];
                let len = Self::node_output_layout(c.nodes, c.nodes[&c.node].input.unwrap()).len();
                c.func_builder.emit_small_memory_copy(
                    c.module.target_config(),
                    output_ptr,
                    source_ptr,
                    len as u64,
                    1,
                    1,
                    true,
                    MemFlags::new(),
                );
            }
            NodeOperation::Basic(op) => {
                let input = node.input.unwrap();
                let argument = node.arguments[0];
                drop(node);
                Self::compile_node_to_instructions(c.reborrow(input), output_ptr);
                let argument_ss = c
                    .func_builder
                    .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 4));
                let argument_ptr = c.func_builder.ins().stack_addr(
                    c.module.target_config().pointer_type(),
                    argument_ss,
                    0,
                );
                Self::compile_node_to_instructions(c.reborrow(argument), argument_ptr);
                let input = c
                    .func_builder
                    .ins()
                    .load(types::F32, MemFlags::new(), output_ptr, 0);
                let argument =
                    c.func_builder
                        .ins()
                        .load(types::F32, MemFlags::new(), argument_ptr, 0);
                let result = match op {
                    BasicOp::Add => c.func_builder.ins().fadd(input, argument),
                    BasicOp::Subtract => c.func_builder.ins().fsub(input, argument),
                    BasicOp::Multiply => c.func_builder.ins().fmul(input, argument),
                    BasicOp::Divide => c.func_builder.ins().fdiv(input, argument),
                };
                c.func_builder
                    .ins()
                    .store(MemFlags::new(), result, output_ptr, 0);
                drop(c);
            }
            NodeOperation::ComposeStruct(_, _) => {
                let mut offset = 0;
                for arg in c.nodes[&c.node].arguments.clone() {
                    let offset_output = c.func_builder.ins().iadd_imm(output_ptr, offset as i64);
                    Self::compile_node_to_instructions(c.reborrow(arg), offset_output);
                    offset += Self::node_output_layout(c.nodes, arg).len();
                }
            }
            NodeOperation::ComposeColor => todo!(),
            NodeOperation::GetComponent(name) => {
                let input = c.nodes[&c.node].input.unwrap();
                let layout = Self::node_output_layout(c.nodes, input);
                let len = layout.len();
                let DataLayout::Struct { components } = layout else { panic!() };
                let stack_slot = c
                    .func_builder
                    .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, len));
                let ptr_type = c.module.target_config().pointer_type();
                let input_ptr = c.func_builder.ins().stack_addr(ptr_type, stack_slot, 0);
                Self::compile_node_to_instructions(c.reborrow(input), input_ptr);
                let mut input_component_offset = 0;
                let mut input_component_len = 0;
                for (candidate_name, layout) in components {
                    if name == &candidate_name {
                        input_component_len = layout.len();
                        break;
                    } else {
                        input_component_offset += layout.len();
                    }
                }
                let input_component_ptr = c.func_builder.ins().stack_addr(
                    ptr_type,
                    stack_slot,
                    input_component_offset as i32,
                );
                assert!(input_component_offset < len);
                c.func_builder.emit_small_memory_copy(
                    c.module.target_config(),
                    output_ptr,
                    input_component_ptr,
                    input_component_len as u64,
                    1,
                    1,
                    true,
                    MemFlags::new(),
                );
            }
            NodeOperation::CustomNode { result, .. } => {
                let func = Self::get_function_declaration_impl(
                    c.functions,
                    c.undefined_functions,
                    c.module,
                    c.nodes,
                    FunctionKind::InternalImplementation(*result),
                );
                let func = c.module.declare_func_in_func(func, c.func_builder.func);
                let node = &c.nodes[&c.node];
                let arg_nodes = node.input.iter().chain(node.arguments.iter()).copied();
                let mut args = vec![output_ptr];
                for arg in arg_nodes {
                    let layout = Self::node_output_layout(c.nodes, arg);
                    let len = layout.len();
                    let stack_slot = c.func_builder.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        len,
                    ));
                    let ptr_type = c.module.target_config().pointer_type();
                    let arg_ptr = c.func_builder.ins().stack_addr(ptr_type, stack_slot, 0);
                    Self::compile_node_to_instructions(c.reborrow(arg), arg_ptr);
                    args.push(arg_ptr);
                }
                c.func_builder.ins().call(func, &args);
            }
        }
    }

    fn compile_node_wrapper(c: NodeDefinitionContext, output_ptr: Value) {
        let fun = Self::get_function_declaration_impl(
            c.functions,
            c.undefined_functions,
            c.module,
            c.nodes,
            FunctionKind::InternalImplementation(c.node),
        );
        let fun = c.module.declare_func_in_func(fun, c.func_builder.func);
        let args = std::iter::once(output_ptr)
            .chain(
                c.param_ptrs
                    .iter()
                    .sorted_by_key(|x| *x.0)
                    .map(|x| x.1)
                    .copied(),
            )
            .collect_vec();
        c.func_builder.ins().call(fun, &args);
    }
}

pub enum DataLayout {
    Float,
    Int,
    Struct {
        components: Vec<(String, DataLayout)>,
    },
}
impl DataLayout {
    fn len(&self) -> u32 {
        match self {
            DataLayout::Float | DataLayout::Int => 4,
            DataLayout::Struct { components } => components.iter().map(|c| c.1.len()).sum(),
        }
    }
}

pub struct Parameter {}

#[derive(Debug)]
pub struct ParameterDescription {
    pub id: ParameterId,
    pub name: String,
    pub default: Value2,
}

pub struct Tool {
    pub target_prototype: NodeId,
    pub mouse_drag_handler: NodeId,
}

pub type ToolId = Id<Tool>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value2 {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    String(String),
    Struct {
        name: String,
        components: Vec<(String, Value2)>,
    },
    // Division by zero, index out of bounds, etc.
    Invalid,
}

impl Value2 {
    pub fn display(&self) -> String {
        if let &Value2::Float(value) = self {
            util::pretty_format_number(value)
        } else {
            self.cast(BaseType::String)
                .unwrap()
                .as_string()
                .unwrap()
                .clone()
        }
    }

    fn as_ne_bytes(&self) -> Vec<u8> {
        match self {
            Value2::Boolean(_) => todo!(),
            Value2::Integer(value) => value.to_ne_bytes().into(),
            Value2::Float(value) => value.to_ne_bytes().into(),
            Value2::String(_) => todo!(),
            Value2::Struct {
                name: _,
                components: _,
            } => todo!(),
            Value2::Invalid => todo!(),
        }
    }
}

impl From<String> for Value2 {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<f32> for Value2 {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<i32> for Value2 {
    fn from(v: i32) -> Self {
        Self::Integer(v)
    }
}

impl From<bool> for Value2 {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl Value2 {
    pub fn r#type(&self) -> BaseType {
        use BaseType::*;
        match self {
            Self::Boolean(..) => Boolean,
            Self::Integer(..) => Integer,
            Self::Float(..) => Float,
            Self::String(..) => String,
            _ => todo!("{:#?}", self),
        }
    }

    pub fn cast(&self, to: BaseType) -> Option<Self> {
        use BaseType::*;
        match self {
            &Self::Boolean(value) => match to {
                Boolean => Some(value.into()),
                String => Some(if value { "True" } else { "False" }.to_owned().into()),
                _ => Self::Integer(if value { 1 } else { 0 }).cast(to),
            },
            &Self::Integer(value) => match to {
                Boolean => Some((value != 0).into()),
                Integer => Some(value.into()),
                Float => Some((value as f32).into()),
                String => Some(format!("{}", value).into()),
            },
            &Self::Float(value) => match to {
                Boolean => Some((value != 0.0).into()),
                Integer => Some((value as i32).into()),
                Float => Some(value.into()),
                String => Some(format!("{}", value).into()),
            },
            Self::String(value) => match to {
                String => Some(value.clone().into()),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        if let Self::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseType {
    Boolean,
    Integer,
    Float,
    String,
}

impl BaseType {
    fn rank(self) -> i32 {
        match self {
            BaseType::Boolean => 0,
            BaseType::Integer => 1,
            BaseType::Float => 2,
            BaseType::String => 3,
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self.rank() > other.rank() {
            self
        } else {
            other
        }
    }
}

pub struct Type2 {
    pub base: BaseType,
    pub deferred_parameters: Vec<ParameterId>,
}

impl Type2 {}

impl From<BaseType> for Type2 {
    fn from(base: BaseType) -> Self {
        Self {
            base,
            deferred_parameters: vec![],
        }
    }
}

pub type NodeId = Id<Node>;
pub type ParameterId = Id<Parameter>;

pub struct Engine {
    nodes: HashMap<NodeId, Node>,
    dirty_nodes: HashSet<NodeId>,
    root_node: NodeId,
    tools: HashMap<ToolId, Tool>,
    node_ids: IdCreator<Node>,
    parameter_ids: IdCreator<Parameter>,
    tool_ids: IdCreator<Tool>,
    context: CodeGenerationContext,
}

pub struct BuiltinDefinitions {
    pub x_component: ParameterId,
    pub y_component: ParameterId,
    pub compose_vector_2d: NodeId,
    pub compose_integer_vector_2d: NodeId,
    pub mouse_offset: (ParameterId, NodeId),
    pub display_position: (ParameterId, NodeId),
    pub adjust_float_tool: ToolId,
}

impl Index<NodeId> for Engine {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        self.nodes.get(&index).unwrap()
    }
}

impl IndexMut<NodeId> for Engine {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        self.nodes.get_mut(&index).unwrap()
    }
}

impl Engine {
    pub fn new() -> (Self, BuiltinDefinitions) {
        let start_node = Node {
            operation: NodeOperation::Literal(1.0.into()),
            input: None,
            arguments: vec![],
        };
        let mut node_ids = IdCreator::new();
        let root_node = node_ids.next();
        let parameter_ids = IdCreator::new();
        let tool_ids = IdCreator::new();
        let context = CodeGenerationContext::new();
        let mut this = Self {
            nodes: hashmap! [root_node => start_node],
            dirty_nodes: hashset![root_node],
            tools: hashmap![],
            root_node,
            node_ids,
            parameter_ids,
            tool_ids,
            context,
        };
        let builtins = this.make_builtins();
        this.setup_demo(&builtins);
        (this, builtins)
    }

    fn make_builtins(&mut self) -> BuiltinDefinitions {
        let _default_struct = Value2::Struct {
            name: "Empty Struct".to_owned(),
            components: vec![],
        };
        let (compose_integer_vector_2d, _integer_vector_2d_parameters) = self
            .push_simple_struct_composer(
                "Compose Integer Vector/2D",
                vec![("X", 0.into()), ("Y", 0.into())],
            );
        let (compose_vector_2d, vector_2d_parameters) = self.push_simple_struct_composer(
            "Compose Vector/2D",
            vec![("X", 0.0.into()), ("Y", 0.0.into())],
        );
        let zero = self.push_literal_node(0.0.into());
        let default_vec2 = self.push_node(Node {
            operation: NodeOperation::CustomNode {
                result: compose_vector_2d,
                input: None,
            },
            input: None,
            arguments: vec![zero, zero],
        });
        let name = self.push_literal_node("Mouse Offset".to_owned().into());
        let mouse_offset = self.push_parameter(name, default_vec2);

        let zero = self.push_literal_node(0.into());
        let default_ivec2 = self.push_node(Node {
            operation: NodeOperation::CustomNode {
                result: compose_integer_vector_2d,
                input: None,
            },
            input: None,
            arguments: vec![zero, zero],
        });
        let name = self.push_literal_node("Display Position".to_owned().into());
        let display_position = self.push_parameter(name, default_ivec2);

        let (prototype, target) = {
            let target = self.push_simple_parameter("SPECIAL TOOL TARGET Factor", 1.0.into());
            let input = self.push_simple_parameter("SPECIAL TOOL WILDCARD INPUT", 0.0.into());
            let prototype = self.push_node(Node {
                operation: NodeOperation::Basic(BasicOp::Multiply),
                input: Some(input),
                arguments: vec![target],
            });
            (prototype, target)
        };
        let drag_handler = {
            let mouse_offset = mouse_offset.1;
            let dx = self.push_get_component(mouse_offset, "X");
            let dy = self.push_get_component(mouse_offset, "Y");
            let dx_plus_dy = self.push_node(Node {
                operation: NodeOperation::Basic(BasicOp::Add),
                input: Some(dx),
                arguments: vec![dy],
            });
            let scale = self.push_literal_node(0.01.into());
            let delta = self.push_node(Node {
                operation: NodeOperation::Basic(BasicOp::Multiply),
                input: Some(dx_plus_dy),
                arguments: vec![scale],
            });
            self.push_node(Node {
                operation: NodeOperation::Basic(BasicOp::Add),
                input: Some(target),
                arguments: vec![delta],
            })
        };
        let adjust_float_tool = self.add_tool(Tool {
            target_prototype: prototype,
            mouse_drag_handler: drag_handler,
        });
        BuiltinDefinitions {
            x_component: vector_2d_parameters[0],
            y_component: vector_2d_parameters[1],
            compose_vector_2d,
            compose_integer_vector_2d,
            mouse_offset,
            display_position,
            adjust_float_tool,
        }
    }

    pub fn compile(&mut self, node: NodeId) {
        self.context
            .define_function_implementation(&self.nodes, FunctionKind::ExternalWrapper(node));
    }

    pub fn write_constant_data(&mut self, node: NodeId, data: &Value2) {
        self.context.write_constant_data(node, data);
    }

    pub unsafe fn execute<IO>(&mut self, node: NodeId, io: &mut IO) {
        self.context
            .execute_node_implementation(&self.nodes, node, io)
    }

    pub unsafe fn execute_multiple_times<IO>(
        &mut self,
        node: NodeId,
        io: &mut IO,
        times: usize,
        setup: impl FnMut(&mut IO, usize),
        teardown: impl FnMut(&mut IO, usize),
    ) {
        self.context.execute_node_implementation_several_times(
            &self.nodes,
            node,
            io,
            times,
            setup,
            teardown,
        )
    }

    fn add_tool(&mut self, tool: Tool) -> ToolId {
        let id = self.tool_ids.next();
        self.tools.insert(id, tool);
        id
    }

    pub fn get_tool(&self, tool: ToolId) -> &Tool {
        &self.tools[&tool]
    }

    fn setup_demo(&mut self, builtins: &BuiltinDefinitions) {
        // let value = self.root_node();
        // let param1 = self.push_literal_node(2.0.into());
        // let value = self.push_node(Node {
        //     operation: NodeOperation::Basic(BasicOp::Multiply),
        //     input: Some(value),
        //     arguments: vec![param1],
        // });
        // // let param2 = self.push_simple_parameter("Value", 123.0.into());
        // let param2 = self.push_literal_node(123.0.into());
        // let root = self.push_node(Node {
        //     operation: NodeOperation::Basic(BasicOp::Add),
        //     input: Some(value),
        //     arguments: vec![param2],
        // });

        // let vec = self.push_simple_struct("Vector/2D", vec![("X", 1.0.into()), ("Y",
        // 2.0.into())]);
        // let one = self.push_literal_node(1.0.into());
        // let two = self.push_literal_node(2.0.into());
        // let vec = self.push_node(Node {
        //     operation: NodeOperation::CustomNode {
        //         result: builtins.compose_vector_2d,
        //         input: None,
        //     },
        //     input: None,
        //     arguments: vec![one, two],
        // });
        // let value = self.push_get_component(vec, "X");
        let value = self.push_get_component(builtins.display_position.1, "Y");
        let divisor = self.push_literal_node(360.0.into());
        let root = self.push_node(Node {
            operation: NodeOperation::Basic(BasicOp::Divide),
            input: Some(value),
            arguments: vec![divisor],
        });

        self.set_root(root);
    }

    pub fn push_simple_struct(&mut self, name: &str, components: Vec<(&str, Value2)>) -> NodeId {
        let mut args = vec![];
        for (_, component) in &components {
            args.push(self.push_literal_node(component.clone()));
        }
        let node = self.push_node(Node {
            operation: NodeOperation::ComposeStruct(
                name.to_owned(),
                components.iter().map(|x| x.0.to_owned()).collect_vec(),
            ),
            input: None,
            arguments: args,
        });
        node
    }

    pub fn push_simple_struct_composer(
        &mut self,
        name: &str,
        default_components: Vec<(&str, Value2)>,
    ) -> (NodeId, Vec<ParameterId>) {
        let mut args = vec![];
        let mut parameters = vec![];
        for (name, default) in &default_components {
            let name = self.push_literal_node(name.to_owned().to_owned().into());
            let default = self.push_literal_node(default.clone());
            let (param, arg) = self.push_parameter(name, default);
            args.push(arg);
            parameters.push(param);
        }
        let node = self.push_node(Node {
            operation: NodeOperation::ComposeStruct(
                name.to_owned(),
                default_components
                    .iter()
                    .map(|x| x.0.to_owned())
                    .collect_vec(),
            ),
            input: None,
            arguments: args,
        });
        (node, parameters)
    }

    pub fn push_node(&mut self, node: Node) -> NodeId {
        let id = self.node_ids.next();
        self.nodes.insert(id, node);
        self.dirty_nodes.insert(id);
        id
    }

    pub fn push_literal_node(&mut self, value: Value2) -> NodeId {
        self.push_node(Node {
            operation: NodeOperation::Literal(value),
            input: None,
            arguments: vec![],
        })
    }

    pub fn push_get_component(&mut self, input: NodeId, component_name: &str) -> NodeId {
        self.push_node(Node {
            operation: NodeOperation::GetComponent(component_name.into()),
            input: Some(input),
            arguments: vec![],
        })
    }

    pub fn push_simple_parameter(&mut self, name: &str, default_value: Value2) -> NodeId {
        let param_name = self.push_literal_node(name.to_owned().into());
        let param_default = self.push_literal_node(default_value);
        let (_, param) = self.push_parameter(param_name, param_default);
        param
    }

    pub fn push_parameter(&mut self, name: NodeId, default_value: NodeId) -> (ParameterId, NodeId) {
        let id = self.parameter_ids.next();
        let node = Node {
            operation: NodeOperation::Parameter(id),
            input: Some(default_value),
            arguments: vec![name],
        };
        let node_id = self.push_node(node);
        (id, node_id)
    }

    pub fn set_root(&mut self, node: NodeId) {
        self.root_node = node;
    }

    pub fn root_node(&self) -> NodeId {
        self.root_node
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.values_mut()
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        self.dirty_nodes.insert(node);
        let mut other_dirty = Vec::new();
        for (id, other) in &self.nodes {
            if other
                .arguments
                .iter()
                .chain(other.input.iter())
                .any(|inp| *inp == node)
            {
                other_dirty.push(*id);
                continue;
            }
        }
        for id in other_dirty {
            self.mark_dirty(id);
        }
    }
}

pub struct Node {
    pub operation: NodeOperation,
    pub input: Option<NodeId>,
    pub arguments: Vec<NodeId>,
}

impl Node {
    pub fn collect_parameter_nodes(
        &self,
        my_id: NodeId,
        nodes: &HashMap<NodeId, Node>,
    ) -> HashSet<NodeId> {
        if let NodeOperation::Parameter(..) = &self.operation {
            hashset![my_id]
        } else {
            self.arguments
                .iter()
                .chain(self.input.iter())
                .flat_map(|node_id| {
                    nodes[node_id]
                        .collect_parameter_nodes(*node_id, nodes)
                        .into_iter()
                })
                .collect()
        }
    }

    pub fn collect_parameters(&self, engine: &Engine) -> Vec<ParameterDescription> {
        let mut into = Vec::new();
        self.collect_parameters_into(engine, &mut into);
        into
    }

    pub fn collect_parameters_into(&self, engine: &Engine, into: &mut Vec<ParameterDescription>) {
        if let &NodeOperation::Parameter(id) = &self.operation {
            into.push(ParameterDescription {
                id,
                name: engine[self.arguments[0]]
                    .evaluate(engine, &HashMap::new())
                    .as_string()
                    .unwrap()
                    .clone(),
                default: engine[self.input.unwrap()].evaluate(engine, &HashMap::new()),
            });
        } else {
            if let Some(input) = self.input {
                engine[input].collect_parameters_into(engine, into);
            }
            for &arg in &self.arguments {
                engine[arg].collect_parameters_into(engine, into);
            }
        }
    }

    pub fn evaluate(&self, engine: &Engine, arguments: &HashMap<ParameterId, Value2>) -> Value2 {
        match &self.operation {
            NodeOperation::Literal(value) => value.clone(),
            NodeOperation::Parameter(id) => arguments
                .get(id)
                .cloned()
                .unwrap_or_else(|| engine[self.input.unwrap()].evaluate(engine, arguments)),
            NodeOperation::Basic(op) => {
                let a = engine[self.input.unwrap()].evaluate(engine, arguments);
                let b = engine[self.arguments[0]].evaluate(engine, arguments);
                op.combine(&a, &b)
            }
            NodeOperation::ComposeStruct(name, component_names) => {
                let name = name.clone();
                let mut components = Vec::new();
                for (component_name, component_value) in
                    component_names.iter().zip(self.arguments.iter())
                {
                    let component_value = engine[*component_value].evaluate(engine, arguments);
                    components.push((component_name.clone(), component_value));
                }
                Value2::Struct { name, components }
            }
            NodeOperation::ComposeColor => {
                let c1 = engine[self.arguments[0]].evaluate(engine, arguments);
                let c2 = engine[self.arguments[1]].evaluate(engine, arguments);
                let c3 = engine[self.arguments[2]].evaluate(engine, arguments);
                Value2::Struct {
                    name: format!("Color"),
                    components: vec![
                        (format!("Channel 1"), c1),
                        (format!("Channel 2"), c2),
                        (format!("Channel 3"), c3),
                    ],
                }
            }
            NodeOperation::GetComponent(component_name) => {
                let input = engine[self.input.unwrap()].evaluate(engine, arguments);
                if let Value2::Struct { components, .. } = input {
                    components
                        .iter()
                        .find(|x| &x.0 == component_name)
                        .unwrap()
                        .1
                        .clone()
                } else {
                    panic!("Cannot extract components from a non-struct value!")
                }
            }
            NodeOperation::CustomNode { result, input } => {
                let result = &engine[*result];
                let mut next_arguments = HashMap::new();
                if let &Some(input) = input {
                    let value = engine[self.input.unwrap()].evaluate(engine, arguments);
                    next_arguments.insert(input, value);
                }
                let params = result.collect_parameters(engine);
                for (index, param) in params.into_iter().enumerate() {
                    let arg = self.arguments[index];
                    let arg = engine[arg].evaluate(engine, arguments);
                    next_arguments.insert(param.id, arg);
                }
                result.evaluate(engine, &next_arguments)
            }
        }
    }

    pub fn as_literal(&self) -> &Value2 {
        if let NodeOperation::Literal(literal) = &self.operation {
            literal
        } else {
            panic!("Not a literal.")
        }
    }

    pub fn as_literal_mut(&mut self) -> &mut Value2 {
        if let NodeOperation::Literal(literal) = &mut self.operation {
            literal
        } else {
            panic!("Not a literal.")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeOperation {
    Literal(Value2),
    Parameter(ParameterId),
    Basic(BasicOp),
    ComposeStruct(String, Vec<String>),
    ComposeColor,
    GetComponent(String),
    CustomNode {
        result: NodeId,
        input: Option<ParameterId>,
    },
}

impl NodeOperation {
    pub fn name(&self) -> String {
        use NodeOperation::*;
        match self {
            Literal(value) => value.display(),
            Parameter(..) => format!("Parameter"),
            Basic(op) => op.name().to_owned(),
            ComposeStruct(name, ..) => format!("Make {}", name),
            ComposeColor => format!("Compose Color"),
            GetComponent(component_name) => format!("Get {}", component_name),
            CustomNode { .. } => format!("Todo"),
        }
    }

    pub fn param_name<'a>(
        &'a self,
        index: usize,
        parameters: &'a [ParameterDescription],
    ) -> &'a str {
        use NodeOperation::*;
        match self {
            ComposeStruct(_, component_names) => &component_names[index],
            CustomNode { input, result } => "todo",
            _ => {
                let names = self.param_names();
                names[index.min(names.len() - 1)]
            }
        }
    }

    fn param_names(&self) -> Vec<&str> {
        use NodeOperation::*;
        match self {
            Literal(..) => vec![],
            Parameter(..) => vec!["Name"],
            Basic(op) => Vec::from(op.param_names()),
            ComposeStruct(_, field_names) => field_names.iter().map(|x| &x[..]).collect_vec(),
            ComposeColor => vec!["Channel 1", "Channel 2", "Channel 3"],
            GetComponent(..) => vec![],
            CustomNode { .. } => vec!["This Label Shouldn't Show Up"],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BasicOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BasicOp {
    pub fn name(&self) -> &'static str {
        use BasicOp::*;
        match self {
            Add => "Add",
            Subtract => "Subtract",
            Multiply => "Multiply",
            Divide => "Divide",
        }
    }

    fn param_names(&self) -> &'static [&'static str] {
        use BasicOp::*;
        match self {
            Add => &["Offset"],
            Subtract => &["Offset"],
            Multiply => &["Factor"],
            Divide => &["Divisor"],
        }
    }

    fn combine(&self, a: &Value2, b: &Value2) -> Value2 {
        let supertype = a.r#type().max(b.r#type());
        let a = a.cast(supertype).unwrap();
        let b = b.cast(supertype).unwrap();
        match (a, b) {
            (Value2::Boolean(a), Value2::Boolean(b)) => self.combine_booleans(a, b).into(),
            (Value2::Integer(a), Value2::Integer(b)) => self.combine_integers(a, b).into(),
            (Value2::Float(a), Value2::Float(b)) => self.combine_floats(a, b).into(),
            (Value2::String(a), Value2::String(b)) => self.combine_strings(a, b).into(),
            _ => unreachable!("Values should be the same type."),
        }
    }

    fn combine_booleans(&self, a: bool, b: bool) -> bool {
        use BasicOp::*;
        match self {
            Add => a || b,
            Subtract => a && !b,
            Multiply => a && b,
            _ => panic!("Invalid boolean operation"),
        }
    }

    fn combine_integers(&self, a: i32, b: i32) -> i32 {
        use BasicOp::*;
        match self {
            Add => a + b,
            Subtract => a - b,
            Multiply => a * b,
            Divide => a / b,
        }
    }

    fn combine_floats(&self, a: f32, b: f32) -> f32 {
        use BasicOp::*;
        match self {
            Add => a + b,
            Subtract => a - b,
            Multiply => a * b,
            Divide => a / b,
        }
    }

    fn combine_strings(&self, a: String, b: String) -> String {
        use BasicOp::*;
        match self {
            Add => format!("{}{}", a, b),
            Multiply => format!("{}{}", a, b),
            _ => panic!("Invalid string operation"),
        }
    }
}
