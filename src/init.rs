use self::{events::EventHandler, render_target::RenderTarget};

pub mod events;
pub mod render_target;

pub async fn run() {
    env_logger::init();
    let (mut handler, event_loop) = EventHandler::create();
    let render_target = RenderTarget::new(&event_loop).await;

    event_loop.run(move |event, _, control_flow| *control_flow = handler.handle_event(event));
}
