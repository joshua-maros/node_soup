mod on_event;
mod render;

use std::collections::{HashMap, HashSet};

use maplit::hashset;
use renderer::{Position, RenderEngine};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    engine::{BuiltinDefinitions, Engine, NodeId, ParameterId},
    widgets::{BoundingBox, BoundingBoxKind, ValueWidget},
};

#[derive(Clone, Debug)]
pub enum DragTarget {
    Parameter(ParameterId),
}

pub struct App {
    window: Window,
    render_engine: RenderEngine,
    control_flow: ControlFlow,
    computation_engine: Engine,
    builtins: BuiltinDefinitions,
    preview_drawer_size: f32,
    root_bbox: BoundingBox,
    selected_node_path: Vec<NodeId>,
    previous_mouse_pos: Position,
    hovering: Option<BoundingBoxKind>,
    dragging: Option<BoundingBoxKind>,
}

impl App {
    pub async fn create_and_run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();
        let render_engine = RenderEngine::new_for_window(&window).await;
        let (computation_engine, builtins) = Engine::new();
        App {
            window,
            render_engine,
            // This is overwritten whenever an event is received anyway.
            control_flow: ControlFlow::Wait,
            computation_engine,
            builtins,
            preview_drawer_size: 200.0,
            root_bbox: BoundingBox::new_start_end(
                Position::zero(),
                Position::zero(),
                BoundingBoxKind::Unused,
            ),
            selected_node_path: vec![],
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

    fn on_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.on_mouse_down(button),
            ElementState::Released => self.on_mouse_up(button),
        }
    }

    fn on_mouse_down(&mut self, button: MouseButton) {
        if button == MouseButton::Left {
            self.dragging = self.hovering.clone();
        }
    }

    fn on_mouse_up(&mut self, button: MouseButton) {
        if button == MouseButton::Left {
            if let Some(BoundingBoxKind::SelectNode(index, node)) = self.dragging {
                assert!(index <= self.selected_node_path.len());
                self.selected_node_path.resize(index, node);
                self.selected_node_path.push(node);
                assert_eq!(self.selected_node_path.last(), Some(&node));
            }
            self.dragging = None;
        }
    }
}
