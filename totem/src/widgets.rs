use wgpu_glyph::{HorizontalAlign, VerticalAlign};

use crate::{
    engine::{NodeId, ParameterId, ToolId, Value},
    renderer::{
        Position, RectInstance, Section, Shapes, Size, Text, BOTTOM_OUTLINE_FLAT,
        LEFT_OUTLINE_ANTIDIAGONAL, LEFT_OUTLINE_DIAGONAL, LEFT_OUTLINE_FLAT,
        RIGHT_OUTLINE_ANTIDIAGONAL, RIGHT_OUTLINE_DIAGONAL, RIGHT_OUTLINE_FLAT, TOP_OUTLINE_FLAT,
    },
    theme::{
        NODE_CORNER_SIZE, NODE_FILL, NODE_GUTTER_WIDTH, NODE_HEIGHT, NODE_OUTLINE,
        NODE_PARAMETER_PADDING, NODE_WIDTH,
    },
};

#[derive(Clone, Debug)]
pub enum BoundingBoxKind {
    InvokeTool(ToolId),
    Parent(Vec<BoundingBox>),
    SelectNode(usize, NodeId),
    Unused,
}

#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub start: Position,
    pub end: Position,
    pub kind: BoundingBoxKind,
}

impl BoundingBox {
    pub fn new_start_end(start: Position, end: Position, kind: BoundingBoxKind) -> Self {
        assert!(start.x <= end.x);
        assert!(start.y <= end.y);
        Self { start, end, kind }
    }

    pub fn new_start_size(start: Position, size: Size, kind: BoundingBoxKind) -> Self {
        assert!(!size.is_negative());
        Self::new_start_end(start, start + size, kind)
    }

    pub fn new_from_children(children: Vec<BoundingBox>) -> Self {
        assert!(children.len() > 0);
        let mut min = children[0].start;
        let mut max = children[0].end;
        for child in &children[1..] {
            min = min.componentwise_min(child.start);
            max = max.componentwise_max(child.end);
        }
        Self::new_start_end(min, max, BoundingBoxKind::Parent(children))
    }

    pub fn size(&self) -> Size {
        self.end - self.start
    }

    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= self.start.x && pos.y >= self.start.y && pos.x <= self.end.x && pos.y <= self.end.y
    }
}

pub struct Socket {
    pub node: Node,
    pub name: String,
}

impl Socket {
    pub fn new(node: Node, name: String) -> Self {
        Self { node, name }
    }

    pub fn size(&self) -> Size {
        self.node.size()
    }
}

pub struct Node {
    pub name: String,
    pub sockets: Vec<Socket>,
}

impl Node {
    pub fn size(&self) -> Size {
        if self.sockets.len() == 0 {
            Size {
                width: NODE_WIDTH,
                height: NODE_HEIGHT,
            }
        } else {
            let socket_child_sizes = self.sockets.iter().map(Socket::size);
            let size_from_children = socket_child_sizes.fold(Size::zero(), |prev, next| Size {
                width: prev.width.max(next.width),
                height: prev.height + next.height,
            });
            Size {
                width: size_from_children.width + NODE_PARAMETER_PADDING + NODE_GUTTER_WIDTH,
                height: size_from_children.height
                    + (self.sockets.len() as f32 + 0.5) * NODE_PARAMETER_PADDING
                    + NODE_HEIGHT,
            }
        }
    }

    pub fn draw(&self, start: Position, containing_socket_name: &str, layer: &mut Shapes) {}
}

pub struct SimpleValueWidget {
    pub label: String,
    pub value: Value,
}

pub struct EventResponse {
    pub request_focus: bool,
    pub new_value: Option<Value>,
}

impl Default for EventResponse {
    fn default() -> Self {
        Self {
            request_focus: false,
            new_value: None,
        }
    }
}

pub trait ValueWidget {
    fn size(&self) -> Size;
    fn draw(&self, start: Position, layer: &mut Shapes);
    fn on_click(&mut self, er: &mut EventResponse) {}
    fn on_drag_start(&mut self, er: &mut EventResponse) {}
    fn on_drag(&mut self, er: &mut EventResponse, offset: (f32, f32)) {}
    fn on_drag_end(&mut self, er: &mut EventResponse) {}
}

impl ValueWidget for SimpleValueWidget {
    fn size(&self) -> Size {
        Size {
            width: NODE_WIDTH,
            height: NODE_HEIGHT,
        }
    }

    fn draw(&self, start: Position, layer: &mut Shapes) {}

    fn on_drag(&mut self, er: &mut EventResponse, offset: (f32, f32)) {}
}
