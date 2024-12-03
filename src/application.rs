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

use crate::state::input::Input;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window, WindowId};

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
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::Math::ceil;

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

    pub fn resize(&mut self) {
        // We want to have ownership of the zoom level ourselves. We therefore disregard the zoom level and always render
        #[cfg(target_arch = "wasm32")]
        {
            let web_window = web_sys::window().expect("Window should exist");

            // Personal reminder to probably never use window's inner width. Visual viewport returns a float
            let viewport = web_window
                .visual_viewport()
                .expect("Visual viewport should exist");
            // viewport.set_onresize(|| self.) // TODO probably use this instead of resized because not everything sends resize event
            let width = viewport.width();
            let height = viewport.height();

            // let width = web_window
            //     .inner_width()
            //     .expect("Width should exist")
            //     .as_f64()
            //     .unwrap()
            //     .ceil(); // Does potentially cause inconsistent center but at least no white lines
            //              // .floor(); // Want to make sure we don't cause extra line to be drawn

            // let height = web_window
            //     .inner_height()
            //     .expect("Height should exist")
            //     .as_f64()
            //     .unwrap()
            //     .ceil();

            // .floor(); // Want to make sure we don't cause extra line to be drawn

            let _ = self
                .window
                .request_inner_size(LogicalSize::new(width, height));
        }
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

        // Note: This is more a logical size than a physical size. https://docs.rs/bevy/latest/bevy/window/struct.WindowResolution.html
        // For example: System scale or web zoom can change physical size, but not this value. (we could have a menu to change this though.)
        let mut initial_width = 0.0;
        let mut initial_height = 0.0;

        let window_attributes = Window::default_attributes()
            .with_title("Kloenk!")
            .with_inner_size(LogicalSize::new(initial_width, initial_height));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        log::warn!("{}", window.scale_factor());

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(monitor) = window.current_monitor() {
                let fullscreen_video_mode = monitor.video_modes().next().unwrap();
                let _ = window.request_inner_size(fullscreen_video_mode.size());
                window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
            }
        }

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
                    dst.append_child(&canvas).ok()?;
                    canvas.focus().expect("Unable to focus on canvas");
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
            // For web, canvas needs to exist before it can be resized
            let _ = window.request_inner_size(LogicalSize::new(initial_width, initial_height));
        }
        let renderer_future = Renderer::new(window.clone());

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
                    ui_state: UIState::new(),
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
                ui_state: UIState::new(),
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

        #[cfg(target_arch = "wasm32")]
        {
            let mut game = event.0;
            //

            //
            // let window_clone = game.window.clone();
            // // let mut renderer = &mut game.renderer;
            // let renderer_ref = Rc::new(RefCell::new(game.renderer));
            // // let window = &game.window;
            // let closure = Closure::wrap(Box::new(move || {
            //     let web_window = &web_sys::window().expect("Window should exist");
            //     let viewport = &web_window
            //         .visual_viewport()
            //         .expect("Visual viewport should exist");
            //     log::warn!("viewport resize");
            //     let viewport_width = viewport.width();
            //     let viewport_height = viewport.height();
            //     let logical_size = LogicalSize::new(viewport_width, viewport_height);
            //     let _ = window_clone.request_inner_size(logical_size);
            //
            //     // TODO make sure to disable resized event for web, cause this is how we handle resizing
            //     // TODO calculate physical size from viewport size? innersize is not yet correct i think
            //     let physical_size = logical_size.to_physical(web_window.device_pixel_ratio());
            //     renderer_ref.borrow_mut().resize(physical_size);
            //     // renderer.resize(PhysicalSize::new(3, 3)); // Web inner size request does not seem to lead to resized event, but also does not seem to immediately apply. Arbitrarily hope resize is done and apply resize here...
            //     //     //
            //     //     // // TODO we forgot about engine.renderer.resize...
            //     //     // // TODO how about just throwing a resized event lol we might have duplicate resized events???
            //     //     //
            //     //     // // let canvas = window.canvas().expect("Canvas should exist");
            //     //     // // canvas.set_width(viewport_width);
            //     //     // // canvas.set_height(viewport_height);
            // }) as Box<dyn FnMut()>);
            // // let viewport = web_sys::window()
            // //     .expect("Window should exist")
            // //     .visual_viewport()
            // //     .expect("Visual viewport should exist");
            // let web_window = &web_sys::window().expect("Window should exist");
            // let viewport = &web_window
            //     .visual_viewport()
            //     .expect("Visual viewport should exist");
            // viewport.set_onresize(Some(closure.as_ref().unchecked_ref()));
            // closure.forget();

            // game.renderer.resize(PhysicalSize::new(3, 3));

            // TODO probably use this instead of resized because not everything sends resize event

            // game.renderer.resize(game.window.inner_size()); // Web inner size request does not seem to lead to resized event, but also does not seem to immediately apply. Arbitrarily hope resize is done and apply resize here...
            game.window.request_redraw();
            log::warn!("{}", game.window.scale_factor());
            self.application_state = State::Initialized(game);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let game = event.0;
            game.window.request_redraw();
            log::warn!("{}", game.window.scale_factor());
            self.application_state = State::Initialized(game);
        }
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
                engine.input_handler.process_mouse_movement(
                    position,
                    engine.window.inner_size().width as f32,
                    engine.window.inner_size().height as f32,
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                engine.input_handler.process_scroll(&delta);
            }
            WindowEvent::Resized(physical_size) => {
                log::warn!("Resize event {:?}", physical_size);
                engine.resize();
                engine.renderer.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                engine.window().request_redraw();

                GameSystem::update(
                    &engine.window,
                    &mut engine.game_state,
                    &mut engine.ui_state,
                    &mut engine.input_handler,
                    &mut engine.frame_state,
                    &mut engine.audio_system,
                );

                match engine.renderer.render(
                    &engine.window,
                    &mut engine.game_state,
                    &engine.frame_state,
                ) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        engine.renderer.resize(engine.window.inner_size());
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
