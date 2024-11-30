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
use crate::state::input::Input;
use std::sync::Arc;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use crate::systems::audio_system::AudioPlayer;
use crate::systems::audio_system::AudioSystem;

use crate::render::render::Renderer;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::systems::game_system::GameSystem;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub struct Engine {
    pub renderer: Renderer,
    pub game_state: GameState,
    pub ui_state: UIState,
    pub input_handler: Input,
    pub frame_state: FrameState,
    pub window: Arc<Window>,
    // AudioSystem is loaded after user has used a gesture. This is to get rid of this warning in Chrome:
    // The AudioContext was not allowed to start. It must be resumed (or created) after a user gesture on the page. https://goo.gl/7K7WLu
    #[cfg(target_arch = "wasm32")]
    pub audio_loading_state: Rc<RefCell<AudioState>>,
    pub audio_system: AudioSystem,
}

#[cfg(target_arch = "wasm32")]
pub enum AudioState {
    NotLoaded,
    Loading,
    Loaded,
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

        let window_width: u32 = 1920;
        let window_height: u32 = 1080;

        let window_attributes = Window::default_attributes()
            .with_title("Kloenk!")
            .with_inner_size(LogicalSize::new(window_width as f32, window_height as f32));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("kloenk-wasm")?;
                    let canvas = window.canvas()?;
                    canvas
                        .set_attribute("tabindex", "0")
                        .expect("failed to set tabindex");
                    canvas.set_width(window_width);
                    canvas.set_height(window_height);
                    dst.append_child(&canvas).ok()?;
                    canvas.focus().expect("Unable to focus on canvas");
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let renderer_future = Renderer::new(window.clone(), window_width, window_height);

        #[cfg(target_arch = "wasm32")]
        {
            let event_loop_proxy = self.event_loop_proxy.clone();
            let audio_future = AudioSystem::new();
            spawn_local(async move {
                let renderer = renderer_future.await;
                let audio_system = audio_future.await;

                let game = Engine {
                    renderer,
                    game_state: GameState::new(),
                    ui_state: UIState::new(window_width, window_height),
                    input_handler: Input::new(),
                    frame_state: FrameState::new(),
                    audio_loading_state: Rc::new(RefCell::new(AudioState::NotLoaded)),
                    audio_system,
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
            let audio_system = pollster::block_on(AudioSystem::new());

            let game = Engine {
                renderer,
                game_state: GameState::new(),
                ui_state: UIState::new(window_width, window_height),
                input_handler: Input::new(),
                frame_state: FrameState::new(),
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
        log::info!("voor {:?}", game.window.inner_size());
        let _ = game.window.request_inner_size(PhysicalSize::new(
            game.ui_state.window_size.width,
            game.ui_state.window_size.height,
        ));
        log::info!("na {:?}", game.window.inner_size());
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

                // Thought of callback or observer pattern but that honestly seems way too complex compared to this.
                #[cfg(target_arch = "wasm32")]
                {
                    let loading_state_clone = engine.audio_loading_state.clone();
                    let mut loading_state_mut = engine.audio_loading_state.borrow_mut();
                    match *loading_state_mut {
                        AudioState::NotLoaded => {
                            *loading_state_mut = AudioState::Loading;
                            let audio_player = engine.audio_system.audio_player.clone();
                            let audio_binaries = engine.audio_system.sounds.clone();
                            spawn_local(async move {
                                let mut ref_mut = loading_state_clone.borrow_mut();
                                let mut audio_player_mut = audio_player.borrow_mut();
                                *audio_player_mut =
                                    Some(AudioPlayer::build_audio_player(&audio_binaries).await);
                                *ref_mut = AudioState::Loaded;
                            });
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                engine.input_handler.process_mouse_button(button, state);

                // Loading audio only after user has gestured
                // Thought of callback or observer pattern but that honestly seems way too complex compared to this.
                #[cfg(target_arch = "wasm32")]
                {
                    let loading_state_clone = engine.audio_loading_state.clone();
                    let mut loading_state_mut = engine.audio_loading_state.borrow_mut();
                    match *loading_state_mut {
                        AudioState::NotLoaded => {
                            *loading_state_mut = AudioState::Loading;
                            let audio_player = engine.audio_system.audio_player.clone();
                            let audio_binaries = engine.audio_system.sounds.clone();
                            spawn_local(async move {
                                let mut ref_mut = loading_state_clone.borrow_mut();
                                let mut audio_player_mut = audio_player.borrow_mut();
                                *audio_player_mut =
                                    Some(AudioPlayer::build_audio_player(&audio_binaries).await);
                                *ref_mut = AudioState::Loaded;
                            });
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let window_size = &engine.ui_state.window_size;
                engine.input_handler.process_mouse_movement(
                    position,
                    window_size.width as f32,
                    window_size.height as f32,
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                engine.input_handler.process_scroll(&delta);
            }
            WindowEvent::Resized(physical_size) => {
                log::info!("{:?}", physical_size);
                engine.renderer.resize(physical_size);
                engine
                    .ui_state
                    .set_window_size(physical_size.width, physical_size.height);
            }
            WindowEvent::RedrawRequested => {
                engine.window().request_redraw();

                GameSystem::update(
                    &mut engine.game_state,
                    &mut engine.ui_state,
                    &mut engine.input_handler,
                    &mut engine.frame_state,
                    &mut engine.audio_system,
                );

                match engine.renderer.render(
                    &mut engine.game_state,
                    &engine.ui_state,
                    &engine.frame_state,
                ) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        engine.renderer.resize(engine.renderer.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory");
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
