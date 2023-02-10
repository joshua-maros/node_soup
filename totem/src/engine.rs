use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

use itertools::Itertools;
use maplit::hashmap;
use renderer::{Position, Shapes};

use crate::{
    bytecode::{
        BinaryOp, BytecodeInstruction, BytecodeProgram, Heap, IntegerOp, MemoryLayout, UnaryOp,
        Usage,
    },
    util::{self, Id, IdCreator},
    widgets::{self, SimpleValueWidget},
};

pub struct Parameter {}

pub struct ParameterDescription {
    pub id: ParameterId,
    pub name: String,
    pub default: Value,
}

pub struct Tool {
    pub target_prototype: NodeId,
    pub mouse_drag_handler: NodeId,
}

pub type ToolId = Id<Tool>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    String(String),
    Struct {
        name: String,
        components: Vec<(String, Value)>,
    },
    // Division by zero, index out of bounds, etc.
    Invalid,
}

impl Value {
    pub fn display(&self) -> String {
        if let &Value::Float(value) = self {
            util::pretty_format_number(value)
        } else {
            self.cast(BaseType::String)
                .unwrap()
                .as_string()
                .unwrap()
                .clone()
        }
    }

    fn size(&self) -> usize {
        match self {
            Value::Boolean(..) => todo!(),
            Value::Integer(..) => 1,
            Value::Float(..) => 1,
            Value::String(_) => todo!(),
            Value::Struct { components, .. } => components.iter().map(|c| c.1.size()).sum(),
            Value::Invalid => todo!(),
        }
    }

    fn layout(&self, start: usize) -> MemoryLayout {
        match self {
            Value::Boolean(..) => todo!(),
            Value::Integer(..) => MemoryLayout::Integer(start),
            Value::Float(..) => MemoryLayout::Float(start),
            Value::String(_) => todo!(),
            Value::Struct { components, .. } => {
                let mut layout_components = Vec::new();
                let mut start = start;
                for (name, value) in components {
                    let layout = value.layout(start);
                    start += value.size();
                    layout_components.push((name.clone(), layout));
                }
                MemoryLayout::Struct {
                    components: layout_components,
                }
            }
            Value::Invalid => todo!(),
        }
    }

    fn add_load_instructions(
        &self,
        instructions: &mut Vec<BytecodeInstruction>,
        heap: &mut Heap,
    ) -> MemoryLayout {
        let start = heap.allocate_space_for_multiple_values(self.size());
        let layout = self.layout(start);
        self.add_load_instructions_impl(instructions, &layout);
        layout
    }

