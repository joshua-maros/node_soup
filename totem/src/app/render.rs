use std::{collections::HashMap, time::Instant};

use renderer::{
    winit::ControlFlow, HorizontalAlign, IconInstance, ImageInstance, Position, RectInstance,
    Section, Shapes, Size, SurfaceError, Text, VerticalAlign, BOTTOM_OUTLINE_FLAT,
    LEFT_OUTLINE_DIAGONAL, LEFT_OUTLINE_FLAT, RIGHT_OUTLINE_ANTIDIAGONAL, RIGHT_OUTLINE_FLAT,
    TOP_OUTLINE_FLAT,
};
use theme::{
    column_colors, INTER_NODE_PADDING, INTER_PANEL_PADDING, NODE_FILL, NODE_GUTTER_WIDTH,
    NODE_ICON_PADDING, NODE_ICON_SIZE, NODE_LABEL_HEIGHT, NODE_LABEL_PADDING, NODE_OUTLINE,
    NODE_PARAMETER_PADDING, NODE_WIDTH, PREVIEW_TEXTURE_SIZE, PREVIEW_WIDGET_SIZE,
    TOOL_BUTTON_PADDING, TOOL_BUTTON_SIZE, TOOL_ICON_SIZE,
};

use super::App;
use crate::{
    engine::{TypedBlob, Node, NodeId, NodeOperation},
    widgets::{BoundingBox, BoundingBoxKind},
};

impl App {
    pub(super) fn render(&mut self) {
        let total_start = Instant::now();
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
        let start = Instant::now();
        let result = self.render_engine.render(&layers);
        self.perf_counters.gpu_time_acc += start.elapsed();
        match result {
            Ok(()) => (),
            Err(SurfaceError::Lost) | Err(SurfaceError::Outdated) => {
                self.render_engine.refresh_target()
            }
            Err(SurfaceError::OutOfMemory) => self.control_flow = ControlFlow::ExitWithCode(1),
            Err(e) => eprintln!("{:#?}", e),
        }
        self.perf_counters.total_time_acc += total_start.elapsed();
        self.perf_counters.samples += 1;
        self.perf_counters.report_and_reset_if_appropriate();
    }

    pub fn active_node(&self) -> NodeId {
        self.selected_node_path.last().copied().unwrap()
    }

    fn render_preview_drawer(&mut self, layer: &mut Shapes) -> BoundingBox {
        let mut bboxes = Vec::new();
        let bbox =
            self.render_output_preview(Position { x: 0.0, y: 0.0 }, layer, self.active_node());
        let y = bbox.end.y + INTER_PANEL_PADDING;
        bboxes.push(bbox);
        let bbox = self.render_toolbox(Position { x: 0.0, y }, layer);
        bboxes.push(bbox);
        BoundingBox::new_from_children(bboxes)
    }

