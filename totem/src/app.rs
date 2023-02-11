mod on_event;
mod render;

use renderer::{
    winit::{ControlFlow, EventLoop, PhysicalSize, Window, WindowBuilder},
    Position, RenderEngine,
};

use crate::{
    engine::{BuiltinDefinitions, Engine, NodeId, ParameterId},
    widgets::{BoundingBox, BoundingBoxKind},
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
    root_bbox: BoundingBox,
    selected_node_path: Vec<NodeId>,
    previous_mouse_pos: Position,
    hovering: Option<BoundingBoxKind>,
    dragging: Option<BoundingBoxKind>,
    tool_targets: Vec<(ParameterId, NodeId)>,
    collapse_to_literal: Option<(NodeId, NodeId)>,
}

impl App {
    pub async fn create_and_run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();
        let render_engine = RenderEngine::new_for_window(&window).await;
        render_engine.upload_image(0, &[[255; 4]; 360 * 360]);
        let (computation_engine, builtins) = Engine::new();
        let selected_node_path = vec![computation_engine.root_node()];
        App {
            window,
            render_engine,
            // This is overwritten whenever an event is received anyway.
            control_flow: ControlFlow::Wait,
            computation_engine,
            builtins,
            root_bbox: BoundingBox::new_start_end(
                Position::zero(),
                Position::zero(),
                BoundingBoxKind::Unused,
            ),
            selected_node_path,
            previous_mouse_pos: Position { x: 0.0, y: 0.0 },
            hovering: None,
            dragging: None,
            tool_targets: vec![],
            collapse_to_literal: None,
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
