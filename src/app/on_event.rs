use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::{App, DragTarget};
use crate::{
    engine::ParameterId,
    renderer::Position,
    widgets::{EventResponse, Node, Socket, ValueWidget},
};

impl App {
    pub(super) fn on_event(&mut self, event: Event<()>) {
        match event {
            Event::RedrawRequested(_window_id) => self.render(),
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                if let WindowEvent::Resized(new_size) = event {
                    self.render_engine.resize_target(new_size.into())
                } else {
                    self.on_window_event(event)
                }
            }
            Event::MainEventsCleared => self.window.request_redraw(),
            _ => (),
        }
    }

    fn physical_pos_to_render_pos(&self, pos: PhysicalPosition<f64>) -> Position {
        Position {
            x: pos.x as f32,
            y: self.render_engine.target_size().height - pos.y as f32,
        }
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_input(input),
            WindowEvent::CursorMoved { position, .. } => {
                self.on_mouse_move(self.physical_pos_to_render_pos(position))
            }
            WindowEvent::MouseInput { state, button, ..} => {
                self.on_mouse_input(button, state)
            }
            _ => (),
        }
    }

    fn on_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => self.on_key_down(code),
                ElementState::Released => self.on_key_up(code),
            }
        }
    }

    fn on_key_down(&mut self, code: VirtualKeyCode) {
        match code {
            VirtualKeyCode::Escape => self.control_flow = ControlFlow::Exit,
            _ => (),
        }
    }

    fn on_key_up(&mut self, code: VirtualKeyCode) {
        match code {
            _ => (),
        }
    }

    fn on_mouse_move(&mut self, new_pos: Position) {
        let dx = new_pos.x - self.previous_mouse_pos.x;
        let dy = new_pos.y - self.previous_mouse_pos.y;
        let d = (dx, dy);
        self.previous_mouse_pos = new_pos;
        if let Some(target) = &self.dragging {
            match target {
                DragTarget::Parameter(param) => {
                    self.drag_parameter(*param, d);
                }
            }
        } else {
            self.update_hovering();
        }
    }

    fn update_hovering(&mut self) {
        let Position { x, y } = self.previous_mouse_pos;
        self.hovering = None;
        if x < self.preview_drawer_size {
            let mut dist_from_top = self.render_engine.target_size().height - y;
            for (index, parameter) in self.computation_engine.root_parameters().iter().enumerate() {
                if !self.parameter_widgets.contains_key(&parameter.id) {
                    let value = self.computation_engine.parameter_preview(index);
                    let visual = value.visual(format!("{}: ", parameter.name));
                    self.parameter_widgets.insert(parameter.id, Box::new(visual) as Box<dyn ValueWidget>);
                }
                let visual = &self.parameter_widgets[&parameter.id];
                dist_from_top -= visual.size().height;
                if dist_from_top <= 0.0 {
                    self.hovering = Some(DragTarget::Parameter(parameter.id))
                }
            }
        }
    }

    fn drag_parameter(&mut self, id: ParameterId, d: (f32, f32)) {
        let mut er = EventResponse::default();
        self.parameter_widgets
            .get_mut(&id)
            .unwrap()
            .on_drag(&mut er, d);
        if let Some(value) = er.new_value {
            for (index, param) in self.computation_engine.root_parameters().iter().enumerate() {
                if param.id == id {
                    *self.computation_engine.parameter_preview_mut(index) = value;
                    break;
                }
            }
        }
    }
}
