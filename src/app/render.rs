use wgpu::SurfaceError;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::App;
use crate::{
    renderer::{Position, Shapes},
    visuals::{Node, Socket, ValueWidget},
};

impl App {
    pub(super) fn render(&mut self) {
        let mut base_layer = Shapes::new();
        self.render_root_node(&mut base_layer);
        self.render_preview_drawer(&mut base_layer);

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

    fn render_root_node(&mut self, layer: &mut Shapes) {
        let root_node = &self.computation_engine[self.computation_engine.root_node()];
        let node_visual = root_node.visual(&self.computation_engine);
        let size = node_visual.size();
        let pos = Position {
            x: self.preview_drawer_size,
            y: self.render_engine.target_size().height - size.height,
        };
        node_visual.draw(pos, "Root", layer);
    }

    fn render_preview_drawer(&mut self, layer: &mut Shapes) {
        let mut y = self.render_engine.target_size().height;
        for (index, parameter) in self.computation_engine.root_parameters().iter().enumerate() {
            if !self.parameter_widgets.contains_key(&parameter.id) {
                let value = self.computation_engine.parameter_preview(index);
                let visual = value.visual(format!("{}: ", parameter.name));
                self.parameter_widgets.insert(parameter.id, Box::new(visual) as Box<dyn ValueWidget>);
            }
            let visual = &self.parameter_widgets[&parameter.id];
            y -= visual.size().height;
            visual.draw(Position { x: 0.0, y }, layer);
        }
        let result = self.computation_engine.evaluate_root_result_preview();
        let visual = result.visual(format!("Root: "));
        y -= visual.size().height;
        visual.draw(Position { x: 0.0, y }, layer);
    }
}
