use wgpu::SurfaceError;
use wgpu_glyph::{HorizontalAlign, VerticalAlign};
use winit::event_loop::ControlFlow;

use super::App;
use crate::{
    engine::{Node, Parameter, ParameterId, Value},
    renderer::{
        Position, RectInstance, Section, Shapes, Size, Text, BOTTOM_OUTLINE_FLAT,
        LEFT_OUTLINE_ANTIDIAGONAL, LEFT_OUTLINE_DIAGONAL, LEFT_OUTLINE_FLAT,
        RIGHT_OUTLINE_ANTIDIAGONAL, RIGHT_OUTLINE_DIAGONAL, RIGHT_OUTLINE_FLAT, TOP_OUTLINE_FLAT,
    },
    theme::{
        NODE_FILL, NODE_GUTTER_WIDTH, NODE_HEADER_HEIGHT, NODE_INNER_CORNER_SIZE, NODE_MIN_WIDTH,
        NODE_OUTER_CORNER_SIZE, NODE_OUTLINE, NODE_PADDING, PARAMETER_LABEL_COLOR,
        PARAMETER_LABEL_SIZE,
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
        let root_node = &self.computation_engine[self.computation_engine.root_node()];
        let pos = Position {
            x: self.preview_drawer_size,
            y: 0.0,
        };
        self.render_node(pos, layer, root_node, "Root")
    }

    fn render_preview_drawer(&mut self, layer: &mut Shapes) -> BoundingBox {
        let mut y = 0.0;
        let mut bboxes = Vec::new();
        for (index, parameter) in self.computation_engine.root_parameters().iter().enumerate() {
            let value = self.computation_engine.parameter_preview(index);
            let bbox = render_parameter_preview(Position { x: 0.0, y }, layer, parameter, value);
            y = bbox.end.y;
            bboxes.push(bbox);
        }
        let value = self.computation_engine.evaluate_root_result_preview();
        let bbox = render_output_preview(Position { x: 0.0, y }, layer, format!("Root"), &value);
        bboxes.push(bbox);
        BoundingBox::new_from_children(bboxes)
    }

    fn render_node(
        &self,
        start: Position,
        layer: &mut Shapes,
        node: &Node,
        label: &str,
    ) -> BoundingBox {
        let Position { x, y } = start;
        let mut label = Text {
            sections: vec![
                Section::parameter_label(format!("{}: ", label)),
                Section::node_label(node.operation.name()),
            ],
            center: [x + NODE_PADDING, y + NODE_HEADER_HEIGHT / 2.0],
            bounds: [NODE_MIN_WIDTH, NODE_HEADER_HEIGHT],
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Center,
        };
        let bbox = if node.arguments.len() == 0 {
            layer.push_rect(RectInstance {
                position: [x, y],
                size: [NODE_MIN_WIDTH, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: BOTTOM_OUTLINE_FLAT
                    | LEFT_OUTLINE_ANTIDIAGONAL
                    | RIGHT_OUTLINE_DIAGONAL,
            });
            layer.push_rect(RectInstance {
                position: [x, y + NODE_OUTER_CORNER_SIZE],
                size: [
                    NODE_MIN_WIDTH,
                    NODE_HEADER_HEIGHT - 2.0 * NODE_OUTER_CORNER_SIZE,
                ],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
            });
            layer.push_rect(RectInstance {
                position: [x, y + NODE_HEADER_HEIGHT - NODE_OUTER_CORNER_SIZE],
                size: [NODE_MIN_WIDTH, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT
                    | LEFT_OUTLINE_DIAGONAL
                    | RIGHT_OUTLINE_ANTIDIAGONAL,
            });
            let size = Size {
                width: NODE_MIN_WIDTH,
                height: NODE_HEADER_HEIGHT,
            };
            let kind = BoundingBoxKind::Unused;
            BoundingBox::new_start_size(start, size, kind)
        } else {
            let mut y = y;
            let arg_names = node.operation.arg_names();
            let mut arg_bboxes = Vec::new();
            for (index, arg) in node.arguments.iter().enumerate() {
                let first = index == 0;
                let last = index == node.arguments.len() - 1;
                let start = Position {
                    x: x + NODE_GUTTER_WIDTH + NODE_PADDING,
                    y: y + 0.5 * NODE_PADDING,
                };
                let arg = &self.computation_engine[*arg];
                let label = arg_names[index.min(arg_names.len() - 1)];
                let arg_bbox = self.render_node(start, layer, arg, label);
                let height = arg_bbox.size().height + if last { 1.5 } else { 1.0 } * NODE_PADDING;
                arg_bboxes.push(arg_bbox);
                if first {
                    layer.push_rect(RectInstance {
                        position: [x, y],
                        size: [
                            NODE_GUTTER_WIDTH + NODE_OUTER_CORNER_SIZE,
                            NODE_OUTER_CORNER_SIZE,
                        ],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: BOTTOM_OUTLINE_FLAT
                            | LEFT_OUTLINE_ANTIDIAGONAL
                            | RIGHT_OUTLINE_ANTIDIAGONAL,
                    });
                    layer.push_rect(RectInstance {
                        position: [x, y + NODE_OUTER_CORNER_SIZE],
                        size: [
                            NODE_GUTTER_WIDTH,
                            height - NODE_OUTER_CORNER_SIZE - NODE_INNER_CORNER_SIZE,
                        ],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
                    });
                } else {
                    layer.push_rect(RectInstance {
                        position: [x, y],
                        size: [
                            NODE_GUTTER_WIDTH + NODE_INNER_CORNER_SIZE,
                            NODE_INNER_CORNER_SIZE,
                        ],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_ANTIDIAGONAL,
                    });
                    layer.push_rect(RectInstance {
                        position: [x, y + NODE_INNER_CORNER_SIZE],
                        size: [NODE_GUTTER_WIDTH, height - 2.0 * NODE_INNER_CORNER_SIZE],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
                    });
                }
                layer.push_rect(RectInstance {
                    position: [x, y + height - NODE_INNER_CORNER_SIZE],
                    size: [
                        NODE_GUTTER_WIDTH + NODE_INNER_CORNER_SIZE,
                        NODE_INNER_CORNER_SIZE,
                    ],
                    fill_color: NODE_FILL,
                    outline_color: NODE_OUTLINE,
                    outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_DIAGONAL,
                });
                y += height;
            }
            let skip = NODE_GUTTER_WIDTH + NODE_INNER_CORNER_SIZE;
            layer.push_rect(RectInstance {
                position: [x, y],
                size: [skip, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT,
            });
            layer.push_rect(RectInstance {
                position: [x + skip, y],
                size: [NODE_MIN_WIDTH - skip, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: BOTTOM_OUTLINE_FLAT | RIGHT_OUTLINE_DIAGONAL,
            });
            layer.push_rect(RectInstance {
                position: [x, y + NODE_OUTER_CORNER_SIZE],
                size: [
                    NODE_MIN_WIDTH,
                    NODE_HEADER_HEIGHT - 2.0 * NODE_OUTER_CORNER_SIZE,
                ],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
            });
            layer.push_rect(RectInstance {
                position: [x, y + NODE_HEADER_HEIGHT - NODE_OUTER_CORNER_SIZE],
                size: [NODE_MIN_WIDTH, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT
                    | LEFT_OUTLINE_DIAGONAL
                    | RIGHT_OUTLINE_ANTIDIAGONAL,
            });
            label.center[1] = y + NODE_HEADER_HEIGHT / 2.0;
            let end = Position {
                x: x + NODE_MIN_WIDTH,
                y: y + NODE_HEADER_HEIGHT,
            };
            let kind = BoundingBoxKind::Parent(arg_bboxes);
            BoundingBox::new_start_end(start, end, kind)
        };
        layer.push_text(label);
        bbox
    }
}

fn render_parameter_preview(
    start: Position,
    layer: &mut Shapes,
    parameter: &Parameter,
    value: &Value,
) -> BoundingBox {
    let bbox_kind = BoundingBoxKind::EditParameter(parameter.id);
    render_value_preview_helper(start, layer, parameter.name.clone(), value, bbox_kind)
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
            Section::parameter_label(label.clone()),
            Section::node_label(value.display()),
        ],
        center: [start.x + NODE_PADDING, start.y + size.height / 2.0],
        bounds: [size.width, size.height],
        horizontal_align: HorizontalAlign::Left,
        vertical_align: VerticalAlign::Center,
    });
    BoundingBox::new_start_size(start, size, bbox_kind)
}
