use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

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

    pub fn handle_event(&mut self, event: Event<()>) -> ControlFlow {
        self.control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { window_id, event } => self.handle_window_event(event),
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
