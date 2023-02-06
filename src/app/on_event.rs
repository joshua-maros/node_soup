use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::{App, DragTarget};
use crate::{
    engine::{ParameterId, Value},
    renderer::Position,
    widgets::{BoundingBoxKind, EventResponse, Node, Socket, ValueWidget},
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
            WindowEvent::MouseInput { state, button, .. } => self.on_mouse_input(button, state),
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
                BoundingBoxKind::EditParameter(param) => {
                    self.drag_parameter(*param, d);
                }
                _ => (),
            }
        } else {
            self.update_hovering();
        }
    }

    fn update_hovering(&mut self) {
        self.hovering = None;
        let mut candidate = &self.root_bbox;
        'look_into_candidate: while let BoundingBoxKind::Parent(children) = &candidate.kind {
            for child in children {
                if child.contains(self.previous_mouse_pos) {
                    candidate = child;
                    continue 'look_into_candidate;
                }
            }
            return;
        }
        self.hovering = Some(candidate.kind.clone());
    }

    fn drag_parameter(&mut self, id: ParameterId, d: (f32, f32)) {
        let value = self.computation_engine.parameter_preview_mut(id);
        if let Value::Float(value) = value {
            let d = d.0 + d.1;
            *value += d;
        }
    }
}
