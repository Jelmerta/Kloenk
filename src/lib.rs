use winit::application::ApplicationHandler;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::{event::*, keyboard::PhysicalKey};

use crate::game_system::GameSystem;
// use anyhow::*;
use crate::game_state::GameState;
use crate::gui::UIState;
use crate::input::Input;
use crate::render::Renderer;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

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

pub struct Game {
    pub renderer: Renderer,
    pub game_state: GameState,
    pub ui_state: UIState,
    pub input_handler: Input,
    pub window: Arc<Window>,
}

impl Game {
    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }
}

pub struct StateInitializationEvent(Game);

pub struct Application {
    application_state: ApplicationState,
    event_loop_proxy: EventLoopProxy<StateInitializationEvent>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<StateInitializationEvent>) -> Application {
        Application {
            application_state: ApplicationState::Uninitialized,
            event_loop_proxy: event_loop.create_proxy(),
        }
    }
}
pub enum ApplicationState {
    Uninitialized,
    Initializing,
    Initialized(Game),
}

impl ApplicationHandler<StateInitializationEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.application_state {
            ApplicationState::Initialized(_) => return,
            ApplicationState::Initializing => return,
            ApplicationState::Uninitialized => {
                self.application_state = ApplicationState::Initializing
            } // Continue
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

        let renderer_future = Renderer::new(window.clone());

        #[cfg(target_arch = "wasm32")]
        {
            let event_loop_proxy = self.event_loop_proxy.clone();
            spawn_local(async move {
                let renderer = renderer_future.await;

                let game = Game {
                    renderer,
                    game_state: GameState::new(),
                    ui_state: UIState::new(),
                    input_handler: Input::new(),
                    window,
                };

                event_loop_proxy
                    .send_event(StateInitializationEvent(game))
                    .unwrap_or_else(|_| {
                        panic!("Failed to send initialization event");
                    });
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = pollster::block_on(renderer_future);
            let game = Game {
                renderer,
                game_state: GameState::new(),
                ui_state: UIState::new(),
                input_handler: Input::new(),
                window,
            };

            self.event_loop_proxy
                .send_event(StateInitializationEvent(game))
                .unwrap_or_else(|_| {
                    panic!("Failed to send initialization event");
                });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: StateInitializationEvent) {
        log::info!("Received initialization event");

        let game = event.0;
        game.window.request_redraw();
        self.application_state = ApplicationState::Initialized(game);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ApplicationState::Initialized(ref mut game) = self.application_state else {
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
                game.input_handler.update(&key, &state);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                game.input_handler.process_mouse_button(&button, &state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                game.input_handler.process_scroll(&delta);
                true;
            }
            WindowEvent::Resized(physical_size) => {
                game.renderer.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                game.window().request_redraw();

                GameSystem::update(
                    &mut game.game_state,
                    &mut game.ui_state,
                    &mut game.input_handler,
                );
                match game.renderer.render(&game.game_state, &game.ui_state) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        game.renderer.resize(game.renderer.size)
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
