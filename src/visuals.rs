use crate::{
    renderer::{
        Position, RectInstance, Shapes, Size, BOTTOM_OUTLINE_FLAT, LEFT_OUTLINE_ANTIDIAGONAL,
        LEFT_OUTLINE_DIAGONAL, LEFT_OUTLINE_FLAT, RIGHT_OUTLINE_ANTIDIAGONAL,
        RIGHT_OUTLINE_DIAGONAL, RIGHT_OUTLINE_FLAT, TOP_OUTLINE_FLAT,
    },
    theme::{
        NODE_BODY_HEIGHT, NODE_FILL, NODE_HEADER_WIDTH, NODE_INNER_CORNER_SIZE, NODE_MIN_WIDTH,
        NODE_OUTER_CORNER_SIZE, NODE_OUTLINE, NODE_PADDING,
    },
};

pub struct Socket {
    pub node: Node,
}

impl Socket {
    pub fn new(node: Node) -> Self {
        Self { node }
    }

    pub fn size(&self) -> Size {
        self.node.size()
    }
}

pub struct Node {
    pub sockets: Vec<Socket>,
}

impl Node {
    pub fn size(&self) -> Size {
        if self.sockets.len() == 0 {
            Size {
                width: NODE_MIN_WIDTH,
                height: NODE_BODY_HEIGHT,
            }
        } else {
            let socket_child_sizes = self.sockets.iter().map(Socket::size);
            let size_from_children = socket_child_sizes.fold(Size::zero(), |prev, next| Size {
                width: prev.width.max(next.width),
                height: prev.height + next.height,
            });
            Size {
                width: size_from_children.width + NODE_PADDING + NODE_HEADER_WIDTH,
                height: size_from_children.height
                    + (self.sockets.len() as f32 + 0.5) * NODE_PADDING
                    + NODE_BODY_HEIGHT,
            }
        }
    }

    pub fn draw(&self, start: Position) -> Shapes {
        let Position { x, y } = start;
        let size = self.size();
        let mut shapes = Shapes::new();
        if self.sockets.len() == 0 {
            shapes.push_rect(RectInstance {
                position: [x, y],
                size: [NODE_MIN_WIDTH, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: BOTTOM_OUTLINE_FLAT
                    | LEFT_OUTLINE_ANTIDIAGONAL
                    | RIGHT_OUTLINE_DIAGONAL,
            });
            shapes.push_rect(RectInstance {
                position: [x, y + NODE_OUTER_CORNER_SIZE],
                size: [
                    NODE_MIN_WIDTH,
                    NODE_BODY_HEIGHT - 2.0 * NODE_OUTER_CORNER_SIZE,
                ],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
            });
            shapes.push_rect(RectInstance {
                position: [x, y + NODE_BODY_HEIGHT - NODE_OUTER_CORNER_SIZE],
                size: [NODE_MIN_WIDTH, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT
                    | LEFT_OUTLINE_DIAGONAL
                    | RIGHT_OUTLINE_ANTIDIAGONAL,
            });
        } else {
            let mut y = y;
            for (index, socket) in self.sockets.iter().enumerate() {
                let first = index == 0;
                let socket_size = socket.size();
                let last = index == self.sockets.len() - 1;
                let height = socket_size.height + if last { 1.5 } else { 1.0 } * NODE_PADDING;
                shapes.append(socket.node.draw(Position {
                    x: x + NODE_HEADER_WIDTH + NODE_PADDING,
                    y: y + 0.5 * NODE_PADDING,
                }));
                if first {
                    shapes.push_rect(RectInstance {
                        position: [x, y],
                        size: [
                            NODE_HEADER_WIDTH + NODE_OUTER_CORNER_SIZE,
                            NODE_OUTER_CORNER_SIZE,
                        ],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: BOTTOM_OUTLINE_FLAT
                            | LEFT_OUTLINE_ANTIDIAGONAL
                            | RIGHT_OUTLINE_ANTIDIAGONAL,
                    });
                    shapes.push_rect(RectInstance {
                        position: [x, y + NODE_OUTER_CORNER_SIZE],
                        size: [
                            NODE_HEADER_WIDTH,
                            height - NODE_OUTER_CORNER_SIZE - NODE_INNER_CORNER_SIZE,
                        ],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
                    });
                } else {
                    shapes.push_rect(RectInstance {
                        position: [x, y],
                        size: [
                            NODE_HEADER_WIDTH + NODE_INNER_CORNER_SIZE,
                            NODE_INNER_CORNER_SIZE,
                        ],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_ANTIDIAGONAL,
                    });
                    shapes.push_rect(RectInstance {
                        position: [x, y + NODE_INNER_CORNER_SIZE],
                        size: [NODE_HEADER_WIDTH, height - 2.0 * NODE_INNER_CORNER_SIZE],
                        fill_color: NODE_FILL,
                        outline_color: NODE_OUTLINE,
                        outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
                    });
                }
                shapes.push_rect(RectInstance {
                    position: [x, y + height - NODE_INNER_CORNER_SIZE],
                    size: [
                        NODE_HEADER_WIDTH + NODE_INNER_CORNER_SIZE,
                        NODE_INNER_CORNER_SIZE,
                    ],
                    fill_color: NODE_FILL,
                    outline_color: NODE_OUTLINE,
                    outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_DIAGONAL,
                });
                y += height;
            }
            let skip = NODE_HEADER_WIDTH + NODE_INNER_CORNER_SIZE;
            shapes.push_rect(RectInstance {
                position: [x, y],
                size: [skip, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT,
            });
            shapes.push_rect(RectInstance {
                position: [x + skip, y],
                size: [NODE_MIN_WIDTH - skip, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: BOTTOM_OUTLINE_FLAT | RIGHT_OUTLINE_DIAGONAL,
            });
            shapes.push_rect(RectInstance {
                position: [x, y + NODE_OUTER_CORNER_SIZE],
                size: [
                    NODE_MIN_WIDTH,
                    NODE_BODY_HEIGHT - 2.0 * NODE_OUTER_CORNER_SIZE,
                ],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT | RIGHT_OUTLINE_FLAT,
            });
            shapes.push_rect(RectInstance {
                position: [x, y + NODE_BODY_HEIGHT - NODE_OUTER_CORNER_SIZE],
                size: [NODE_MIN_WIDTH, NODE_OUTER_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT
                    | LEFT_OUTLINE_DIAGONAL
                    | RIGHT_OUTLINE_ANTIDIAGONAL,
            });
        }
        shapes
    }
}
