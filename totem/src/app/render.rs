use std::collections::HashMap;

use renderer::{
    winit::ControlFlow, HorizontalAlign, IconInstance, Position, RectInstance, Section, Shapes,
    Size, SurfaceError, Text, VerticalAlign, BOTTOM_OUTLINE_FLAT, LEFT_OUTLINE_ANTIDIAGONAL,
    LEFT_OUTLINE_DIAGONAL, LEFT_OUTLINE_FLAT, RIGHT_OUTLINE_ANTIDIAGONAL, RIGHT_OUTLINE_DIAGONAL,
    RIGHT_OUTLINE_FLAT, TOP_OUTLINE_FLAT,
};
use theme::{
    column_colors, INTER_NODE_PADDING, INTER_PANEL_PADDING, NODE_CORNER_SIZE, NODE_FILL,
    NODE_GUTTER_WIDTH, NODE_HEIGHT, NODE_ICON_PADDING, NODE_ICON_SIZE, NODE_LABEL_PADDING,
    NODE_OUTLINE, NODE_PARAMETER_PADDING, NODE_WIDTH, PREVIEW_WIDGET_SIZE, TOOL_BUTTON_PADDING,
    TOOL_BUTTON_SIZE, TOOL_ICON_SIZE,
};

use super::App;
use crate::{
    engine::{
        BaseType, Node, NodeId, NodeOperation, Parameter, ParameterDescription, ParameterId, Type,
        Value,
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
        let mut index = 0;
        while editor_nodes.len() > 0 {
            let (bbox, next_nodes) = self.render_node_editor(
                Position {
                    x: x + INTER_PANEL_PADDING,
                    y: 0.0,
                },
                &mut base_layer,
                index,
                editor_nodes,
            );
            editor_nodes = next_nodes;
            x = bbox.end.x;
            bboxes.push(bbox);
            index += 1;
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

    pub fn active_node(&self) -> NodeId {
        self.selected_node_path.last().copied().unwrap()
    }

    fn render_preview_drawer(&mut self, layer: &mut Shapes) -> BoundingBox {
        let mut bboxes = Vec::new();
        let active = &self.computation_engine[self.active_node()];
        let mut parameters = HashMap::new();
        for param_desc in active.collect_parameters(&self.computation_engine) {
            parameters.insert(param_desc.id, param_desc.default.clone());
        }
        let value = active.evaluate(&self.computation_engine, &parameters);
        let bbox = render_output_preview(Position { x: 0.0, y: 0.0 }, layer, &value);
        let y = bbox.end.y + INTER_PANEL_PADDING;
        bboxes.push(bbox);
        let bbox = self.render_toolbox(Position { x: 0.0, y }, layer);
        bboxes.push(bbox);
        BoundingBox::new_from_children(bboxes)
    }

    fn render_node_editor(
        &self,
        start: Position,
        layer: &mut Shapes,
        index: usize,
        nodes: Vec<(String, NodeId)>,
    ) -> (BoundingBox, Vec<(String, NodeId)>) {
        let mut next_column_nodes = Vec::new();
        let mut bboxes = Vec::new();
        let mut y = 0.0;
        for &(_, node) in &nodes {
            let mut next_node = Some(node);
            while let Some(node_id) = next_node {
                let node = &self.computation_engine[node_id];
                if self.selected_node_path.contains(&node_id) {
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
            let node_bbox = self.render_node(Position { x: start.x, y }, layer, node, index);
            let x = start.x;
            y = node_bbox.end.y + INTER_NODE_PADDING;
            let [fill_color, outline_color] = column_colors()[index];
            layer.push_rect(RectInstance {
                position: [x, y],
                size: [NODE_WIDTH, NODE_HEIGHT],
                fill_color,
                outline_color,
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

    fn render_node(
        &self,
        start: Position,
        layer: &mut Shapes,
        node_id: NodeId,
        containing_editor_index: usize,
    ) -> BoundingBox {
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
            let bbox = self.render_node(start, layer, input, containing_editor_index);
            y = bbox.end.y;
            y += INTER_NODE_PADDING;
            bboxes.push(bbox);
        }
        let output_type = self.type_of_node(node_id);
        let [fill_color, outline_color] = column_colors()[containing_editor_index];
        let bottom = y;
        if self.selected_node_path.contains(&node_id) {
            let parameters = node.collect_parameters(&self.computation_engine);
            for (index, arg) in node.arguments.iter().enumerate().rev() {
                let start = Position {
                    x: x + NODE_GUTTER_WIDTH + NODE_PARAMETER_PADDING,
                    y,
                };
                let label = node.operation.param_name(index, &parameters);
                let param_bbox = render_parameter(
                    start,
                    layer,
                    label,
                    column_colors()[containing_editor_index + 1],
                );
                y = param_bbox.end.y + NODE_PARAMETER_PADDING;
                bboxes.push(param_bbox);
            }
        }
        let selected = self.selected_node_path.last() == Some(&node_id);
        layer.push_rect(RectInstance {
            position: [x, y],
            size: [NODE_GUTTER_WIDTH, NODE_HEIGHT],
            fill_color,
            outline_color,
            outline_modes: LEFT_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
        });
        layer.push_rect(RectInstance {
            position: [x + NODE_GUTTER_WIDTH, y],
            size: [NODE_WIDTH - NODE_GUTTER_WIDTH, NODE_HEIGHT],
            fill_color,
            outline_color,
            outline_modes: RIGHT_OUTLINE_FLAT | TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
        });
        if self.selected_node_path.contains(&node_id) {
            layer.push_rect(RectInstance {
                position: [start.x, bottom],
                size: [NODE_GUTTER_WIDTH, y - bottom + 1.0],
                fill_color,
                outline_color,
                outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
            });
        }
        label.center[1] = y + NODE_HEIGHT / 2.0;
        let end = Position {
            x: x + NODE_WIDTH,
            y: y + NODE_HEIGHT,
        };
        let d = NODE_ICON_PADDING + NODE_ICON_SIZE;
        layer.push_icon(IconInstance {
            position: [end.x - d, end.y - d],
            size: NODE_ICON_SIZE,
            index: if selected { 1 } else { 0 },
        });
        layer.push_rect(RectInstance {
            position: [start.x, end.y],
            size: [INTER_NODE_PADDING * 2.0, INTER_NODE_PADDING],
            fill_color,
            outline_color,
            outline_modes: LEFT_OUTLINE_DIAGONAL | RIGHT_OUTLINE_ANTIDIAGONAL,
        });
        let kind = self.default_node_bbox_kind(node_id, &node.operation, containing_editor_index);
        bboxes.push(BoundingBox::new_start_end(Position { x, y }, end, kind));
        layer.push_text(label);
        BoundingBox::new_from_children(bboxes)
    }

    fn default_node_bbox_kind(
        &self,
        id: NodeId,
        operation: &NodeOperation,
        containing_editor_index: usize,
    ) -> BoundingBoxKind {
        match operation {
            _ => BoundingBoxKind::SelectNode(containing_editor_index, id),
        }
    }

    fn render_toolbox(&self, start: Position, layer: &mut Shapes) -> BoundingBox {
        let mut bboxes = Vec::new();
        layer.push_rect(RectInstance {
            position: [start.x, start.y],
            size: [TOOL_BUTTON_SIZE, TOOL_BUTTON_SIZE],
            fill_color: column_colors()[0][0],
            outline_color: column_colors()[0][1],
            outline_modes: TOP_OUTLINE_FLAT
                | BOTTOM_OUTLINE_FLAT
                | LEFT_OUTLINE_FLAT
                | RIGHT_OUTLINE_FLAT,
        });
        layer.push_icon(IconInstance {
            position: [start.x + TOOL_BUTTON_PADDING, start.y + TOOL_BUTTON_PADDING],
            size: TOOL_ICON_SIZE,
            index: 1,
        });
        bboxes.push(BoundingBox::new_start_end(
            start,
            Position {
                x: start.x + TOOL_BUTTON_SIZE,
                y: start.y + TOOL_BUTTON_SIZE,
            },
            BoundingBoxKind::InvokeTool(self.builtins.adjust_float_tool),
        ));
        BoundingBox::new_from_children(bboxes)
    }
}

fn render_parameter(
    start: Position,
    layer: &mut Shapes,
    name: &str,
    [fill_color, outline_color]: [[f32; 3]; 2],
) -> BoundingBox {
    let width = NODE_WIDTH - NODE_PARAMETER_PADDING - NODE_GUTTER_WIDTH;
    let height = NODE_HEIGHT;
    let kind = BoundingBoxKind::Unused;
    layer.push_rect(RectInstance {
        position: [start.x, start.y],
        size: [width, height],
        fill_color,
        outline_color,
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

fn render_output_preview(start: Position, layer: &mut Shapes, value: &Value) -> BoundingBox {
    let size = Size {
        width: PREVIEW_WIDGET_SIZE,
        height: PREVIEW_WIDGET_SIZE,
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
        sections: vec![Section::big_value_text(value.display())],
        center: [start.x + size.width / 2.0, start.y + size.height / 2.0],
        bounds: [size.width, size.height],
        horizontal_align: HorizontalAlign::Center,
        vertical_align: VerticalAlign::Center,
    });
    BoundingBox::new_start_size(start, size, BoundingBoxKind::Unused)
}
