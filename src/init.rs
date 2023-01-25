use self::{
    events::EventHandler,
    old_renderer::{RenderTarget, VisualNode, VisualSocket},
};

pub mod events;
pub mod old_renderer;
pub mod render_device;
mod render_target;
pub mod uniform_buffer;
pub mod render_engine;
pub mod pipeline_util;

pub async fn run() {
    env_logger::init();
    let (mut handler, event_loop) = EventHandler::create();
    let mut render_target = RenderTarget::new(&event_loop).await;

    let node = VisualNode {
        sockets: vec![
            VisualSocket::new(VisualNode {
                sockets: vec![VisualSocket::new(VisualNode { sockets: vec![] })],
            }),
            VisualSocket::new(VisualNode { sockets: vec![] }),
        ],
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = handler.handle_event(event, &mut render_target, &node)
    });
}