    fn render_output_preview(
        &mut self,
        position: Position,
        layer: &mut Shapes,
        output_of: NodeId,
    ) -> BoundingBox {
        let node = &self.computation_engine[output_of];
        let parameters = node.collect_parameters(self.computation_engine.nodes());
        let mut arguments = HashMap::new();
        for param_desc in &parameters {
            arguments.insert(param_desc.id, param_desc.default.clone());
        }
        let start = Instant::now();
        self.computation_engine.compile(output_of);
        self.perf_counters.compilation_time_acc += start.elapsed();
        if parameters
            .iter()
            .any(|param| param.id == self.builtins.display_position.0)
        {
            let mut data = [[0; 4]; (PREVIEW_TEXTURE_SIZE * PREVIEW_TEXTURE_SIZE) as usize];
            let mut input_output = self.computation_engine.default_io_blob(output_of);
            let start = Instant::now();
            self.computation_engine.execute_multiple_times(
                output_of,
                &mut input_output,
                (PREVIEW_TEXTURE_SIZE * PREVIEW_TEXTURE_SIZE) as usize,
                |io, time| {
                    let x = time % PREVIEW_TEXTURE_SIZE as usize;
                    let y = time / PREVIEW_TEXTURE_SIZE as usize;
                    unsafe {
                        io.as_raw_bytes_mut()[4..8].copy_from_slice(&(x as f32).to_ne_bytes());
                        io.as_raw_bytes_mut()[8..12].copy_from_slice(&(y as f32).to_ne_bytes());
                    }
                },
                |io, time| {
                    let value =
                        unsafe { f32::from_ne_bytes(io.as_raw_bytes_mut().as_chunks().0[0]) };
                    let v = (value.clamp(0.0, 1.0) * 255.99) as u8;
                    data[time] = [v, v, v, v]
                },
            );
            self.perf_counters.execution_time_acc += start.elapsed();

            let start = Instant::now();
            self.render_engine.upload_image(0, &data);
            self.perf_counters.upload_time_acc += start.elapsed();

            render_texture_output_preview(position, layer, 0)
        } else {
            let mut io = self.computation_engine.default_io_blob(output_of);
            let start = Instant::now();
            self.computation_engine.execute(output_of, &mut io);
            self.perf_counters.execution_time_acc += start.elapsed();
            let value = unsafe { f32::from_ne_bytes(io.as_raw_bytes_mut().as_chunks().0[0]) };
            render_simple_output_preview(position, layer, &value.into())
        }
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
                    let params = node.collect_parameters(self.computation_engine.nodes());
                    for (index, arg) in node.arguments.iter().enumerate().rev() {
                        next_column_nodes.push((
                            format!("{}", node.operation.param_name(index, &params)),
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
                size: [NODE_WIDTH, NODE_LABEL_HEIGHT],
                fill_color,
                outline_color,
                outline_modes: TOP_OUTLINE_FLAT
                    | BOTTOM_OUTLINE_FLAT
                    | LEFT_OUTLINE_FLAT
                    | RIGHT_OUTLINE_FLAT,
            });
            layer.push_text(Text {
                sections: vec![Section::node_label(name)],
                center: [x + NODE_LABEL_PADDING, y + NODE_LABEL_HEIGHT / 2.0],
                bounds: [NODE_WIDTH, NODE_LABEL_HEIGHT],
                horizontal_align: HorizontalAlign::Left,
                vertical_align: VerticalAlign::Center,
            });
            y += NODE_LABEL_HEIGHT + INTER_PANEL_PADDING;
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
            center: [x + NODE_LABEL_PADDING, y + NODE_LABEL_HEIGHT / 2.0],
            bounds: [NODE_WIDTH, NODE_LABEL_HEIGHT],
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
        let [fill_color, outline_color] = column_colors()[containing_editor_index];
        let bottom = y;
        if self.selected_node_path.contains(&node_id) {
            let parameters = node.collect_parameters(self.computation_engine.nodes());
            for (index, _) in node.arguments.iter().enumerate().rev() {
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
        let height = if self.selected_node_path.last() == Some(&node_id) {
            NODE_LABEL_HEIGHT + NODE_ICON_SIZE + 2.0 * NODE_ICON_PADDING - NODE_LABEL_PADDING - 2.0
        } else {
            NODE_LABEL_HEIGHT
        };
        layer.push_rect(RectInstance {
            position: [x, y],
            size: [NODE_GUTTER_WIDTH, height],
            fill_color,
            outline_color,
            outline_modes: LEFT_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
        });
        layer.push_rect(RectInstance {
            position: [x + NODE_GUTTER_WIDTH, y],
            size: [NODE_WIDTH - NODE_GUTTER_WIDTH, height],
            fill_color,
            outline_color,
            outline_modes: RIGHT_OUTLINE_FLAT | TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
        });
        label.center[1] = y + NODE_LABEL_HEIGHT / 2.0;
        let end = Position {
            x: x + NODE_WIDTH,
            y: y + height,
        };
        let icon_d = NODE_ICON_PADDING + NODE_ICON_SIZE;
        layer.push_rect(RectInstance {
            position: [start.x, end.y],
            size: [INTER_NODE_PADDING * 2.0, INTER_NODE_PADDING],
            fill_color,
            outline_color,
            outline_modes: LEFT_OUTLINE_DIAGONAL | RIGHT_OUTLINE_ANTIDIAGONAL,
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
        if self.selected_node_path.last() == Some(&node_id) {
            layer.push_icon(IconInstance {
                position: [start.x + NODE_ICON_PADDING, end.y - icon_d],
                size: NODE_ICON_SIZE,
                index: 1,
            });
            layer.push_icon(IconInstance {
                position: [end.x - icon_d, end.y - icon_d],
                size: NODE_ICON_SIZE,
                index: 1,
            });
            layer.push_icon(IconInstance {
                position: [end.x - 2.0 * icon_d, end.y - icon_d],
                size: NODE_ICON_SIZE,
                index: 1,
            });
            layer.push_icon(IconInstance {
                position: [end.x - 3.0 * icon_d, end.y - icon_d],
                size: NODE_ICON_SIZE,
                index: 1,
            });
        }
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
    let height = NODE_LABEL_HEIGHT;
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
        center: [
            start.x + NODE_WIDTH - NODE_LABEL_PADDING - NODE_GUTTER_WIDTH - NODE_PARAMETER_PADDING,
            start.y + height / 2.0,
        ],
        bounds: [width - 2.0 * NODE_LABEL_PADDING, height],
        horizontal_align: HorizontalAlign::Right,
        vertical_align: VerticalAlign::Center,
    });
    BoundingBox::new_start_size(start, Size { width, height }, kind)
}

fn render_simple_output_preview(start: Position, layer: &mut Shapes, value: &TypedBlob) -> BoundingBox {
    let size = PREVIEW_WIDGET_SIZE;
    layer.push_rect(RectInstance {
        position: [start.x, start.y],
        size: [size, size],
        fill_color: NODE_FILL,
        outline_color: NODE_OUTLINE,
        outline_modes: TOP_OUTLINE_FLAT
            | BOTTOM_OUTLINE_FLAT
            | LEFT_OUTLINE_FLAT
            | RIGHT_OUTLINE_FLAT,
    });
    layer.push_text(Text {
        sections: vec![Section::big_value_text(format!("{:?}", value))],
        center: [start.x + size / 2.0, start.y + size / 2.0],
        bounds: [size, size],
        horizontal_align: HorizontalAlign::Center,
        vertical_align: VerticalAlign::Center,
    });
    let size = Size {
        width: size,
        height: size,
    };
    BoundingBox::new_start_size(start, size, BoundingBoxKind::Unused)
}

fn render_texture_output_preview(
    start: Position,
    layer: &mut Shapes,
    image_index: i32,
) -> BoundingBox {
    let size = PREVIEW_WIDGET_SIZE;
    layer.push_image(ImageInstance {
        position: [start.x, start.y],
        size,
        index: image_index,
    });
    let size = Size {
        width: size,
        height: size,
    };
    BoundingBox::new_start_size(start, size, BoundingBoxKind::Unused)
}
