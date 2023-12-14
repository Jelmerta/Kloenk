#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    event::*,
    event_loop::{EventLoop},
    window::WindowBuilder,
};
use winit::dpi::PhysicalSize;
use winit::keyboard::{Key, NamedKey};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Kloenk")
        .with_inner_size(PhysicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let mut close_requested = false;

    event_loop.run(move |event, event_loop_window_target|
        match event {
            Event::WindowEvent {
                ref event,
                window_id: _window_id,
            } => match event { //if window_id == window.id() =>
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => close_requested = true,
                _ => {}
            },
            Event::AboutToWait => {
                if close_requested {
                    event_loop_window_target.exit();
                }
            }
            _ => {}
        }
    ).unwrap();

    #[cfg(target_arch="wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                let wasm_element = document.get_element_by_id("kloenk-wasm")?;
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                wasm_element.append_child(&canvas).ok()?;
                Some(()) // What the hell does this do???
            })
            .expect("Couldn't append canvas to document body");
    }
}


