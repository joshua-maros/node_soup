use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

use itertools::Itertools;
use maplit::hashmap;
use wgpu::PrimitiveState;

use crate::{
    util::{self, Id, IdCreator},
    visuals,
};

pub struct Parameter {
    pub id: ParameterId,
    pub r#type: Option<BuiltinType>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    String(String),
    Type(BuiltinType),
    // Division by zero, index out of bounds, etc.
    Invalid,
    // Depends on a parameter for which a value was not provided.
    Free,
}

impl From<BuiltinType> for Value {
    fn from(v: BuiltinType) -> Self {
        Self::Type(v)
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
    pub fn as_type(&self) -> Option<BuiltinType> {
        if let &Self::Type(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn r#type(&self) -> BuiltinType {
        use BuiltinType::*;
        match self {
            Self::Boolean(..) => Boolean,
            Self::Integer(..) => Integer,
            Self::Float(..) => Float,
            Self::String(..) => String,
            Self::Type(..) => Type,
            _ => todo!(),
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
            &Self::Type(r#type) => match to {
                Type => Some(r#type.into()),
                _ => None,
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

#[derive(Clone, Copy, Debug)]
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
}

pub type NodeId = Id<Node>;
pub type ParameterId = Id<Parameter>;

pub struct Engine {
    nodes: HashMap<NodeId, Node>,
    parameter_nodes: HashMap<ParameterId, NodeId>,
    parameter_preview_values: HashMap<ParameterId, Value>,
    root_node: NodeId,
    node_ids: IdCreator<Node>,
    parameter_ids: IdCreator<Parameter>,
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
    pub fn new() -> Self {
        let start_node = Node {
            operation: NodeOperation::PrimitiveLiteral(1.0.into()),
            arguments: vec![],
        };
        let mut node_ids = IdCreator::new();
        let root_node = node_ids.next();
        let parameter_ids = IdCreator::new();
        let mut this = Self {
            nodes: hashmap! [root_node => start_node],
            parameter_nodes: hashmap![],
            parameter_preview_values: hashmap![],
            root_node,
            node_ids,
            parameter_ids,
        };
        this.setup_demo();
        this
    }

    fn setup_demo(&mut self) {
        let value = self.root_node();
        let param_name = self.push_node(Node {
            operation: NodeOperation::PrimitiveLiteral(format!("Value").into()),
            arguments: vec![],
        });
        let param_type = self.push_node(Node {
            operation: NodeOperation::BuiltinTypeLiteral(BuiltinType::Float),
            arguments: vec![],
        });
        let (_, param) = self.push_parameter(param_name, param_type);
        let root = self.push_node(Node {
            operation: NodeOperation::Combination(CombinationOperation::Sum),
            arguments: vec![param, value],
        });
        self.set_root(root);
    }

    pub fn push_node(&mut self, node: Node) -> NodeId {
        let id = self.node_ids.next();
        self.nodes.insert(id, node);
        id
    }

    pub fn push_parameter(&mut self, name: NodeId, r#type: NodeId) -> (ParameterId, NodeId) {
        let id = self.parameter_ids.next();
        let node = Node {
            operation: NodeOperation::Parameter(id),
            arguments: vec![name, r#type],
        };
        let node_id = self.push_node(node);
        self.parameter_nodes.insert(id, node_id);
        (id, node_id)
    }

    pub fn set_root(&mut self, node: NodeId) {
        self.root_node = node;
    }

    pub fn recalculate_preview_result(&mut self) {}

    pub fn root_node(&self) -> NodeId {
        self.root_node
    }
}

pub struct Node {
    operation: NodeOperation,
    arguments: Vec<NodeId>,
}

impl Node {
    pub fn visual(&self, engine: &Engine) -> visuals::Node {
        let argument_names = self.operation.arg_names();
        visuals::Node {
            name: self.operation.name(),
            sockets: self
                .arguments
                .iter()
                .enumerate()
                .map(|(index, &arg)| visuals::Socket {
                    name: argument_names[index.min(argument_names.len() - 1)].to_owned(),
                    node: engine[arg].visual(engine),
                })
                .rev()
                .collect_vec(),
        }
    }

    pub fn collect_parameters(&self, engine: &Engine, into: &mut Vec<Parameter>) {
        if let &NodeOperation::Parameter(id) = &self.operation {
            into.push(Parameter {
                id,
                r#type: engine[self.arguments[1]]
                    .evaluate(&HashMap::new())
                    .as_type(),
            });
        } else {
        }
    }

    pub fn evaluate(&self, parameters: &HashMap<ParameterId, Value>) -> Value {
        match &self.operation {
            NodeOperation::PrimitiveLiteral(value) => value.clone(),
            &NodeOperation::BuiltinTypeLiteral(value) => Value::Type(value),
            NodeOperation::Parameter(id) => parameters.get(id).unwrap_or(&Value::Free).clone(),
            NodeOperation::Combination(_) => todo!(),
        }
    }
}

pub enum NodeOperation {
    PrimitiveLiteral(Value),
    BuiltinTypeLiteral(BuiltinType),
    Parameter(ParameterId),
    Combination(CombinationOperation),
}

impl NodeOperation {
    pub fn name(&self) -> String {
        use NodeOperation::*;
        match self {
            &PrimitiveLiteral(Value::Float(value)) => util::pretty_format_number(value),
            PrimitiveLiteral(value) => value
                .cast(BuiltinType::String)
                .unwrap()
                .as_string()
                .unwrap()
                .clone(),
            &BuiltinTypeLiteral(r#type) => r#type.name().to_owned(),
            Parameter(..) => format!("Parameter"),
            Combination(op) => op.name().to_owned(),
        }
    }

    fn arg_names(&self) -> &'static [&'static str] {
        use NodeOperation::*;
        match self {
            Combination(op) => op.arg_names(),
            Parameter(..) => &["Name", "Type"],
            _ => &[],
        }
    }
}

pub enum CombinationOperation {
    Sum,
    Difference,
    Product,
    Quotient,
}

impl CombinationOperation {
    pub fn name(&self) -> &'static str {
        use CombinationOperation::*;
        match self {
            Sum => "Sum",
            Difference => "Difference",
            Product => "Product",
            Quotient => "Quotient",
        }
    }

    fn arg_names(&self) -> &'static [&'static str] {
        match self {
            _ => &[""],
        }
    }
}
