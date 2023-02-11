mod on_event;
mod render;

use std::time::Duration;

use renderer::{
    winit::{ControlFlow, EventLoop, PhysicalSize, Window, WindowBuilder},
    Position, RenderEngine,
};

use crate::{
    engine::{BuiltinDefinitions, Engine, NodeId, ParameterId},
    widgets::{BoundingBox, BoundingBoxKind},
};

pub struct PerfCounters {
    pub compilation_time_acc: Duration,
    pub execution_time_acc: Duration,
    pub upload_time_acc: Duration,
    pub gpu_time_acc: Duration,
    pub total_time_acc: Duration,
    pub samples: u32,
}

impl PerfCounters {
    pub fn new() -> Self {
        Self {
            compilation_time_acc: Duration::ZERO,
            execution_time_acc: Duration::ZERO,
            upload_time_acc: Duration::ZERO,
            gpu_time_acc: Duration::ZERO,
            total_time_acc: Duration::ZERO,
            samples: 0,
        }
    }

    pub fn report_and_reset_if_appropriate(&mut self) {
        if self.execution_time_acc + self.upload_time_acc + self.gpu_time_acc > Duration::new(1, 0)
        {
            let c = self.compilation_time_acc.as_millis() as f32 / self.samples as f32;
            let e = self.execution_time_acc.as_millis() as f32 / self.samples as f32;
            let u = self.upload_time_acc.as_millis() as f32 / self.samples as f32;
            let g = self.gpu_time_acc.as_millis() as f32 / self.samples as f32;
            let t = self.total_time_acc.as_millis() as f32 / self.samples as f32;
            println!(
                "({} samples)\n  compilation: {:.02}ms\n  execution: {:.02}ms\n  upload: {:.02}ms\n  gpu: {:.02}ms\n  total: {:.02}ms",
                self.samples, c, e, u, g,t
            );
            self.samples = 0;
            self.compilation_time_acc = Duration::ZERO;
            self.execution_time_acc = Duration::ZERO;
            self.upload_time_acc = Duration::ZERO;
            self.gpu_time_acc = Duration::ZERO;
            self.total_time_acc = Duration::ZERO;
        }
    }
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
    perf_counters: PerfCounters,
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
            perf_counters: PerfCounters::new(),
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