    fn add_load_instructions_impl(
        &self,
        instructions: &mut Vec<BytecodeInstruction>,
        layout: &MemoryLayout,
    ) {
        match layout {
            &MemoryLayout::Integer(position) => {
                let &Self::Integer(value) = self else { panic!() };
                instructions.push(BytecodeInstruction::IntegerLiteral(value, position))
            }
            &MemoryLayout::Float(position) => {
                let &Self::Float(value) = self else { panic!() };
                instructions.push(BytecodeInstruction::FloatLiteral(value, position))
            }
            MemoryLayout::Struct {
                components: layout_components,
            } => {
                let Self::Struct { components: value_components, .. } = self else{ panic!()};
                for (index, (name, layout)) in layout_components.iter().enumerate() {
                    assert_eq!(name, &value_components[index].0);
                    value_components[index]
                        .1
                        .add_load_instructions_impl(instructions, layout);
                }
            }
        }
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Integer(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl Value {
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
    pub fn name(self) -> &'static str {
        use BaseType::*;
        match self {
            Boolean => "Boolean",
            Integer => "Integer",
            Float => "Float",
            String => "String",
        }
    }

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

pub struct Type {
    pub base: BaseType,
    pub deferred_parameters: Vec<ParameterId>,
}

impl Type {}

impl From<BaseType> for Type {
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
    root_node: NodeId,
    tools: HashMap<ToolId, Tool>,
    node_ids: IdCreator<Node>,
    parameter_ids: IdCreator<Parameter>,
    tool_ids: IdCreator<Tool>,
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
        let mut parameter_ids = IdCreator::new();
        let tool_ids = IdCreator::new();
        let mut this = Self {
            nodes: hashmap! [root_node => start_node],
            tools: hashmap![],
            root_node,
            node_ids,
            parameter_ids,
            tool_ids,
        };
        let builtins = this.make_builtins();
        this.setup_demo(&builtins);
        (this, builtins)
    }

    fn make_builtins(&mut self) -> BuiltinDefinitions {
        let default_struct = Value::Struct {
            name: "Empty Struct".to_owned(),
            components: vec![],
        };
        let (compose_integer_vector_2d, integer_vector_2d_parameters) = self
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
        // let param2 = self.push_simple_parameter("Value", 123.0.into());
        // let root = self.push_node(Node {
        //     operation: NodeOperation::Basic(BasicOp::Add),
        //     input: Some(value),
        //     arguments: vec![param2],
        // });

        let value = self.push_get_component(builtins.display_position.1, "X");
        let divisor = self.push_literal_node(360.0.into());
        let root = self.push_node(Node {
            operation: NodeOperation::Basic(BasicOp::Divide),
            input: Some(value),
            arguments: vec![divisor],
        });

        self.set_root(root);
    }

    pub fn push_simple_struct_composer(
        &mut self,
        name: &str,
        default_components: Vec<(&str, Value)>,
    ) -> (NodeId, Vec<ParameterId>) {
        let mut args = vec![self.push_literal_node(name.to_owned().into())];
        let mut parameters = vec![];
        for (name, default) in default_components {
            let name = self.push_literal_node(name.to_owned().into());
            let default = self.push_literal_node(default.clone());
            let (param, arg) = self.push_parameter(name, default);
            args.push(name);
            args.push(arg);
            parameters.push(param);
        }
        let node = self.push_node(Node {
            operation: NodeOperation::ComposeStruct,
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

    pub fn push_literal_node(&mut self, value: Value) -> NodeId {
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

    pub fn push_simple_parameter(&mut self, name: &str, default_value: Value) -> NodeId {
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
        let mut root_parameters = Vec::new();
        self[self.root_node].collect_parameters_into(self, &mut root_parameters);
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
}

pub struct Node {
    pub operation: NodeOperation,
    pub input: Option<NodeId>,
    pub arguments: Vec<NodeId>,
}

impl Node {
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

    pub fn evaluate(&self, engine: &Engine, arguments: &HashMap<ParameterId, Value>) -> Value {
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
            NodeOperation::ComposeStruct => {
                let name = engine[self.arguments[0]]
                    .evaluate(engine, arguments)
                    .as_string()
                    .unwrap()
                    .clone();
                let mut components = Vec::new();
                for (component_name, component_value) in self.arguments[1..].iter().tuples() {
                    let component_name = engine[*component_name]
                        .evaluate(engine, arguments)
                        .as_string()
                        .unwrap()
                        .clone();
                    let component_value = engine[*component_value].evaluate(engine, arguments);
                    components.push((component_name, component_value));
                }
                Value::Struct { name, components }
            }
            NodeOperation::ComposeColor => {
                let c1 = engine[self.arguments[0]].evaluate(engine, arguments);
                let c2 = engine[self.arguments[1]].evaluate(engine, arguments);
                let c3 = engine[self.arguments[2]].evaluate(engine, arguments);
                Value::Struct {
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
                if let Value::Struct { components, .. } = input {
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

    pub fn compile(
        &self,
        engine: &Engine,
    ) -> (
        HashMap<ParameterId, MemoryLayout>,
        BytecodeProgram,
        MemoryLayout,
    ) {
        let mut temporary_heap = Heap::new(0);
        let mut arg_layout = HashMap::new();
        let parameters = self.collect_parameters(engine);
        for param in parameters {
            let size = param.default.size();
            let start = temporary_heap.allocate_space_for_multiple_values(size);
            arg_layout.insert(param.id, param.default.layout(start));
        }
        let instructions = vec![];
        let output_layout =
            self.compile_impl(engine, &arg_layout, &mut instructions, &mut temporary_heap);
        (
            arg_layout,
            BytecodeProgram::new(instructions),
            output_layout,
        )
    }

    fn compile_impl(
        &self,
        engine: &Engine,
        arg_layout: &HashMap<Id<Parameter>, MemoryLayout>,
        instructions: &mut Vec<BytecodeInstruction>,
        heap: &mut Heap,
    ) -> MemoryLayout {
        let output_layout = match self.operation {
            NodeOperation::Literal(value) => value.add_load_instructions(instructions, heap),
            NodeOperation::Parameter(id) => arg_layout[&id].clone(),
            NodeOperation::Basic(op) => {
                let input = engine[self.input.unwrap()].compile_impl(
                    engine,
                    arg_layout,
                    instructions,
                    heap,
                );
                let argument =
                    engine[self.arguments[0]].compile_impl(engine, arg_layout, instructions, heap);
                match (input, argument) {
                    (MemoryLayout::Integer(a), MemoryLayout::Integer(b)) => {
                        op.compile_int(a, b, instructions, heap)
                    }
                    (MemoryLayout::Integer(a), MemoryLayout::Float(b)) => {
                        let a_cast = cast_int_to_float(a, instructions, heap);
                        op.compile_float(a_cast, b, instructions, heap)
                    }
                    (MemoryLayout::Float(a), MemoryLayout::Integer(b)) => {
                        let b_cast = cast_int_to_float(b, instructions, heap);
                        op.compile_float(a, b_cast, instructions, heap)
                    }
                    (MemoryLayout::Float(a), MemoryLayout::Float(b)) => {
                        op.compile_float(a, b, instructions, heap)
                    }
                    _ => unreachable!("Unsupported operation"),
                }
            }
            NodeOperation::ComposeStruct => {
                let mut components = Vec::new();
                for (&component_name, &component_value) in (&self.arguments[1..]).iter().tuples() {
                    let component_name = engine[component_name]
                        .evaluate(engine, &HashMap::new())
                        .as_string()
                        .unwrap()
                        .clone();
                    let component_value = engine[component_value].compile_impl(
                        engine,
                        arg_layout,
                        instructions,
                        heap,
                    );
                    components.push((component_name, component_value));
                }
                MemoryLayout::Struct { components }
            }
            NodeOperation::ComposeColor => todo!(),
            NodeOperation::GetComponent(name) => {
                let base = engine[self.input.unwrap()].compile_impl(
                    engine,
                    arg_layout,
                    instructions,
                    heap,
                );
                let MemoryLayout::Struct { components } = base else { panic!() };
                components.iter().find(|x| x.0 == name).unwrap().1.clone()
            }
            NodeOperation::CustomNode { result, input } => {
                let mut parameters = engine[result].collect_parameters(engine);
                let mut new_args = HashMap::new();
                let mut arg_index = 0;
                for param in parameters {
                    let value = if Some(param.id) == input {
                        self.input.unwrap()
                    } else {
                        let value = self.arguments[arg_index];
                        arg_index += 1;
                        value
                    };
                    let value = engine[value].compile_impl(engine, arg_layout, instructions, heap);
                    new_args.insert(param.id, value);
                }
                engine[result].compile_impl(engine, &new_args, instructions, heap)
            }
        };
        output_layout
    }

    pub fn as_literal(&self) -> &Value {
        if let NodeOperation::Literal(literal) = &self.operation {
            literal
        } else {
            panic!("Not a literal.")
        }
    }

    pub fn as_literal_mut(&mut self) -> &mut Value {
        if let NodeOperation::Literal(literal) = &mut self.operation {
            literal
        } else {
            panic!("Not a literal.")
        }
    }

    pub fn output_type(&self, type_of_other_node: impl FnOnce(NodeId) -> Type) -> Type {
        use NodeOperation::*;
        match &self.operation {
            Literal(lit) => lit.r#type().into(),
            Parameter(..) => type_of_other_node(self.input.unwrap()),
            Basic(..) => type_of_other_node(self.input.unwrap()),
            ComposeStruct => todo!(),
            ComposeColor => todo!(),
            GetComponent(_) => todo!(),
            &CustomNode { result, .. } => type_of_other_node(result),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeOperation {
    Literal(Value),
    Parameter(ParameterId),
    Basic(BasicOp),
    ComposeStruct,
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
            ComposeStruct => format!("Compose Struct"),
            ComposeColor => format!("Compose Color"),
            GetComponent(component_name) => format!("{} Component", component_name),
            CustomNode { .. } => format!("This Name Shouldn't Show Up"),
        }
    }

    pub fn param_name<'a>(&self, index: usize, parameters: &'a [ParameterDescription]) -> &'a str {
        use NodeOperation::*;
        match self {
            ComposeStruct => todo!(),
            CustomNode { input, .. } => {
                &parameters
                    .iter()
                    .filter(|p| *input != Some(p.id))
                    .nth(index)
                    .unwrap()
                    .name
            }
            _ => {
                let names = self.param_names();
                names[index.min(names.len() - 1)]
            }
        }
    }

    fn param_names(&self) -> &'static [&'static str] {
        use NodeOperation::*;
        match self {
            Literal(..) => &[],
            Parameter(..) => &["Name"],
            Basic(op) => op.param_names(),
            ComposeStruct => &["This Label Shouldn't Show Up"],
            ComposeColor => &["Channel 1", "Channel 2", "Channel 3"],
            GetComponent(..) => &[],
            CustomNode { .. } => &["This Label Shouldn't Show Up"],
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

    fn combine(&self, a: &Value, b: &Value) -> Value {
        let supertype = a.r#type().max(b.r#type());
        let a = a.cast(supertype).unwrap();
        let b = b.cast(supertype).unwrap();
        match (a, b) {
            (Value::Boolean(a), Value::Boolean(b)) => self.combine_booleans(a, b).into(),
            (Value::Integer(a), Value::Integer(b)) => self.combine_integers(a, b).into(),
            (Value::Float(a), Value::Float(b)) => self.combine_floats(a, b).into(),
            (Value::String(a), Value::String(b)) => self.combine_strings(a, b).into(),
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

    fn compile_int(
        &self,
        input_1: usize,
        input_2: usize,
        instructions: &mut Vec<BytecodeInstruction>,
        heap: &mut Heap,
    ) -> MemoryLayout {
        let output = heap.allocate_space_for_single_value();
        let op = match self {
            BasicOp::Add => IntegerOp::Add,
            BasicOp::Subtract => IntegerOp::Subtract,
            BasicOp::Multiply => IntegerOp::Multiply,
            BasicOp::Divide => IntegerOp::Divide,
        };
        instructions.push(BytecodeInstruction::BinaryOp {
            op: BinaryOp::IntegerOp(op),
            input_1,
            input_2,
            output,
        });
        MemoryLayout::Integer(output)
    }

    fn compile_float(
        &self,
        input_1: usize,
        input_2: usize,
        instructions: &mut Vec<BytecodeInstruction>,
        heap: &mut Heap,
    ) -> MemoryLayout {
        let output = heap.allocate_space_for_single_value();
        let op = match self {
            BasicOp::Add => FloatOp::Add,
            BasicOp::Subtract => FloatOp::Subtract,
            BasicOp::Multiply => FloatOp::Multiply,
            BasicOp::Divide => FloatOp::Divide,
        };
        instructions.push(BytecodeInstruction::BinaryOp {
            op: BinaryOp::FloatOp(op),
            input_1,
            input_2,
            output,
        });
        MemoryLayout::Integer(output)
    }
}

fn cast_int_to_float(
    int: usize,
    instructions: &mut Vec<BytecodeInstruction>,
    heap: &mut Heap,
) -> usize {
    let cast = heap.allocate_space_for_single_value();
    instructions.push(BytecodeInstruction::UnaryOp {
        op: UnaryOp::CastIntToFloat,
        input: a,
        output: a_cast,
    });
    cast
}
