use crate::application::{Application, CustomEvent};
// use console_log::init_with_level;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

/// # Panics
pub fn run() {
    // std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    // init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    let event_loop = EventLoop::<CustomEvent>::with_user_event().build().unwrap();

    let application: Application = Application::new(&event_loop);
    event_loop.spawn_app(application);
}
