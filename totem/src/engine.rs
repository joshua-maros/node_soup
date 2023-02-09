use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

use itertools::Itertools;
use maplit::hashmap;
use renderer::{Position, Shapes};

use crate::{
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
        components: HashMap<String, Value>,
    },
    // Division by zero, index out of bounds, etc.
    Invalid,
}

impl Value {
    pub fn display(&self) -> String {
        if let &Value::Float(value) = self {
            util::pretty_format_number(value)
        } else {
            self.cast(BuiltinType::String)
                .unwrap()
                .as_string()
                .unwrap()
                .clone()
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
    pub fn r#type(&self) -> BuiltinType {
        use BuiltinType::*;
        match self {
            Self::Boolean(..) => Boolean,
            Self::Integer(..) => Integer,
            Self::Float(..) => Float,
            Self::String(..) => String,
            _ => todo!("{:#?}", self),
        }
    }

    pub fn cast(&self, to: BuiltinType) -> Option<Self> {
        use BuiltinType::*;
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
                Type => None,
            },
            &Self::Float(value) => match to {
                Boolean => Some((value != 0.0).into()),
                Integer => Some((value as i32).into()),
                Float => Some(value.into()),
                String => Some(format!("{}", value).into()),
                Type => None,
            },
            Self::String(value) => match to {
                String => Some(value.clone().into()),
                Type => None,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    Boolean,
    Integer,
    Float,
    String,
    Type,
}

impl BuiltinType {
    pub fn name(self) -> &'static str {
        use BuiltinType::*;
        match self {
            Boolean => "Boolean",
            Integer => "Integer",
            Float => "Float",
            String => "String",
            Type => "Type",
        }
    }

    fn rank(self) -> i32 {
        match self {
            BuiltinType::Boolean => 0,
            BuiltinType::Integer => 1,
            BuiltinType::Float => 2,
            BuiltinType::String => 3,
            BuiltinType::Type => 4,
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

pub type NodeId = Id<Node>;
pub type ParameterId = Id<Parameter>;

pub struct Engine {
    nodes: HashMap<NodeId, Node>,
    root_node: NodeId,
    tools: HashMap<ToolId, Tool>,
    node_ids: IdCreator<Node>,
    parameter_ids: IdCreator<Parameter>,
    tool_ids: IdCreator<Tool>,
    pub input_parameter_for_reused_nodes: ParameterId,
}

pub struct BuiltinDefinitions {
    pub x_component: ParameterId,
    pub y_component: ParameterId,
    pub compose_vector_2d: NodeId,
    pub mouse_offset: (ParameterId, NodeId),
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
        let input_parameter_for_reused_nodes = parameter_ids.next();
        let tool_ids = IdCreator::new();
        let mut this = Self {
            nodes: hashmap! [root_node => start_node],
            tools: hashmap![],
            root_node,
            node_ids,
            parameter_ids,
            tool_ids,
            input_parameter_for_reused_nodes,
        };
        let builtins = this.make_builtins();
        this.setup_demo(&builtins);
        (this, builtins)
    }

    fn make_builtins(&mut self) -> BuiltinDefinitions {
        let default_struct = Value::Struct {
            name: "Empty Struct".to_owned(),
            components: hashmap![],
        };
        let (compose_vector_2d, vector_2d_parameters) = self.push_simple_struct_composer(
            "Compose Vector/2D",
            vec![("X", 0.0.into()), ("Y", 0.0.into())],
        );
        let zero = self.push_literal_node(0.0.into());
        let default_vec2 = self.push_node(Node {
            operation: NodeOperation::ReuseNode(compose_vector_2d),
            input: None,
            arguments: vec![zero, zero],
        });
        let name = self.push_literal_node("Mouse Offset".to_owned().into());
        let mouse_offset = self.push_parameter(name, default_vec2);

        let (prototype, target) = {
            let target = self.push_simple_parameter("SPECIAL TOOL TARGET Factor", 1.0.into());
            let input = self.push_simple_parameter("SPECIAL TOOL WILDCARD", 0.0.into());
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
            mouse_offset,
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
        let value = self.root_node();
        // let param1 = self.push_simple_parameter("Value", 2.0.into());
        let param1 = self.push_literal_node(2.0.into());
        let value = self.push_node(Node {
            operation: NodeOperation::Basic(BasicOp::Multiply),
            input: Some(value),
            arguments: vec![param1],
        });
        let param2 = self.push_simple_parameter("Value", 123.0.into());
        let root = self.push_node(Node {
            operation: NodeOperation::Basic(BasicOp::Add),
            input: Some(value),
            arguments: vec![param2],
        });

        // let make_state = self.push_simple_struct_composer(
        //     "Compose Adjust Float Tool Scope",
        //     vec![("target", 0.0.into()), ("state", default_struct.clone())],
        // );

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

    pub fn push_simple_input(&mut self, default_value: Value) -> NodeId {
        let param_default = self.push_literal_node(default_value);
        self.push_input(param_default)
    }

    pub fn push_input(&mut self, default_value: NodeId) -> NodeId {
        self.push_node(Node {
            operation: NodeOperation::InputParameter,
            input: Some(default_value),
            arguments: vec![],
        })
    }

    pub fn set_root(&mut self, node: NodeId) {
        self.root_node = node;
        let mut root_parameters = Vec::new();
        self[self.root_node].collect_parameters_into(self, &mut root_parameters);
    }

    pub fn root_node(&self) -> NodeId {
        self.root_node
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
            NodeOperation::InputParameter => arguments
                .get(&engine.input_parameter_for_reused_nodes)
                .cloned()
                .unwrap(),
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
                let mut components = HashMap::new();
                for (component_name, component_value) in self.arguments[1..].iter().tuples() {
                    let component_name = engine[*component_name]
                        .evaluate(engine, arguments)
                        .as_string()
                        .unwrap()
                        .clone();
                    let component_value = engine[*component_value].evaluate(engine, arguments);
                    components.insert(component_name, component_value);
                }
                Value::Struct { name, components }
            }
            NodeOperation::ComposeColor => {
                let c1 = engine[self.arguments[0]].evaluate(engine, arguments);
                let c2 = engine[self.arguments[1]].evaluate(engine, arguments);
                let c3 = engine[self.arguments[2]].evaluate(engine, arguments);
                Value::Struct {
                    name: format!("Color"),
                    components: hashmap! [
                        format!("Channel 1") => c1,
                        format!("Channel 2") => c2,
                        format!("Channel 3") => c3,
                    ],
                }
            }
            NodeOperation::GetComponent(component_name) => {
                let input = engine[self.input.unwrap()].evaluate(engine, arguments);
                if let Value::Struct { components, .. } = input {
                    components[component_name].clone()
                } else {
                    panic!("Cannot extract components from a non-struct value!")
                }
            }
            NodeOperation::ReuseNode(node) => {
                let node = &engine[*node];
                let mut next_arguments = HashMap::new();
                if let Some(input) = self.input {
                    let value = engine[input].evaluate(engine, arguments);
                    next_arguments.insert(engine.input_parameter_for_reused_nodes, value);
                }
                let params = node.collect_parameters(engine);
                for (index, param) in params.into_iter().enumerate() {
                    let arg = self.arguments[index];
                    let arg = engine[arg].evaluate(engine, arguments);
                    next_arguments.insert(param.id, arg);
                }
                node.evaluate(engine, &next_arguments)
            }
        }
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeOperation {
    Literal(Value),
    Parameter(ParameterId),
    InputParameter,
    Basic(BasicOp),
    ComposeStruct,
    ComposeColor,
    GetComponent(String),
    ReuseNode(NodeId),
}

impl NodeOperation {
    pub fn name(&self) -> String {
        use NodeOperation::*;
        match self {
            Literal(value) => value.display(),
            Parameter(..) => format!("Parameter"),
            InputParameter => format!("Input"),
            Basic(op) => op.name().to_owned(),
            ComposeStruct => format!("Compose Struct"),
            ComposeColor => format!("Compose Color"),
            GetComponent(component_name) => format!("{} Component", component_name),
            ReuseNode(..) => format!("This Name Shouldn't Show Up"),
        }
    }

    pub fn param_name<'a>(&self, index: usize, parameters: &'a [ParameterDescription]) -> &'a str {
        use NodeOperation::*;
        match self {
            ComposeStruct => todo!(),
            ReuseNode(..) => &parameters[index].name,
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
            InputParameter => &[],
            Basic(op) => op.param_names(),
            ComposeStruct => &["This Label Shouldn't Show Up"],
            ComposeColor => &["Channel 1", "Channel 2", "Channel 3"],
            GetComponent(..) => &[],
            ReuseNode(..) => &["This Label Shouldn't Show Up"],
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
}
