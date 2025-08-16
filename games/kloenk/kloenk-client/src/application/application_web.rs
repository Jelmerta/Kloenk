use winit::application::ApplicationHandler;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::PhysicalKey,
};

use winit::platform::web::{CustomCursorExtWebSys, WindowExtWebSys};

use crate::state::input::Input;
use hydrox::AudioSystem;
use std::sync::Arc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CustomCursor, Window, WindowId};

use crate::application::application::Asset::{Color, Texture, Vertices};
use crate::application::{AssetLoader, ImageAsset};
use crate::render::model::ColorDefinition;
use crate::render::model_loader::ModelLoader;
use crate::render::primitive_vertices_manager::PrimitiveVertices;
use crate::render::render::Renderer;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::systems::game_system::GameSystem;
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

enum Asset {
    Vertices(Vec<PrimitiveVertices>),
    Color(ColorDefinition),
    Texture(ImageAsset),
}

pub enum CustomEvent {
    StateInitializationEvent(Engine),
    WebResizedEvent, // Only sent on web when window gets resized. Resized event only seems to send event on dpi change (browser zoom or system scale ratio) which is insufficient
    AudioStateChanged(AudioState),
    AssetLoaded(Asset),
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
        // Note: 0.0 would lead to error on x11 so we define a minimum size of 1 by 1
        #[allow(unused_mut)]
        let mut initial_width = 1.0;
        #[allow(unused_mut)]
        let mut initial_height = 1.0;
        let web_window = web_sys::window().expect("Window should exist");
        // Personal reminder to probably never use window's inner width. Visual viewport returns a float
        let viewport = &web_window
            .visual_viewport()
            .expect("Visual viewport should exist");
        initial_width = viewport.width();
        initial_height = viewport.height();

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

        let window_attributes;

        window_attributes = Window::default_attributes()
            .with_title("Kloenk!")
            .with_inner_size(LogicalSize::new(initial_width, initial_height))
            .with_active(true)
            // Not so happy with browser only supporting 32x32 without edge failures. Other option is using larger image and mirroring or rotating the image near right/bottom border. Rendering the cursor leads to slight trailing which feels not great in a game so probably want to use system supported, limited cursor
            .with_cursor(event_loop.create_custom_cursor(CustomCursor::from_url(
                String::from("assets/cursor.webp"),
                3,
                3,
            )));
        // Note: window icon on web unsupported and performed through favicon in html

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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

        let renderer_future = Renderer::new(window.clone());

        let event_loop_proxy = self.event_loop_proxy.clone();

        spawn_local(async move {
            let renderer = renderer_future.await;
            let engine = Engine {
                renderer,
                game_state: GameState::new(),
                ui_state: UIState::new(),
                input_handler: Input::new(),
                frame_state: FrameState::new(),
                audio_state: AudioState::NotLoaded,
                window,
            };

            event_loop_proxy
                .send_event(CustomEvent::StateInitializationEvent(engine))
                .unwrap_or_else(|_| {
                    panic!("Failed to send initialization event");
                });
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: CustomEvent) {
        match event {
            CustomEvent::StateInitializationEvent(mut engine) => {
                // log::info!("Received initialization event"); dev flag
                engine.renderer.resize(engine.window.inner_size()); // Web inner size request does not seem to lead to resized event, but also does not seem to immediately apply. Arbitrarily hope resize is done and apply resize here...
                engine.window.request_redraw(); // TODO are these resizes here necessary?

                // TODO make a method for loading the models
                // Getting quickly through setup stage such that renderer can start rendering first frame
                // TODO wondering about order loading models/request redraw? below is blocking anyway for until next window draw event? prob does not matter much
                for (_, model) in engine.renderer.model_manager.get_active_models().clone() {
                    // maybe first make sure uniqueness before loading
                    for primitive in &model.primitives {
                        let vertices_id_clone = primitive.vertices_id.clone();
                        if primitive.vertices_id.ends_with(".gltf") {
                            // let vertices_clone = primitive.clone();
                            let event_loop = self.event_loop_proxy.clone();
                            spawn_local(async move {
                                let primitive_vertices =
                                    ModelLoader::load_gltf(&*vertices_id_clone).await;
                                event_loop
                                    .send_event(CustomEvent::AssetLoaded(Vertices(
                                        primitive_vertices,
                                    )))
                                    .unwrap_or_else(|_| {
                                        panic!("Failed to send vertices event");
                                    });
                            });
                        }

                        if let Some(texture_id) = &primitive.texture_definition {
                            let event_loop = self.event_loop_proxy.clone();

                            let texture_id_clone = texture_id.clone();
                            spawn_local(async move {
                                // TODO check if not already loaded first
                                let image_texture_asset =
                                    AssetLoader::load_image_asset(&texture_id_clone.file_name).await;
                                // AssetLoader::load_image_asset(&texture_id.file_name).await;
                                event_loop
                                    .send_event(CustomEvent::AssetLoaded(Texture(
                                        image_texture_asset,
                                    )))
                                    .unwrap_or_else(|_| {
                                        panic!("Failed to send texture event");
                                    });
                            });
                        }

                        let event_loop = self.event_loop_proxy.clone();
                        let color_definition = primitive.color_definition.clone();
                        spawn_local(async move {
                            event_loop
                                .send_event(CustomEvent::AssetLoaded(Color(color_definition)))
                                .unwrap_or_else(|_| {
                                    panic!("Failed to send texture event");
                                });
                        });
                        // todo check if color not already loaded? Maybe only send unique assets?
                    }
                }

                self.application_state = State::Initialized(engine);
            }
            CustomEvent::AssetLoaded(asset) => {
                if let State::Initialized(engine) = &mut self.application_state {
                    match asset {
                        Vertices(primitive_vertices) => {
                            engine
                                .renderer
                                .load_primitive_vertices_to_memory(primitive_vertices);
                        }
                        Color(color_definition) => {
                            engine.renderer.load_color_to_memory(color_definition);
                        }
                        Texture(texture_asset) => {
                            engine.renderer.load_material_to_memory(texture_asset);
                        }
                    }
                } else {
                    panic!("Renderer should be ready before assets are loaded to GPU memory");
                }
            }
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

                // Loading audio only after user has gestured on web
                // Thought of callback or observer pattern but that honestly seems way too complex compared to this.
                if key_is_gesture(key) {
                    Self::load_audio_player(&self.event_loop_proxy, engine);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                engine.input_handler.process_mouse_button(button, state);

                // Loading audio only after user has gestured on web
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
                    &mut engine.frame_state,
                ) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        engine.renderer.resize(engine.window.inner_size());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        // log::error!("Out of memory"); dev
                        event_loop.exit();
                    }

                    Err(wgpu::SurfaceError::Timeout) => {
                        // log::warn!("Surface timeout"); dev
                    }

                    Err(wgpu::SurfaceError::Other) => {
                        // log::warn!("Other error"); dev
                    }
                }
            }
            _ => {}
        }
    }
}

fn key_is_gesture(key: KeyCode) -> bool {
    !matches!(
        key,
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
            | KeyCode::Hyper
    )
}
