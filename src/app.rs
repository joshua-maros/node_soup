mod on_event;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderer::RenderEngine;

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
}
