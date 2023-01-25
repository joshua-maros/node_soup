use init::{
    old_renderer::{Size, VisualNode, VisualSocket},
    render_engine::RenderEngine,
};
use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod constants;
mod init;

pub struct App {
    window: Window,
    render_engine: RenderEngine,
    control_flow: ControlFlow,
}

impl App {
    pub async fn create_and_run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();
        let render_engine = RenderEngine::new_for_window(&window).await;
        App {
            window,
            render_engine,
            // This is overwritten whenever an event is received anyway.
            control_flow: ControlFlow::Wait,
        }
        .run(event_loop)
    }

    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            self.control_flow = *control_flow;
            self.on_event(event);
            *control_flow = self.control_flow;
        });
    }

    fn on_event(&mut self, event: Event<()>) {
        let node = VisualNode {
            sockets: vec![
                VisualSocket::new(VisualNode {
                    sockets: vec![VisualSocket::new(VisualNode { sockets: vec![] })],
                }),
                VisualSocket::new(VisualNode { sockets: vec![] }),
            ],
        };
        match event {
            Event::RedrawRequested(window_id) => match self.render_engine.render_node(&node) {
                Ok(()) => (),
                Err(SurfaceError::Lost) | Err(SurfaceError::Outdated) => {
                    self.render_engine.refresh_target()
                }
                Err(SurfaceError::OutOfMemory) => self.control_flow = ControlFlow::ExitWithCode(1),
                Err(e) => eprintln!("{:#?}", e),
            },
            Event::WindowEvent { window_id, event } => {
                if let WindowEvent::Resized(new_size) = event {
                    let new_size = Size {
                        width: new_size.width as _,
                        height: new_size.height as _,
                    };
                    self.render_engine.resize_target(new_size)
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

pub fn main() {
    pollster::block_on(App::create_and_run());
}
