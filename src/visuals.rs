use crate::{
    constants::{
        self, NODE_BODY_WIDTH, NODE_CORNER_SIZE, NODE_FILL, NODE_HEADER_HEIGHT, NODE_MIN_HEIGHT,
        NODE_OUTLINE, NODE_PADDING,
    },
    renderer::{
        rect_data::{
            RectInstance, BOTTOM_OUTLINE_ANTIDIAGONAL, BOTTOM_OUTLINE_DIAGONAL,
            BOTTOM_OUTLINE_FLAT, LEFT_OUTLINE_FLAT, RIGHT_OUTLINE_FLAT, TOP_OUTLINE_ANTIDIAGONAL,
            TOP_OUTLINE_DIAGONAL, TOP_OUTLINE_FLAT,
        },
        size::Size,
    },
};

#[derive(Clone, Debug)]
pub struct Shapes {
    pub rects: Vec<RectInstance>,
}

impl Shapes {
    pub fn new() -> Self {
        Self { rects: vec![] }
    }

    pub fn push_rect(&mut self, rect: RectInstance) {
        self.rects.push(rect)
    }

    pub fn append(&mut self, other: Self) {
        self.rects.append(&mut { other.rects });
    }
}

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
                width: NODE_BODY_WIDTH,
                height: NODE_MIN_HEIGHT,
            }
        } else {
            let socket_child_sizes = self.sockets.iter().map(Socket::size);
            let size_from_children = socket_child_sizes.fold(Size::zero(), |prev, next| Size {
                width: prev.width + next.width,
                height: prev.height.max(next.height),
            });
            Size {
                width: size_from_children.width
                    + (self.sockets.len() as f32 + 0.5) * NODE_PADDING
                    + NODE_BODY_WIDTH,
                height: size_from_children.height + NODE_PADDING + NODE_HEADER_HEIGHT,
            }
        }
    }

    // x and y are bottom-left corner.
    pub fn draw(&self, x: f32, y: f32) -> Shapes {
        let size = self.size();
        let mut shapes = Shapes::new();
        if self.sockets.len() == 0 {
            let height = NODE_MIN_HEIGHT;
            shapes.push_rect(RectInstance {
                position: [x, y],
                size: [NODE_CORNER_SIZE, height],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT
                    | TOP_OUTLINE_DIAGONAL
                    | BOTTOM_OUTLINE_ANTIDIAGONAL,
            });
            shapes.push_rect(RectInstance {
                position: [x + NODE_CORNER_SIZE, y],
                size: [NODE_BODY_WIDTH - NODE_CORNER_SIZE * 2.0, height],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
            });
            shapes.push_rect(RectInstance {
                position: [x + NODE_BODY_WIDTH - NODE_CORNER_SIZE, y],
                size: [NODE_CORNER_SIZE, height],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: RIGHT_OUTLINE_FLAT
                    | TOP_OUTLINE_ANTIDIAGONAL
                    | BOTTOM_OUTLINE_DIAGONAL,
            });
        } else {
            let mut x = x;
            for (index, socket) in self.sockets.iter().enumerate() {
                let first = index == 0;
                let socket_size = socket.size();
                shapes.append(socket.node.draw(
                    x + 0.5 * NODE_PADDING,
                    y + size.height - socket_size.height - NODE_HEADER_HEIGHT - NODE_PADDING,
                ));
                let last = index == self.sockets.len() - 1;
                let width = socket_size.width + if last { 1.5 } else { 1.0 } * NODE_PADDING;
                shapes.push_rect(RectInstance {
                    position: [x, y + size.height - NODE_HEADER_HEIGHT - NODE_CORNER_SIZE],
                    size: [NODE_CORNER_SIZE, NODE_HEADER_HEIGHT + NODE_CORNER_SIZE],
                    fill_color: NODE_FILL,
                    outline_color: NODE_OUTLINE,
                    outline_modes: if first {
                        LEFT_OUTLINE_FLAT | TOP_OUTLINE_DIAGONAL | BOTTOM_OUTLINE_DIAGONAL
                    } else {
                        TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_DIAGONAL
                    },
                });
                shapes.push_rect(RectInstance {
                    position: [x + NODE_CORNER_SIZE, y + size.height - NODE_HEADER_HEIGHT],
                    size: [width - 2.0 * NODE_CORNER_SIZE, NODE_HEADER_HEIGHT],
                    fill_color: NODE_FILL,
                    outline_color: NODE_OUTLINE,
                    outline_modes: TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
                });
                shapes.push_rect(RectInstance {
                    position: [
                        x + width - NODE_CORNER_SIZE,
                        y + size.height - NODE_HEADER_HEIGHT - NODE_CORNER_SIZE,
                    ],
                    size: [NODE_CORNER_SIZE, NODE_HEADER_HEIGHT + NODE_CORNER_SIZE],
                    fill_color: NODE_FILL,
                    outline_color: NODE_OUTLINE,
                    outline_modes: TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_ANTIDIAGONAL,
                });
                x += width;
            }
            shapes.push_rect(RectInstance {
                position: [x, y + size.height - NODE_HEADER_HEIGHT - NODE_CORNER_SIZE],
                size: [NODE_CORNER_SIZE, NODE_HEADER_HEIGHT + NODE_CORNER_SIZE],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT,
            });
            shapes.push_rect(RectInstance {
                position: [x, y],
                size: [
                    NODE_CORNER_SIZE,
                    size.height - NODE_HEADER_HEIGHT - NODE_CORNER_SIZE,
                ],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: LEFT_OUTLINE_FLAT | BOTTOM_OUTLINE_ANTIDIAGONAL,
            });
            shapes.push_rect(RectInstance {
                position: [x + NODE_CORNER_SIZE, y],
                size: [NODE_BODY_WIDTH - 2.0 * NODE_CORNER_SIZE, size.height],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: TOP_OUTLINE_FLAT | BOTTOM_OUTLINE_FLAT,
            });
            shapes.push_rect(RectInstance {
                position: [x + NODE_BODY_WIDTH - NODE_CORNER_SIZE, y],
                size: [NODE_CORNER_SIZE, size.height],
                fill_color: NODE_FILL,
                outline_color: NODE_OUTLINE,
                outline_modes: RIGHT_OUTLINE_FLAT
                    | TOP_OUTLINE_ANTIDIAGONAL
                    | BOTTOM_OUTLINE_DIAGONAL,
            });
        }
        shapes
    }
}
