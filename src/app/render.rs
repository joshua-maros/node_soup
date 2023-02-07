use std::collections::HashMap;

use wgpu::SurfaceError;
use wgpu_glyph::{HorizontalAlign, VerticalAlign};
use winit::event_loop::ControlFlow;

use super::App;
use crate::{
    engine::{Node, NodeId, NodeOperation, Parameter, ParameterDescription, ParameterId, Value},
    renderer::{
        Position, RectInstance, Section, Shapes, Size, Text, BOTTOM_OUTLINE_FLAT,
        LEFT_OUTLINE_ANTIDIAGONAL, LEFT_OUTLINE_DIAGONAL, LEFT_OUTLINE_FLAT,
        RIGHT_OUTLINE_ANTIDIAGONAL, RIGHT_OUTLINE_DIAGONAL, RIGHT_OUTLINE_FLAT, TOP_OUTLINE_FLAT,
    },
    theme::{
        NODE_FILL, NODE_GUTTER_WIDTH, NODE_HEADER_HEIGHT, NODE_INNER_CORNER_SIZE, NODE_MIN_WIDTH,
        NODE_OUTER_CORNER_SIZE, NODE_OUTLINE, NODE_PADDING,
    },
    widgets::{BoundingBox, BoundingBoxKind},
};

impl App {
    pub(super) fn render(&mut self) {
        let mut base_layer = Shapes::new();
        self.root_bbox = BoundingBox::new_from_children(vec![
            self.render_root_node(&mut base_layer),
            self.render_preview_drawer(&mut base_layer),
        ]);

        let layers = [&base_layer];
        let result = self.render_engine.render(&layers);
        match result {
            Ok(()) => (),
            Err(SurfaceError::Lost) | Err(SurfaceError::Outdated) => {
                self.render_engine.refresh_target()
            }
            Err(SurfaceError::OutOfMemory) => self.control_flow = ControlFlow::ExitWithCode(1),
            Err(e) => eprintln!("{:#?}", e),
        }
    }

    fn render_root_node(&mut self, layer: &mut Shapes) -> BoundingBox {
        let pos = Position {
            x: self.preview_drawer_size,
            y: 0.0,
        };
        self.render_node_stack(pos, layer, self.computation_engine.root_node(), "Root")
    }

    fn render_preview_drawer(&mut self, layer: &mut Shapes) -> BoundingBox {
        let mut y = 0.0;
        let mut bboxes = Vec::new();
        let root = self.computation_engine.root_node();
        let root = &self.computation_engine[root];
        let mut parameters = HashMap::new();
        for param_desc in root.collect_parameters(&self.computation_engine) {
            parameters.insert(param_desc.id, param_desc.default.clone());
        }
        let value = root.evaluate(&self.computation_engine, &parameters);
        let bbox = render_output_preview(Position { x: 0.0, y }, layer, format!("Root"), &value);
        bboxes.push(bbox);
        BoundingBox::new_from_children(bboxes)
    }

    fn render_node_stack(
        &self,
        start: Position,
        layer: &mut Shapes,
        node_id: NodeId,
        output_label: &str,
    ) -> BoundingBox {
        let node_bbox = self.render_node(start, layer, node_id);
        let x = start.x;
        let y = node_bbox.end.y + NODE_PADDING;
        layer.push_rect(RectInstance {
            position: [x, y],
            size: [NODE_MIN_WIDTH, NODE_HEADER_HEIGHT],
            fill_color: NODE_FILL,
            outline_color: NODE_OUTLINE,
            outline_modes: TOP_OUTLINE_FLAT
                | BOTTOM_OUTLINE_FLAT
                | LEFT_OUTLINE_FLAT
                | RIGHT_OUTLINE_FLAT,
        });
        layer.push_text(Text {
            sections: vec![Section::node_label(output_label.to_owned())],
            center: [x + NODE_MIN_WIDTH / 2.0, y + NODE_HEADER_HEIGHT / 2.0],
            bounds: [NODE_MIN_WIDTH, NODE_HEADER_HEIGHT],
            horizontal_align: HorizontalAlign::Center,
            vertical_align: VerticalAlign::Center,
        });
        node_bbox
    }

