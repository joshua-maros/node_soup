use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::App;
use crate::{
    renderer::{Position, RenderEngine},
    visuals::{Node, Socket},
};

impl App {
    pub(super) fn on_event(&mut self, event: Event<()>) {
        let node = Node {
            sockets: vec![
                Socket::new(Node {
                    sockets: vec![Socket::new(Node { sockets: vec![] })],
                }),
                Socket::new(Node { sockets: vec![] }),
            ],
        };
        match event {
            Event::RedrawRequested(_window_id) => match self
                .render_engine
                .render(&node.draw(Position { x: 0.0, y: 0.0 }))
            {
                Ok(()) => (),
                Err(SurfaceError::Lost) | Err(SurfaceError::Outdated) => {
                    self.render_engine.refresh_target()
                }
                Err(SurfaceError::OutOfMemory) => self.control_flow = ControlFlow::ExitWithCode(1),
                Err(e) => eprintln!("{:#?}", e),
            },
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

    fn on_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_input(input),
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
}
