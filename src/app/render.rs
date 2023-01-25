use wgpu::SurfaceError;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::App;
use crate::{
    renderer::Position,
    visuals::{Node, Socket},
};

impl App {
    pub(super) fn render(&mut self) {
        let node = Node {
            name: format!("Sum"),
            sockets: vec![
                Socket::new(
                    Node {
                        name: format!("Something"),
                        sockets: vec![Socket::new(
                            Node {
                                name: format!("0.0"),
                                sockets: vec![],
                            },
                            format!("Input"),
                        )],
                    },
                    format!("B"),
                ),
                Socket::new(
                    Node {
                        name: format!("1.3"),
                        sockets: vec![],
                    },
                    format!("A"),
                ),
            ],
        };
        let base_layer = node.draw(Position { x: 0.0, y: 0.0 }, "Root");
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
}
