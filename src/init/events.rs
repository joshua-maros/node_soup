use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::old_renderer::{RenderTarget, VisualNode};

pub struct EventHandler {
    control_flow: ControlFlow,
}

impl EventHandler {
    pub fn create() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        (
            Self {
                control_flow: ControlFlow::Poll,
            },
            event_loop,
        )
    }

    pub fn handle_event(
        &mut self,
        event: Event<()>,
        render_target: &mut RenderTarget,
        render_object: &VisualNode,
    ) -> ControlFlow {
        self.control_flow = ControlFlow::Poll;
        match event {
            Event::RedrawRequested(window_id) if window_id == render_target.window_id() => {
                match render_target.render(render_object) {
                    Ok(()) => (),
                    Err(SurfaceError::Lost) | Err(SurfaceError::Outdated) => {
                        render_target.refresh_surface()
                    }
                    Err(SurfaceError::OutOfMemory) => {
                        self.control_flow = ControlFlow::ExitWithCode(1)
                    }
                    Err(e) => eprintln!("{:#?}", e),
                }
            }
            Event::WindowEvent { window_id, event } if window_id == render_target.window_id() => {
                if let WindowEvent::Resized(new_size) = event {
                    render_target.resize_surface(new_size)
                } else {
                    self.handle_window_event(event)
                }
            }
            Event::MainEventsCleared => render_target.request_redraw(),
            _ => (),
        }
        self.control_flow
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => self.handle_keyboard_input(input),
            _ => (),
        }
    }

    fn handle_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => self.handle_key_down(code),
                ElementState::Released => self.handle_key_up(code),
            }
        }
    }

    fn handle_key_down(&mut self, code: VirtualKeyCode) {
        match code {
            VirtualKeyCode::Escape => self.control_flow = ControlFlow::Exit,
            _ => (),
        }
    }

    fn handle_key_up(&mut self, code: VirtualKeyCode) {
        match code {
            _ => (),
        }
    }
}
