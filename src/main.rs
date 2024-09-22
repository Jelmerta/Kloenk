use std::env;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use winit::event_loop::EventLoop;
use kloenk::Application;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // No good generic solution yet for this. Folder should be copied during build process
    #[cfg(not(target_arch = "wasm32"))]
    {
        let out_dir = env::var("OUT_DIR").unwrap();
        let mut copy_options = CopyOptions::new();
        copy_options.overwrite = true;
        let mut paths_to_copy = Vec::new();
        paths_to_copy.push("resources/");
        copy_items(&paths_to_copy, out_dir, &copy_options).unwrap();
    }

    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut Application { render_state: None, game_state: None, ui_state: None, input_handler: None, surface_configured: false })
        .unwrap();
}
