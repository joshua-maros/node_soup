mod blob;
mod layout;

use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
    ops::{Index, IndexMut},
};

pub use blob::*;
use bytemuck::Zeroable;
use cranelift::{
    codegen::{ir::Function, Context},
    prelude::*,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, DataId, FuncId, Linkage, Module};
use itertools::Itertools;
pub use layout::*;
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
    constants: HashMap<NodeId, (DataId, ObjectLayout)>,
    undefined_functions: HashSet<FunctionKind>,
    previously_defined_functions: HashSet<FunctionKind>,
}

struct NodeDefinitionContext<'x, 'f> {
    func_builder: &'x mut FunctionBuilder<'f>,
    constants: &'x mut HashMap<NodeId, (DataId, ObjectLayout)>,
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
        constants: &mut HashMap<NodeId, (DataId, ObjectLayout)>,
        data_c: &mut DataContext,
        module: &mut JITModule,
        node: NodeId,
        data: Blob,
    ) -> DataId {
        constants
            .entry(node)
            .or_insert_with(|| {
                let (layout, bytes) = data.leak();
                data_c.define(bytes);
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
                (id, layout)
            })
            .0
    }

    fn write_constant_data(&mut self, node: NodeId, data: Blob) {
        let (buffer_id, buffer_layout) = &self.constants[&node];
        let buffer = self.module.get_finalized_data(*buffer_id);
        let slice = unsafe { std::slice::from_raw_parts_mut(buffer.0.cast_mut(), buffer.1) };
        // TODO: Fix leaked dynamic components.
        assert_eq!(slice.len(), data.layout().size() as usize);
        let (data_layout, bytes) = data.leak();
        assert_eq!(slice.len(), bytes.len());
        assert_eq!(buffer_layout, &data_layout);
        slice.copy_from_slice(&bytes);
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
            let mut offset = output_layout.size();
            let mut parameter_ptrs = HashMap::new();
            for parameter in nodes[&node]
                .collect_parameter_nodes(node, nodes)
                .into_iter()
                .sorted()
            {
                let parameter_ptr = builder.ins().iadd_imm(output_ptr, offset as i64);
                offset += Self::node_output_layout(nodes, parameter).size();
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

    fn execute_node_implementation(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        node: NodeId,
        io: &mut Blob,
    ) {
        assert_eq!(io.layout(), &Self::io_layout(nodes, node));
        assert!(io.layout().is_fixed());
        let id = self.get_function_declaration(nodes, FunctionKind::ExternalWrapper(node));
        let func = self.module.get_finalized_function(id);
        let func = unsafe { std::mem::transmute::<_, fn(&mut u8)>(func) };
        func(unsafe { &mut io.as_raw_bytes_mut()[0] });
    }

    /// Optimized way to execute a node multiple times in a row
    /// (execute_node_implementation has to look up the implementation every
    /// time you invoke it, which contributes significantly to the performance
    /// of very small functions.)
    fn execute_node_implementation_several_times(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        node: NodeId,
        io: &mut Blob,
        times: usize,
        mut setup: impl FnMut(&mut Blob, usize),
        mut teardown: impl FnMut(&mut Blob, usize),
    ) {
        assert_eq!(io.layout(), &Self::io_layout(nodes, node));
        assert!(io.layout().is_fixed());
        let id = self.get_function_declaration(nodes, FunctionKind::ExternalWrapper(node));
        let func = self.module.get_finalized_function(id);
        let func = unsafe { std::mem::transmute::<_, fn(&mut u8)>(func) };
        for time in 0..times {
            setup(io, time);
            func(unsafe { &mut io.as_raw_bytes_mut()[0] });
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

    fn node_output_layout(nodes: &HashMap<NodeId, Node>, node: NodeId) -> ObjectLayout {
        let node = &nodes[&node];
        match &node.operation {
            NodeOperation::Literal(lit) => lit.layout().clone(),
            NodeOperation::Parameter(_) => Self::node_output_layout(nodes, node.input.unwrap()),
            NodeOperation::Basic(_) => Self::node_output_layout(nodes, node.input.unwrap()),
            NodeOperation::ComposeStruct(_, component_names) => {
                let mut keys = Vec::new();
                let mut value_types = Vec::new();
                for (label, &value) in component_names.iter().zip(node.arguments.iter()) {
                    keys.push(Blob::from(label.clone()));
                    value_types.push(Self::node_output_layout(nodes, value));
                }
                ObjectLayout::FixedHeterogeneousMap(Box::new(Blob::fixed_array(keys)), value_types)
            }
            NodeOperation::GetComponent(name) => {
                let layout = Self::node_output_layout(nodes, node.input.unwrap());
                layout
                    .layout_after_index(Some(&Blob::from(name.clone())))
                    .clone()
            }
            NodeOperation::CustomNode { result, .. } => Self::node_output_layout(nodes, *result),
        }
    }

    fn io_layout(nodes: &HashMap<NodeId, Node>, node: NodeId) -> ObjectLayout {
        let output_layout = CodeGenerationContext::node_output_layout(nodes, node);
        let mut keys = vec![(format!("OUTPUT"), output_layout)];
        let params = nodes[&node].collect_parameter_nodes(node, nodes);
        for param in params.into_iter().sorted() {
            let param = &nodes[&param].collect_parameters(nodes)[0];
            keys.push((
                format!("INPUT {}", param.name),
                Self::node_output_layout(nodes, param.default),
            ));
        }
        let keys_blob = Blob::fixed_array(
            keys.iter()
                .map(|key| Blob::from(key.0.clone()))
                .collect_vec(),
        );
        let eltypes = keys.into_iter().map(|x| x.1).collect();
        ObjectLayout::FixedHeterogeneousMap(Box::new(keys_blob), eltypes)
    }

    fn compile_node_to_instructions(mut c: NodeDefinitionContext, output_ptr: Value) {
        let node = &c.nodes[&c.node];
        match &node.operation {
            NodeOperation::Literal(value) => {
                let data = Self::get_constant_declaration(
                    c.constants,
                    c.data_c,
                    c.module,
                    c.node,
                    value.clone(),
                );
                match value.layout() {
                    ObjectLayout::Integer => {
                        Self::load_global_data(c, types::I32, data, output_ptr)
                    }
                    ObjectLayout::Float => Self::load_global_data(c, types::F32, data, output_ptr),
                    _ => (),
                }
            }
            NodeOperation::Parameter(_) => {
                let source_ptr = c.param_ptrs[&c.node];
                let len = Self::node_output_layout(c.nodes, c.nodes[&c.node].input.unwrap()).size();
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
                    offset += Self::node_output_layout(c.nodes, arg).size();
                }
            }
            NodeOperation::GetComponent(name) => {
                let input = c.nodes[&c.node].input.unwrap();
                let layout = Self::node_output_layout(c.nodes, input);
                let len = layout.size();
                let ObjectLayout::FixedHeterogeneousMap(keys, value_layouts) = layout else { panic!() };
                let stack_slot = c
                    .func_builder
                    .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, len));
                let ptr_type = c.module.target_config().pointer_type();
                let input_ptr = c.func_builder.ins().stack_addr(ptr_type, stack_slot, 0);
                Self::compile_node_to_instructions(c.reborrow(input), input_ptr);
                let mut input_component_offset = 0;
                let mut input_component_len = 0;
                let keys = keys.view();
                let name = Blob::from(name.clone());
                for index in 0..keys.len().unwrap() {
                    let component_layout = &value_layouts[index as usize];
                    if keys.index(&Blob::from(index as i32)) == name.view() {
                        input_component_len = component_layout.size();
                        break;
                    } else {
                        input_component_offset += component_layout.size();
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
                    let len = layout.size();
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

pub struct Parameter {}

#[derive(Debug)]
pub struct ParameterDescription {
    pub id: ParameterId,
    pub name: String,
    pub default: NodeId,
}

pub struct Tool {
    pub target_prototype: NodeId,
    pub mouse_drag_handler: NodeId,
}

pub type ToolId = Id<Tool>;
pub type NodeId = Id<Node>;
pub type ParameterId = Id<Parameter>;

pub struct Engine {
    nodes: HashMap<NodeId, Node>,
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

    pub fn write_constant_data(&mut self, node: NodeId, data: Blob) {
        self.context.write_constant_data(node, data);
    }

    pub fn default_io_blob(&self, node: NodeId) -> Blob {
        CodeGenerationContext::io_layout(&self.nodes, node).default_blob()
    }

    pub fn execute(&mut self, node: NodeId, io: &mut Blob) {
        self.compile(node);
        self.context
            .execute_node_implementation(&self.nodes, node, io)
    }

    pub fn execute_multiple_times(
        &mut self,
        node: NodeId,
        io: &mut Blob,
        times: usize,
        setup: impl FnMut(&mut Blob, usize),
        teardown: impl FnMut(&mut Blob, usize),
    ) {
        self.compile(node);
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
        let divisor = self.push_literal_node(180.0.into());
        let root = self.push_node(Node {
            operation: NodeOperation::Basic(BasicOp::Divide),
            input: Some(value),
            arguments: vec![divisor],
        });

        self.set_root(root);
    }

    pub fn push_simple_struct(&mut self, name: &str, components: Vec<(&str, Blob)>) -> NodeId {
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
        default_components: Vec<(&str, Blob)>,
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
        id
    }

    pub fn push_literal_node(&mut self, value: Blob) -> NodeId {
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

    pub fn push_simple_parameter(&mut self, name: &str, default_value: Blob) -> NodeId {
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

    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.values_mut()
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        self.context.undefined_functions.insert(FunctionKind::ExternalWrapper(node));
        self.context.undefined_functions.insert(FunctionKind::InternalImplementation(node));
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

    pub fn collect_parameters(&self, nodes: &HashMap<NodeId, Node>) -> Vec<ParameterDescription> {
        let mut into = Vec::new();
        self.collect_parameters_into(nodes, &mut into);
        into
    }

    pub fn collect_parameters_into(
        &self,
        nodes: &HashMap<NodeId, Node>,
        into: &mut Vec<ParameterDescription>,
    ) {
        if let &NodeOperation::Parameter(id) = &self.operation {
            into.push(ParameterDescription {
                id,
                name: nodes[&self.arguments[0]]
                    .as_literal()
                    .view()
                    .as_string()
                    .unwrap()
                    .to_owned(),
                default: self.input.unwrap(),
            });
        } else {
            if let Some(input) = self.input {
                nodes[&input].collect_parameters_into(nodes, into);
            }
            for &arg in &self.arguments {
                nodes[&arg].collect_parameters_into(nodes, into);
            }
        }
    }

    pub fn as_literal(&self) -> &Blob {
        if let NodeOperation::Literal(literal) = &self.operation {
            literal
        } else {
            panic!("Not a literal.")
        }
    }

    pub fn as_literal_mut(&mut self) -> &mut Blob {
        if let NodeOperation::Literal(literal) = &mut self.operation {
            literal
        } else {
            panic!("Not a literal.")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeOperation {
    Literal(Blob),
    Parameter(ParameterId),
    Basic(BasicOp),
    ComposeStruct(String, Vec<String>),
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
            Literal(value) => format!("{:?}", value.view()),
            Parameter(..) => format!("Parameter"),
            Basic(op) => op.name().to_owned(),
            ComposeStruct(name, ..) => format!("Make {}", name),
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
}
