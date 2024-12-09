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
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, CustomCursor, Fullscreen, Icon, Window, WindowId};

use crate::cursor_manager::CursorManager;
use crate::render::render::Renderer;
use crate::resources::load_binary;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::systems::audio_system::AudioSystem;
use crate::systems::game_system::GameSystem;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::Math::ceil;
use winit::keyboard::KeyCode;

pub struct Engine {
    pub renderer: Renderer,
    pub game_state: GameState,
    pub ui_state: UIState,
    pub input_handler: Input,
    pub frame_state: FrameState,
    pub window: Arc<Window>,

    pub audio_state: AudioState,
}

// On web, AudioSystem is loaded after user has used a gesture. This is to get rid of this warning in Chrome:
// The AudioContext was not allowed to start. It must be resumed (or created) after a user gesture on the page. https://goo.gl/7K7WLu
pub enum AudioState {
    NotLoaded,
    Loading,
    Loaded(AudioSystem),
}

impl Engine {
    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }
}

pub enum CustomEvent {
    StateInitializationEvent(Engine),
    #[cfg(target_arch = "wasm32")]
    WebResizedEvent, // Only sent on web when window gets resized. Resized event only seems to send event on dpi change (browser zoom or system scale ratio) which is insufficient
    #[cfg(target_arch = "wasm32")]
    AudioStateChanged(AudioState),
}
pub struct Application {
    application_state: State,
    event_loop_proxy: EventLoopProxy<CustomEvent>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<CustomEvent>) -> Application {
        Application {
            application_state: State::Uninitialized,
            event_loop_proxy: event_loop.create_proxy(),
        }
    }

    fn load_audio_player(event_loop_proxy: &EventLoopProxy<CustomEvent>, engine: &mut Engine) {
        #[cfg(target_arch = "wasm32")]
        {
            let mut new_loading_state = None;
            match engine.audio_state {
                AudioState::NotLoaded => {
                    new_loading_state = Some(AudioState::Loading);
                    let event_loop_proxy_clone = event_loop_proxy.clone();
                    spawn_local(async move {
                        let audio_system = AudioSystem::new().await;

                        event_loop_proxy_clone
                            .send_event(CustomEvent::AudioStateChanged(AudioState::Loaded(
                                audio_system,
                            )))
                            .unwrap_or_else(|_| {
                                panic!("Failed to send audio state event");
                            });
                    });
                }
                _ => (),
            }
            if new_loading_state.is_some() {
                engine.audio_state = new_loading_state.unwrap();
            }
        }
    }
}

pub enum State {
    Uninitialized,
    Initializing,
    Initialized(Engine),
}

