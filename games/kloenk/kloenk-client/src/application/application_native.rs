use winit::application::ApplicationHandler;
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::PhysicalKey,
};

use winit::event::{DeviceEvent, DeviceId, ElementState, StartCause};

use crate::state::input::Input;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Cursor, CustomCursor, Fullscreen, Icon, Window, WindowId};

use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::systems::game_system::GameSystem;
use winit::keyboard::KeyCode;

use crate::application::framerate_handler::UpdateTickHandler;
use crate::render::model_loader::ModelLoader;
use crate::render::renderer::Renderer;
use hydrox::{load_binary, AudioSystem, Sound};

pub struct Engine {
    pub game_state: GameState,
    pub ui_state: UIState,
    pub frame_state: FrameState,

    pub input_handler: Input, // TODO if we do really need this: maybe more like input_state?
    pub window: Arc<Window>, // TODO Is the only reason for having this in engine to access inner size? although that might still be valid reason. dont want to copy the data
    pub renderer: Renderer,
    pub framerate_handler: UpdateTickHandler,
    pub audio_system: AudioSystem,
}

pub enum State {
    Uninitialized,
    Initialized(Box<Engine>),
}

pub struct Application {
    application_state: State,
}

impl Application {
    pub fn new() -> Application {
        Application {
            application_state: State::Uninitialized,
        }
    }
}

