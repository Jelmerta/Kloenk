use crate::application::{Application, StateInitializationEvent};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use winit::event_loop::EventLoop;
mod application;
mod audio_system;
mod camera;
mod components;
mod game_state;
mod game_system;
mod gui;
mod input;
mod model;
mod render;
mod resources;
mod text_renderer;
mod texture;

fn main() {
    run();
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(start)]
    pub fn run() {
        crate::run();
    }
}

/// # Panics
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::builder()
            .filter(None, log::LevelFilter::Warn)
            .init();
        }
    }

    // // No good generic solution yet for this. Folder should be copied during build process
    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     let out_dir = env::var("OUT_DIR").unwrap();
    //     let mut copy_options = CopyOptions::new();
    //     copy_options.overwrite = true;
    //     let paths_to_copy = vec!["resources/"];
    //     copy_items(&paths_to_copy, out_dir, &copy_options).unwrap();
    // }

    let event_loop = EventLoop::<StateInitializationEvent>::with_user_event()
        .build()
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        let application: Application = Application::new(&event_loop);
        event_loop.spawn_app(application);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut application: Application = Application::new(&event_loop);
        event_loop.run_app(&mut application).unwrap();
    }
}
