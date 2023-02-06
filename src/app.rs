mod on_event;
mod render;

use std::collections::HashMap;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    engine::{Engine, ParameterId},
    renderer::{Position, RenderEngine},
    visuals::ValueWidget,
};

pub enum DragTarget {
    Parameter(ParameterId)
}

pub struct App {
    window: Window,
    render_engine: RenderEngine,
    control_flow: ControlFlow,
    computation_engine: Engine,
    preview_drawer_size: f32,
    parameter_widgets: HashMap<ParameterId, Box<dyn ValueWidget>>,
    previous_mouse_pos: Position,
    hovering: Option<DragTarget>,
    dragging: Option<DragTarget>,
}

impl App {
    pub async fn create_and_run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();
        let render_engine = RenderEngine::new_for_window(&window).await;
        let computation_engine = Engine::new();
        App {
            window,
            render_engine,
            // This is overwritten whenever an event is received anyway.
            control_flow: ControlFlow::Wait,
            computation_engine,
            preview_drawer_size: 200.0,
            parameter_widgets: HashMap::new(),
            previous_mouse_pos: Position { x: 0.0, y: 0.0 },
            hovering: None,
            dragging: None,
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
