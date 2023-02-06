use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

use itertools::Itertools;
use maplit::hashmap;
use wgpu::PrimitiveState;

use crate::{
    renderer::{Position, Shapes},
    util::{self, Id, IdCreator},
    visuals::{self, SimpleValueWidget},
};

pub struct Parameter {
    pub id: ParameterId,
    pub name: String,
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

impl Value {
    pub fn visual(&self, label: String) -> SimpleValueWidget {
        SimpleValueWidget {
            label,
            value: self.clone(),
        }
    }

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
    parameter_nodes: HashMap<ParameterId, NodeId>,
    parameter_preview_values: HashMap<ParameterId, Value>,
    root_node: NodeId,
    root_parameters: Vec<Parameter>,
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
            root_parameters: vec![],
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
        let param_default = self.push_node(Node {
            operation: NodeOperation::PrimitiveLiteral(123.0.into()),
            arguments: vec![],
        });
        let (_, param) = self.push_parameter(param_name, param_type, param_default);
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

    pub fn push_parameter(
        &mut self,
        name: NodeId,
        r#type: NodeId,
        default_value: NodeId,
    ) -> (ParameterId, NodeId) {
        let id = self.parameter_ids.next();
        let node = Node {
            operation: NodeOperation::Parameter(id),
            arguments: vec![name, r#type, default_value],
        };
        let node_id = self.push_node(node);
        self.parameter_nodes.insert(id, node_id);
        let default = self[default_value].evaluate(self, &HashMap::new());
        self.parameter_preview_values.insert(id, default);
        (id, node_id)
    }

    pub fn set_root(&mut self, node: NodeId) {
        self.root_node = node;
        let mut root_parameters = Vec::new();
        self[self.root_node].collect_parameters(self, &mut root_parameters);
        self.root_parameters = root_parameters;
    }

    pub fn root_parameters(&self) -> &[Parameter] {
        &self.root_parameters
    }

    pub fn parameter_preview(&self, index: usize) -> &Value {
        self.parameter_preview_values
            .get(&self.root_parameters[index].id)
            .unwrap()
    }

    pub fn parameter_preview_mut(&mut self, index: usize) -> &mut Value {
        self.parameter_preview_values
            .get_mut(&self.root_parameters[index].id)
            .unwrap()
    }

    pub fn evaluate_root_result_preview(&self) -> Value {
        let mut parameters = hashmap![];
        for param in &self.root_parameters {
            parameters.insert(param.id, self.parameter_preview_values[&param.id].clone());
        }
        self[self.root_node].evaluate(self, &parameters)
    }

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
                name: engine[self.arguments[0]]
                    .evaluate(engine, &HashMap::new())
                    .as_string()
                    .unwrap()
                    .clone(),
                r#type: engine[self.arguments[1]]
                    .evaluate(engine, &HashMap::new())
                    .as_type(),
            });
        } else {
            for &arg in &self.arguments {
                engine[arg].collect_parameters(engine, into);
            }
        }
    }

    pub fn evaluate(&self, engine: &Engine, parameters: &HashMap<ParameterId, Value>) -> Value {
        match &self.operation {
            NodeOperation::PrimitiveLiteral(value) => value.clone(),
            &NodeOperation::BuiltinTypeLiteral(value) => Value::Type(value),
            NodeOperation::Parameter(id) => parameters.get(id).unwrap_or(&Value::Free).clone(),
            NodeOperation::Combination(op) => {
                let mut value = engine[self.arguments[0]].evaluate(engine, parameters);
                for &arg in &self.arguments[1..] {
                    let next = engine[arg].evaluate(engine, parameters);
                    value = op.combine(&value, &next);
                }
                value
            }
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
            PrimitiveLiteral(value) => value.display(),
            &BuiltinTypeLiteral(r#type) => r#type.name().to_owned(),
            Parameter(..) => format!("Parameter"),
            Combination(op) => op.name().to_owned(),
        }
    }

    fn arg_names(&self) -> &'static [&'static str] {
        use NodeOperation::*;
        match self {
            Combination(op) => op.arg_names(),
            Parameter(..) => &["Name", "Type", "Default Value"],
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

    fn combine(&self, a: &Value, b: &Value) -> Value {
        let supertype = a.r#type().max(b.r#type());
        let a = a.cast(supertype).unwrap();
        let b = b.cast(supertype).unwrap();
        match (a, b) {
            (Value::Boolean(a), Value::Boolean(b)) => self.combine_booleans(a, b).into(),
            (Value::Integer(a), Value::Integer(b)) => self.combine_integers(a, b).into(),
            (Value::Float(a), Value::Float(b)) => self.combine_floats(a, b).into(),
            (Value::String(a), Value::String(b)) => self.combine_strings(a, b).into(),
            (Value::Type(a), Value::Type(b)) => panic!("Cannot perform arithmetic on types."),
            _ => unreachable!("Values should be the same type."),
        }
    }

    fn combine_booleans(&self, a: bool, b: bool) -> bool {
        use CombinationOperation::*;
        match self {
            Sum => a || b,
            Difference => a && !b,
            Product => a && b,
            _ => panic!("Invalid boolean operation"),
        }
    }

    fn combine_integers(&self, a: i32, b: i32) -> i32 {
        use CombinationOperation::*;
        match self {
            Sum => a + b,
            Difference => a - b,
            Product => a * b,
            Quotient => a / b,
        }
    }

    fn combine_floats(&self, a: f32, b: f32) -> f32 {
        use CombinationOperation::*;
        match self {
            Sum => a + b,
            Difference => a - b,
            Product => a * b,
            Quotient => a / b,
        }
    }

    fn combine_strings(&self, a: String, b: String) -> String {
        use CombinationOperation::*;
        match self {
            Sum => format!("{}{}", a, b),
            Product => format!("{}{}", a, b),
            _ => panic!("Invalid string operation"),
        }
    }
}
