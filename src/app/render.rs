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
        FLOAT_TYPE_FILL_COLOR, FLOAT_TYPE_OUTLINE_COLOR, INTER_NODE_PADDING, INTER_PANEL_PADDING,
        NODE_CORNER_SIZE, NODE_FILL, NODE_GUTTER_WIDTH, NODE_HEIGHT, NODE_LABEL_PADDING,
        NODE_OUTLINE, NODE_PARAMETER_PADDING, NODE_WIDTH, VECTOR_TYPE_FILL_COLOR,
        VECTOR_TYPE_OUTLINE_COLOR,
    },
    widgets::{BoundingBox, BoundingBoxKind},
};

impl App {
    pub(super) fn render(&mut self) {
        let mut bboxes = Vec::new();
        let mut base_layer = Shapes::new();
        bboxes.push(self.render_preview_drawer(&mut base_layer));
        let mut x = bboxes[0].end.x;
        let mut editor_nodes = vec![(format!("Root"), self.computation_engine.root_node())];
        while editor_nodes.len() > 0 {
            let (bbox, next_nodes) = self.render_node_editor(
                Position {
                    x: x + INTER_PANEL_PADDING,
                    y: 0.0,
                },
                &mut base_layer,
                editor_nodes,
            );
            editor_nodes = next_nodes;
            x = bbox.end.x;
            bboxes.push(bbox);
        }
        self.root_bbox = BoundingBox::new_from_children(bboxes);

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

    fn render_preview_drawer(&mut self, layer: &mut Shapes) -> BoundingBox {
        let mut bboxes = Vec::new();
        let root = self.computation_engine.root_node();
        let root = &self.computation_engine[root];
        let mut parameters = HashMap::new();
        for param_desc in root.collect_parameters(&self.computation_engine) {
            parameters.insert(param_desc.id, param_desc.default.clone());
        }
        let value = root.evaluate(&self.computation_engine, &parameters);
        let bbox =
            render_output_preview(Position { x: 0.0, y: 0.0 }, layer, format!("Root"), &value);
        bboxes.push(bbox);
        BoundingBox::new_from_children(bboxes)
    }

    fn render_node_editor(
        &self,
        start: Position,
        layer: &mut Shapes,
        nodes: Vec<(String, NodeId)>,
    ) -> (BoundingBox, Vec<(String, NodeId)>) {
        let mut next_column_nodes = Vec::new();
        let mut bboxes = Vec::new();
        let mut y = 0.0;
        for &(_, node) in &nodes {
            let mut next_node = Some(node);
            while let Some(node_id) = next_node {
                let node = &self.computation_engine[node_id];
                if self.selected_nodes.contains(&node_id) {
                    let params = node.collect_parameters(&self.computation_engine);
                    for (index, arg) in node.arguments.iter().enumerate().rev() {
                        next_column_nodes.push((
                            format!(
                                "{}/{}",
                                node.operation.name(),
                                node.operation.param_name(index, &params)
                            ),
                            *arg,
                        ));
                    }
                }
                next_node = node.input;
            }
        }
        for (name, node) in nodes {
            let node_bbox = self.render_node(Position { x: start.x, y }, layer, node);
            let x = start.x;
            y = node_bbox.end.y + INTER_NODE_PADDING;
            layer.push_rect(RectInstance {
                position: [x, y],
                size: [NODE_WIDTH, NODE_HEIGHT],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT
                    | BOTTOM_OUTLINE_FLAT
                    | LEFT_OUTLINE_FLAT
                    | RIGHT_OUTLINE_FLAT,
            });
            layer.push_text(Text {
                sections: vec![Section::node_label(name)],
                center: [x + NODE_WIDTH / 2.0, y + NODE_HEIGHT / 2.0],
                bounds: [NODE_WIDTH, NODE_HEIGHT],
                horizontal_align: HorizontalAlign::Center,
                vertical_align: VerticalAlign::Center,
            });
            y += NODE_HEIGHT + INTER_PANEL_PADDING;
            bboxes.push(node_bbox);
        }
        next_column_nodes.reverse();
        (BoundingBox::new_from_children(bboxes), next_column_nodes)
    }

    fn render_node(&self, start: Position, layer: &mut Shapes, node_id: NodeId) -> BoundingBox {
        let node = &self.computation_engine[node_id];
        let Position { x, y } = start;
        let mut label = Text {
            sections: vec![Section::node_label(node.operation.name())],
            center: [x + NODE_LABEL_PADDING, y + NODE_HEIGHT / 2.0],
            bounds: [NODE_WIDTH, NODE_HEIGHT],
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Center,
        };
        let mut y = y;
        let mut bboxes = Vec::new();
        if let Some(input) = node.input {
            let bbox = self.render_node(start, layer, input);
            y = bbox.end.y;
            y += INTER_NODE_PADDING;
            bboxes.push(bbox);
        }
        if self.selected_nodes.contains(&node_id) {
            let bottom = y;
            let parameters = node.collect_parameters(&self.computation_engine);
            for (index, _) in node.arguments.iter().enumerate().rev() {
                let start = Position {
                    x: x + NODE_GUTTER_WIDTH + NODE_PARAMETER_PADDING,
                    y,
                };
                let label = node.operation.param_name(index, &parameters);
                let param_bbox = render_parameter(start, layer, label);
                y = param_bbox.end.y + NODE_PARAMETER_PADDING;
                bboxes.push(param_bbox);
            }
            layer.push_rect(RectInstance {
                position: [start.x, bottom],
                size: [NODE_GUTTER_WIDTH, y - bottom],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
            });
        }
        layer.push_rect(RectInstance {
            position: [x, y],
            size: [NODE_WIDTH, NODE_HEIGHT],
            fill_color: NODE_FILL,
            outline_color: NODE_OUTLINE,
            outline_modes: LEFT_OUTLINE_FLAT
                | RIGHT_OUTLINE_FLAT
                | TOP_OUTLINE_FLAT
                | BOTTOM_OUTLINE_FLAT,
        });
        label.center[1] = y + NODE_HEIGHT / 2.0;
        let end = Position {
            x: x + NODE_WIDTH,
            y: y + NODE_HEIGHT,
        };
        // let (fill_color, outline_color) = (VECTOR_TYPE_FILL_COLOR, VECTOR_TYPE_OUTLINE_COLOR);
        let (fill_color, outline_color) = (FLOAT_TYPE_FILL_COLOR, FLOAT_TYPE_OUTLINE_COLOR);
        layer.push_rect(RectInstance {
            position: [start.x, end.y],
            size: [INTER_NODE_PADDING * 2.0, INTER_NODE_PADDING],
            fill_color,
            outline_color,
            outline_modes: LEFT_OUTLINE_DIAGONAL | RIGHT_OUTLINE_ANTIDIAGONAL,
        });
        let kind = self.default_node_bbox_kind(node_id, &node.operation);
        bboxes.push(BoundingBox::new_start_end(Position { x, y }, end, kind));
        layer.push_text(label);
        BoundingBox::new_from_children(bboxes)
    }

    fn default_node_bbox_kind(&self, id: NodeId, operation: &NodeOperation) -> BoundingBoxKind {
        match operation {
            NodeOperation::Literal(Value::Float(..)) => {
                BoundingBoxKind::InvokeTool(self.builtins.adjust_float_tool, id)
            }
            _ => BoundingBoxKind::ToggleNodeSelected(id),
        }
    }
}

fn render_parameter(start: Position, layer: &mut Shapes, name: &str) -> BoundingBox {
    let width = NODE_WIDTH - NODE_PARAMETER_PADDING - NODE_GUTTER_WIDTH;
    let height = NODE_HEIGHT;
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
        center: [start.x + NODE_LABEL_PADDING, start.y + height / 2.0],
        bounds: [width - 2.0 * NODE_LABEL_PADDING, height],
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
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
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
        center: [
            start.x + NODE_PARAMETER_PADDING,
            start.y + size.height / 2.0,
        ],
        bounds: [size.width, size.height],
        horizontal_align: HorizontalAlign::Left,
        vertical_align: VerticalAlign::Center,
    });
    BoundingBox::new_start_size(start, size, bbox_kind)
}
