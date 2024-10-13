#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::PhysicalKey,
};

#[cfg(not(target_arch = "wasm32"))]
use winit::event::ElementState;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

// use anyhow::*;
use crate::game_state::GameState;
use crate::game_system::GameSystem;
use crate::gui::UIState;
use crate::input::Input;
use crate::render::Renderer;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::audio_system::AudioSystem;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub struct Engine {
    pub renderer: Renderer,
    pub game_state: GameState,
    pub ui_state: UIState,
    pub input_handler: Input,
    pub window: Arc<Window>,
    #[cfg(target_arch = "wasm32")]
    pub audio_system: Rc<RefCell<Option<AudioSystem>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub audio_system: AudioSystem,
}

impl Engine {
    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }
}

pub struct StateInitializationEvent(Engine);

pub struct Application {
    application_state: State,
    event_loop_proxy: EventLoopProxy<StateInitializationEvent>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<StateInitializationEvent>) -> Application {
        Application {
            application_state: State::Uninitialized,
            event_loop_proxy: event_loop.create_proxy(),
        }
    }
}
pub enum State {
    Uninitialized,
    Initializing,
    Initialized(Engine),
}

impl ApplicationHandler<StateInitializationEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.application_state {
            State::Initializing | State::Initialized(_) => return,
            State::Uninitialized => {
                self.application_state = State::Initializing;
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

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("kloenk-wasm")?;
                    let canvas = window.canvas()?;
                    canvas
                        .set_attribute("tabindex", "0")
                        .expect("failed to set tabindex");
                    dst.append_child(&canvas).ok()?;
                    canvas.focus().expect("Unable to focus on canvas");
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

                let game = Engine {
                    renderer,
                    game_state: GameState::new(),
                    ui_state: UIState::new(),
                    input_handler: Input::new(),
                    window,
                    audio_system: Rc::new(RefCell::new(None)),
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
            let audio_system = Some(pollster::block_on(AudioSystem::new()));

            let game = Engine {
                renderer,
                game_state: GameState::new(),
                ui_state: UIState::new(),
                input_handler: Input::new(),
                window,
                audio_system,
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
        self.application_state = State::Initialized(game);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let State::Initialized(ref mut engine) = self.application_state else {
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
                engine.input_handler.update(key, state);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                engine.input_handler.process_mouse_button(button, state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                engine.input_handler.process_scroll(&delta);
            }
            WindowEvent::Resized(physical_size) => {
                engine.renderer.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                engine.window().request_redraw();

                // TODO make sure run once
                #[cfg(target_arch = "wasm32")]
                {
                    let audio_system_clone = engine.audio_system.clone();
                    let has_gestured = engine.input_handler.user_has_gestured.clone();
                    if engine.audio_system.borrow().is_none() {
                        spawn_local(async move {
                            let mut audio_system = audio_system_clone.borrow_mut();
                            if audio_system.is_none() && has_gestured {
                                *audio_system = Some(AudioSystem::new().await);
                            }
                        });
                    }
                }

                #[cfg(not(target_arch = "wasm32"))]
                if engine.audio_system.is_none() {
                    engine.audio_system = Some(pollster::block_on(AudioSystem::new()));
                }

                if engine.audio_system.borrow().is_some() {
                    GameSystem::update(
                        &mut engine.game_state,
                        &mut engine.ui_state,
                        &mut engine.input_handler,
                        &mut engine.audio_system.borrow_mut(),
                    );
                } else {
                    GameSystem::update(
                        &mut engine.game_state,
                        &mut engine.ui_state,
                        &mut engine.input_handler,
                        &mut None,
                    );
                }

                match engine.renderer.render(&engine.game_state, &engine.ui_state) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        engine.renderer.resize(engine.renderer.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                    }

                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout");
                    }
                }
            }
            _ => {}
        }
    }
}
