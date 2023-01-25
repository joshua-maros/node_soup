use itertools::Itertools;

use crate::{util, visuals};

pub struct Engine {
    root: Node,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            root: Node {
                operation: NodeOperation::Builtin(BuiltinOperation::Sum),
                arguments: vec![
                    Node {
                        operation: NodeOperation::FloatLiteral(1e-6),
                        arguments: vec![],
                    },
                    Node {
                        operation: NodeOperation::FloatLiteral(1e+6),
                        arguments: vec![],
                    }
                ],
            },
        }
    }

    pub fn root_node(&self) -> &Node {
        &self.root
    }
}

pub struct Node {
    operation: NodeOperation,
    arguments: Vec<Node>,
}

impl Node {
    pub fn visual(&self) -> visuals::Node {
        visuals::Node {
            name: self.operation.name(),
            sockets: self
                .arguments
                .iter()
                .map(|arg| visuals::Socket {
                    name: format!(""),
                    node: arg.visual(),
                })
                .collect_vec(),
        }
    }
}

pub enum NodeOperation {
    FloatLiteral(f32),
    Builtin(BuiltinOperation),
}

impl NodeOperation {
    pub fn name(&self) -> String {
        use NodeOperation::*;
        match self {
            &FloatLiteral(num) => util::pretty_format_number(num),
            Builtin(op) => op.name().to_owned(),
        }
    }
}

pub enum BuiltinOperation {
    Sum,
    Difference,
    Product,
    Quotient,
}

impl BuiltinOperation {
    pub fn name(&self) -> &'static str {
        use BuiltinOperation::*;
        match self {
            Sum => "Sum",
            Difference => "Difference",
            Product => "Product",
            Quotient => "Quotient",
        }
    }
}
