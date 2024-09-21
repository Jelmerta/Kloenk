#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{event::*, event_loop::EventLoop, keyboard::PhysicalKey};

use crate::game_system::GameSystem;
// use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
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
        log::warn!("lol{:?}", env::var("OUT_DIR"));
        println!("lol{:?}", env::var("OUT_DIR"));
        let mut copy_options = CopyOptions::new();
        copy_options.overwrite = true;
        let mut paths_to_copy = Vec::new();
        paths_to_copy.push("resources/");
        copy_items(&paths_to_copy, out_dir, &copy_options).unwrap();
    }

    let event_loop = EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_title("Kloenk!")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(800, 600));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("kloenk-wasm")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
        // Not sure why, but width/height not yet set... Do it again.
        let _ = window.request_inner_size(PhysicalSize::new(800, 600));
    }

    let mut state = render::State::new(&window).await;

    let mut game_state = game_state::GameState::new();
    let mut ui_state = gui::UIState::new();
    let mut input_handler = input::Input::new();

    let mut surface_configured = false;
    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => match event {
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    input_handler.update(key, state);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    input_handler.process_mouse_button(button, state);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    input_handler.process_scroll(delta);
                    true;
                }
                WindowEvent::Resized(physical_size) => {
                    surface_configured = true;
                    state.resize(*physical_size);
                }
                WindowEvent::RedrawRequested => {
                    state.window().request_redraw();

                    // Make sure the window/surface is configured such that config
                    // contains right information such as width and height
                    // before rendering
                    if !surface_configured {
                        return;
                    }
                    GameSystem::update(&mut game_state, &mut ui_state, &mut input_handler);
                    match state.render(&game_state, &ui_state) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            control_flow.exit();
                        }

                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }
                }

                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
