use crate::application::{Application, CustomEvent};
#[cfg(feature = "debug-logging")]
use console_log::init_with_level;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

/// # Panics
pub fn run() {
    #[cfg(feature = "debug-logging")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
    }

    // TODO maybe check and handle this case:
    // wgpu::util::is_browser_webgpu_supported()

    let event_loop = EventLoop::<CustomEvent>::with_user_event()
        .build()
        .expect("Couldn't build event loop");

    let application: Application = Application::new(&event_loop);
    event_loop.spawn_app(application);
}
