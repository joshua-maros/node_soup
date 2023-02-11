use renderer::{Position, Shapes, Size};
use theme::{NODE_LABEL_HEIGHT, NODE_WIDTH};

use crate::engine::{NodeId, ToolId, Value2};

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

pub struct SimpleValueWidget {
    pub label: String,
    pub value: Value2,
}

pub struct EventResponse {
    pub request_focus: bool,
    pub new_value: Option<Value2>,
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
    fn on_click(&mut self, _er: &mut EventResponse) {}
    fn on_drag_start(&mut self, _er: &mut EventResponse) {}
    fn on_drag(&mut self, _er: &mut EventResponse, _offset: (f32, f32)) {}
    fn on_drag_end(&mut self, _er: &mut EventResponse) {}
}

impl ValueWidget for SimpleValueWidget {
    fn size(&self) -> Size {
        Size {
            width: NODE_WIDTH,
            height: NODE_LABEL_HEIGHT,
        }
    }

    fn draw(&self, _start: Position, _layer: &mut Shapes) {}

    fn on_drag(&mut self, _er: &mut EventResponse, _offset: (f32, f32)) {}
}