    fn render_node(&self, start: Position, layer: &mut Shapes, node_id: NodeId) -> BoundingBox {
        let node = &self.computation_engine[node_id];
        let Position { x, y } = start;
        let mut label = Text {
            sections: vec![Section::node_label(node.operation.name())],
            center: [x + NODE_PADDING, y + NODE_HEADER_HEIGHT / 2.0],
            bounds: [NODE_MIN_WIDTH, NODE_HEADER_HEIGHT],
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Center,
        };
        let mut y = y;
        let mut bboxes = Vec::new();
        if let Some(input) = node.input {
            let bbox = self.render_node(start, layer, input);
            y = bbox.end.y + NODE_PADDING;
            bboxes.push(bbox);
        }
        if let NodeOperation::ReuseNode(..) = node.operation {
            for parameter in node
                .collect_parameters(&self.computation_engine)
                .iter()
                .rev()
            {
                let start = Position {
                    x: x + NODE_GUTTER_WIDTH + NODE_PADDING,
                    y,
                };
                let param_bbox = render_parameter(start, layer, &parameter.name);
                y = param_bbox.end.y + NODE_PADDING;
                bboxes.push(param_bbox);
            }
        } else {
            let param_names = node.operation.param_names();
            for (index, _) in node.arguments.iter().enumerate().rev() {
                let start = Position {
                    x: x + NODE_GUTTER_WIDTH + NODE_PADDING,
                    y,
                };
                let label = param_names[index.min(param_names.len() - 1)];
                let param_bbox = render_parameter(start, layer, label);
                y = param_bbox.end.y + NODE_PADDING;
                bboxes.push(param_bbox);
            }
        }
        layer.push_rect(RectInstance {
            position: [x, y],
            size: [NODE_MIN_WIDTH, NODE_HEADER_HEIGHT],
            fill_color: NODE_FILL,
            outline_color: NODE_OUTLINE,
            outline_modes: LEFT_OUTLINE_FLAT
                | RIGHT_OUTLINE_FLAT
                | TOP_OUTLINE_FLAT
                | BOTTOM_OUTLINE_FLAT,
        });
        label.center[1] = y + NODE_HEADER_HEIGHT / 2.0;
        let end = Position {
            x: x + NODE_MIN_WIDTH,
            y: y + NODE_HEADER_HEIGHT,
        };
        bboxes.push(BoundingBox::new_start_end(
            Position { x, y },
            end,
            BoundingBoxKind::InvokeTool(self.builtins.adjust_float_tool, node_id),
        ));
        layer.push_text(label);
        BoundingBox::new_from_children(bboxes)
    }
}

fn render_parameter(start: Position, layer: &mut Shapes, name: &str) -> BoundingBox {
    let width = NODE_MIN_WIDTH - NODE_PADDING - NODE_GUTTER_WIDTH;
    let height = NODE_HEADER_HEIGHT;
    let kind = BoundingBoxKind::Unused;
    layer.push_rect(RectInstance {
        position: [start.x, start.y],
        size: [width, height],
        fill_color: NODE_FILL,
        outline_color: NODE_OUTLINE,
        outline_modes: LEFT_OUTLINE_FLAT
            | RIGHT_OUTLINE_FLAT
            | TOP_OUTLINE_FLAT
            | BOTTOM_OUTLINE_FLAT,
    });
    layer.push_text(Text {
        sections: vec![Section::node_label(name.to_owned())],
        center: [start.x + NODE_PADDING, start.y + height / 2.0],
        bounds: [width - 2.0 * NODE_PADDING, height],
        horizontal_align: HorizontalAlign::Left,
        vertical_align: VerticalAlign::Center,
    });
    BoundingBox::new_start_size(start, Size { width, height }, kind)
}

fn render_output_preview(
    start: Position,
    layer: &mut Shapes,
    label: String,
    value: &Value,
) -> BoundingBox {
    let bbox_kind = BoundingBoxKind::Unused;
    render_value_preview_helper(start, layer, label, value, bbox_kind)
}

fn render_value_preview_helper(
    start: Position,
    layer: &mut Shapes,
    label: String,
    value: &Value,
    bbox_kind: BoundingBoxKind,
) -> BoundingBox {
    let size = Size {
        width: NODE_MIN_WIDTH,
        height: NODE_HEADER_HEIGHT,
    };
    layer.push_rect(RectInstance {
        position: [start.x, start.y],
        size: [size.width, size.height],
        fill_color: NODE_FILL,
        outline_color: NODE_OUTLINE,
        outline_modes: TOP_OUTLINE_FLAT
            | BOTTOM_OUTLINE_FLAT
            | LEFT_OUTLINE_FLAT
            | RIGHT_OUTLINE_FLAT,
    });
    layer.push_text(Text {
        sections: vec![
            Section::node_label(label.clone()),
            Section::node_label(value.display()),
        ],
        center: [start.x + NODE_PADDING, start.y + size.height / 2.0],
        bounds: [size.width, size.height],
        horizontal_align: HorizontalAlign::Left,
        vertical_align: VerticalAlign::Center,
    });
    BoundingBox::new_start_size(start, size, bbox_kind)
}