impl ApplicationHandler<CustomEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.application_state {
            State::Initializing | State::Initialized(_) => return,
            State::Uninitialized => {
                self.application_state = State::Initializing;
            } // Continue
        }

        // Note: This is more of a logical size than a physical size. https://docs.rs/bevy/latest/bevy/window/struct.WindowResolution.html
        // For example: System scale or web zoom can change physical size, but not this value. (we could have a menu to change this though.)
        // We want to have ownership of the zoom level ourselves. We therefore disregard the dpi ratio and always attempt to render the same image
        #[allow(unused_mut)]
        let mut initial_width = 0.0;
        #[allow(unused_mut)]
        let mut initial_height = 0.0;
        #[cfg(target_arch = "wasm32")]
        {
            let web_window = web_sys::window().expect("Window should exist");
            // Personal reminder to probably never use window's inner width. Visual viewport returns a float
            let viewport = &web_window
                .visual_viewport()
                .expect("Visual viewport should exist");
            initial_width = viewport.width();
            initial_height = viewport.height();

            // does clone work?
            let event_loop_proxy = self.event_loop_proxy.clone();
            let closure = Closure::wrap(Box::new(move || {
                event_loop_proxy
                    .send_event(CustomEvent::WebResizedEvent)
                    .unwrap_or_else(|_| {
                        panic!("Failed to send Web resize event");
                    });
            }) as Box<dyn FnMut()>);

            let web_window = &web_sys::window().expect("Window should exist");
            let viewport = &web_window
                .visual_viewport()
                .expect("Visual viewport should exist");
            viewport.set_onresize(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        let window_attributes;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let cursor_binary = pollster::block_on(load_binary("cursor.png")).unwrap();
            let cursor_rgba = image::load_from_memory(&cursor_binary)
                .unwrap()
                .to_rgba8()
                .into_raw();
            let custom_cursor_source =
                CustomCursor::from_rgba(cursor_rgba, 122, 120, 7, 7).unwrap();
            let custom_cursor = event_loop.create_custom_cursor(custom_cursor_source);

            let window_icon_binary = pollster::block_on(load_binary("kunst.png")).unwrap();
            let window_icon_rgba = image::load_from_memory(&window_icon_binary)
                .unwrap()
                .to_rgba8()
                .into_raw();
            let window_icon = Some(Icon::from_rgba(window_icon_rgba, 64, 64).unwrap());

            window_attributes = Window::default_attributes()
                .with_title("Kloenk!")
                .with_inner_size(LogicalSize::new(initial_width, initial_height))
                .with_active(true)
                .with_cursor(Cursor::Custom(custom_cursor))
                .with_window_icon(window_icon);
        }

        #[cfg(target_arch = "wasm32")]
        {
            window_attributes = Window::default_attributes()
                .with_title("Kloenk!")
                .with_inner_size(LogicalSize::new(initial_width, initial_height))
                .with_active(true);
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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
        }
        let renderer_future = Renderer::new(window.clone());

        #[cfg(target_arch = "wasm32")]
        {
            let event_loop_proxy = self.event_loop_proxy.clone();

            spawn_local(async move {
                let renderer = renderer_future.await;
                let cursor = CursorManager::load_cursor_future().await;

                let game = Engine {
                    renderer,
                    game_state: GameState::new(),
                    ui_state: UIState::new(cursor),
                    input_handler: Input::new(),
                    frame_state: FrameState::new(),
                    audio_state: AudioState::NotLoaded,
                    window,
                };

                event_loop_proxy
                    .send_event(CustomEvent::StateInitializationEvent(game))
                    .unwrap_or_else(|_| {
                        panic!("Failed to send initialization event");
                    });
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = pollster::block_on(renderer_future);
            let audio_system = pollster::block_on(AudioSystem::new());
            let cursor = pollster::block_on(CursorManager::load_cursor_future());

            let game = Engine {
                renderer,
                game_state: GameState::new(),
                ui_state: UIState::new(cursor),
                input_handler: Input::new(),
                frame_state: FrameState::new(),
                window,
                audio_state: AudioState::Loaded(audio_system),
            };

            self.event_loop_proxy
                .send_event(CustomEvent::StateInitializationEvent(game))
                .unwrap_or_else(|_| {
                    panic!("Failed to send initialization event");
                });
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CustomEvent) {
        match event {
            CustomEvent::StateInitializationEvent(mut engine) => {
                log::info!("Received initialization event");
                engine.renderer.resize(engine.window.inner_size()); // Web inner size request does not seem to lead to resized event, but also does not seem to immediately apply. Arbitrarily hope resize is done and apply resize here...
                engine.window.set_cursor(event_loop.create_custom_cursor(
                    CursorManager::load_cursor(engine.ui_state.cursor.clone()),
                ));
                engine.window.request_redraw();
                self.application_state = State::Initialized(engine);
            }
            #[cfg(target_arch = "wasm32")]
            CustomEvent::WebResizedEvent => {
                let State::Initialized(ref mut engine) = self.application_state else {
                    return;
                };

                let web_window = web_sys::window().expect("Window should exist");
                let viewport = &web_window
                    .visual_viewport()
                    .expect("Visual viewport should exist");
                let viewport_width = viewport.width();
                let viewport_height = viewport.height();
                let logical_size = LogicalSize::new(viewport_width, viewport_height);
                let _ = engine.window.request_inner_size(logical_size);

                let physical_size = logical_size.to_physical(web_window.device_pixel_ratio());
                engine.renderer.resize(physical_size);
            }
            #[cfg(target_arch = "wasm32")]
            CustomEvent::AudioStateChanged(audio_state) => match self.application_state {
                State::Uninitialized => {
                    panic!("Expected application to be loaded")
                }
                State::Initializing => {
                    panic!("Expected application to be loaded")
                }
                State::Initialized(ref mut engine) => {
                    engine.audio_state = audio_state;
                }
            },
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

                // Loading audio only after user has gestured
                // Thought of callback or observer pattern but that honestly seems way too complex compared to this.
                if key_is_gesture(key) {
                    Self::load_audio_player(&self.event_loop_proxy, engine);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                engine.input_handler.process_mouse_button(button, state);

                // Loading audio only after user has gestured
                // Thought of callback or observer pattern but that honestly seems way too complex compared to this.
                Self::load_audio_player(&self.event_loop_proxy, engine);
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
            // // Web uses custom resize event as web only sends event on dpi changes
            WindowEvent::Resized(physical_size) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    engine.renderer.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                engine.window().request_redraw();

                GameSystem::update(
                    &engine.window,
                    &mut engine.game_state,
                    &mut engine.ui_state,
                    &mut engine.input_handler,
                    &mut engine.frame_state,
                    &mut engine.audio_state,
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

fn key_is_gesture(key: KeyCode) -> bool {
    match key {
        KeyCode::AltLeft
        | KeyCode::AltRight
        | KeyCode::ControlLeft
        | KeyCode::ControlRight
        | KeyCode::CapsLock
        | KeyCode::ShiftLeft
        | KeyCode::ShiftRight
        | KeyCode::Fn
        | KeyCode::SuperLeft
        | KeyCode::SuperRight
        | KeyCode::Escape
        | KeyCode::Meta
        | KeyCode::Hyper => false,
        _ => true,
    }
}