impl ApplicationHandler for Application {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } => {}
            StartCause::WaitCancelled { .. } => {}
            StartCause::Poll => match &mut self.application_state {
                State::Uninitialized => {}
                State::Initialized(engine) => {
                    while engine.framerate_handler.should_update() {
                        engine.renderer.updating();
                        #[cfg(feature = "debug-logging")]
                        log::debug!("updating");
                        GameSystem::update(
                            &engine.window,
                            &mut engine.game_state,
                            &mut engine.ui_state,
                            &mut engine.input_handler,
                            &mut engine.frame_state,
                            &mut engine.audio_system,
                        );
                        engine.renderer.updated(
                            &engine.window,
                            &mut engine.frame_state,
                            &mut engine.game_state,
                        );
                        engine.framerate_handler.updated();
                    }
                    #[cfg(feature = "debug-logging")]
                    log::debug!("drawing");
                    engine.window.request_redraw();
                }
            },
            StartCause::Init => {
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(feature = "debug-logging")]
        log::debug!("Resumed loop");

        let State::Uninitialized = self.application_state else {
            return;
        };

        // Note: This is more of a logical size than a physical size. https://docs.rs/bevy/latest/bevy/window/struct.WindowResolution.html
        // For example: System scale or web zoom can change physical size, but not this value. (we could have a menu to change this though.)
        // We want to have ownership of the zoom level ourselves. We therefore disregard the dpi ratio and always attempt to render the same image
        // Note: 0.0 would lead to error on x11 so we define a minimum size of 1 by 1
        let initial_width = 1.0;
        let initial_height = 1.0;

        let window_attributes;
        let cursor_binary = pollster::block_on(load_binary("cursor.rgba")).unwrap();
        let cursor_source = CustomCursor::from_rgba(cursor_binary, 61, 60, 3, 3).unwrap();
        let custom_cursor = event_loop.create_custom_cursor(cursor_source);

        let window_icon_binary = pollster::block_on(load_binary("favicon.rgba")).unwrap();
        let window_icon = Some(Icon::from_rgba(window_icon_binary, 64, 64).unwrap());

        window_attributes = Window::default_attributes()
            .with_title("Kloenk!")
            .with_inner_size(LogicalSize::new(initial_width, initial_height))
            .with_active(true)
            .with_cursor(Cursor::Custom(custom_cursor))
            .with_visible(false)
            .with_window_icon(window_icon);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        if let Some(monitor) = window.current_monitor() {
            let fullscreen_video_mode = monitor.video_modes().next().unwrap();
            let _ = window.request_inner_size(fullscreen_video_mode.size());

            #[cfg(feature = "debug-logging")]
            {
                if let Some(mhz) = monitor.refresh_rate_millihertz() {
                    log::debug!("Monitor millihertz: {mhz}");
                }
            }
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
        }

        let mut renderer = pollster::block_on(Renderer::new(window.clone()));
        for (_, model) in renderer.model_manager.get_active_models().clone() {
            // TODO maybe first make sure uniqueness before loading
            for primitive in &model.primitives {
                if std::path::Path::new(&primitive.vertices_id)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("gltf"))
                {
                    let primitive_vertices =
                        pollster::block_on(ModelLoader::load_gltf(&primitive.vertices_id));
                    renderer.load_primitive_vertices_to_memory(&primitive_vertices);
                }

                if let Some(texture_id) = &primitive.texture_definition {
                    // TODO check if not already loaded first
                    let image_texture_asset = pollster::block_on(
                        crate::application::asset_loader::AssetLoader::load_image_asset(
                            &texture_id.file_name,
                        ),
                    );
                    renderer.load_material_to_memory(&image_texture_asset);
                }

                renderer.load_color_to_memory(&primitive.color_definition);

                // todo check if not already loaded
            }
        }

        let mut audio_system = AudioSystem::new();
        let bonk = pollster::block_on(load_binary("bonk.wav")).expect("bonk exists");
        audio_system.load_sound("bonk", &Sound { bytes: bonk });

        self.application_state = State::Initialized(Box::new(Engine {
            renderer,
            framerate_handler: UpdateTickHandler::new(),
            game_state: GameState::new(),
            ui_state: UIState::new(),
            input_handler: Input::new(),
            frame_state: FrameState::new(),
            window: window.clone(),
            audio_system,
        }));
        window.set_visible(true); // Not sure why, but cannot draw (just on windows? not tested elsewhere) without the window being visible -> set_visible also implicit seems to start requesting redraws
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
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => event_loop.exit(),
            // TODO ModifiersChanged? like shift ctrl. is this more for typing?
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
            WindowEvent::CursorMoved { position, .. } => {
                engine.input_handler.process_mouse_movement(
                    position,
                    engine.window.inner_size().width,
                    engine.window.inner_size().height,
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                engine.input_handler.process_scroll(&delta);
            }
            // TODO ScaleFactorChanged? check DPI change
            WindowEvent::Resized(physical_size) => {
                engine.renderer.resize(physical_size);
            }
            // TODO handle window going out of focus/out of view (occluded)
            WindowEvent::RedrawRequested => {
                // TODO Wondering if we should do something with extrapolation: lag / MS_PER_UPDATE. Maybe only for animations etc
                // https://gameprogrammingpatterns.com/game-loop.html: The renderer knows each game object and its current velocity. doubt it
                // maybe an option?
                // maybe only perform if off by some amount like 2ms?
                // maybe we do run an update() on a cloned version? not the real state. slow updat
                match engine.renderer.render(&engine.window) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        engine.renderer.resize(engine.window.inner_size());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        #[cfg(feature = "debug-logging")]
                        log::warn!("Out of memory");
                        event_loop.exit();
                    }

                    Err(wgpu::SurfaceError::Timeout) => {
                        #[cfg(feature = "debug-logging")]
                        log::warn!("Surface timeout");
                    }

                    Err(wgpu::SurfaceError::Other) => {
                        #[cfg(feature = "debug-logging")]
                        log::warn!("Other error");
                    }
                }
            }
            _ => {}
        }
    }

    // Interesting: Can provide device information even when not focused. I would hope I cannot read keyboard input from other windows though.
    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, _: DeviceEvent) {
        // todo!()
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        #[cfg(feature = "debug-logging")]
        log::debug!("Suspended application");
        // pause rendering? i mean this just works on its own does it not? what is recommended here?
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        #[cfg(feature = "debug-logging")]
        log::debug!("Exiting");
    }

    fn memory_warning(&mut self, _: &ActiveEventLoop) {
        #[cfg(feature = "debug-logging")]
        log::warn!("Memory warning");
    }
}
