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
        let visual = self.computation_engine.root_node().visual();
        let size = visual.size();
        let pos = Position {
            x: self.result_drawer_size,
            y: self.render_engine.target_size().height - size.height,
        };
        let base_layer = visual.draw(pos, "Root");
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
