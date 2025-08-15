use crate::application::Application;
use winit::event_loop::EventLoop;

pub fn run() {
    // Needed for logging, maybe make a dev flag
    env_logger::builder()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("wgpu_hal::vulkan"), log::LevelFilter::Error)
        .init();

    let event_loop = EventLoop::new().unwrap();

    let mut application: Application = Application::new();
    event_loop.run_app(&mut application).unwrap();
}
