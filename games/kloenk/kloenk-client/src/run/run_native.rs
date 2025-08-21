use crate::application::Application;
use winit::event_loop::EventLoop;

pub fn run() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("wgpu_hal::vulkan"), log::LevelFilter::Error)
        .init();

    let event_loop = EventLoop::new().expect("Couldn't create event loop");

    let mut application: Application = Application::new();
    event_loop.run_app(&mut application).expect("Failed to start running event loop");
}
