use winit::{event::*, keyboard::PhysicalKey};

use crate::game_system::GameSystem;
// use anyhow::*;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::game_state::GameState;
use crate::gui::UIState;
use crate::input::Input;
use crate::render::RenderState;

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

pub struct Application {
    pub render_state: Option<RenderState>,
    pub game_state: Option<GameState>,
    pub ui_state: Option<UIState>,
    pub input_handler: Option<Input>,
    pub surface_configured: bool,
}

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_state.is_some() { // Assumes other application state is empty as well
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Kloenk!")
            .with_inner_size(LogicalSize::new(800.0, 600.0));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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

        self.render_state = Some(pollster::block_on(RenderState::new(window)));
        self.game_state = Some(GameState::new());
        self.ui_state = Some(UIState::new());
        self.input_handler = Some(Input::new());
        self.surface_configured = false;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(render_state) = &mut self.render_state else {
            return;
        };

        let Some(game_state) = &mut self.game_state else {
            return;
        };

        let Some(ui_state) = &mut self.ui_state else {
            return;
        };

        let Some(input_handler) = &mut self.input_handler else {
            return;
        };

        match event {
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
            } => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state,
                    ..
                },
                ..
            } => {
                input_handler.update(&key, &state);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                input_handler.process_mouse_button(&button, &state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                input_handler.process_scroll(&delta);
                true;
            }
            WindowEvent::Resized(physical_size) => {
                self.surface_configured = true;
                render_state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                render_state.window().request_redraw();

                // Make sure the window/surface is configured such that config
                // contains right information such as width and height
                // before rendering
                if !self.surface_configured {
                    return;
                }
                GameSystem::update(game_state, ui_state, input_handler);
                match render_state.render(&game_state, &ui_state) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        render_state.resize(render_state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                    }

                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            _ => {}
        }
    }
}
