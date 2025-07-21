use winit::application::ApplicationHandler;
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::PhysicalKey,
};

use winit::event::ElementState;

use crate::state::input::Input;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Cursor, Fullscreen, Icon, Window, WindowId};

use crate::application::cursor_manager::CursorManager;
use crate::render::render::Renderer;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::systems::game_system::GameSystem;
use winit::keyboard::KeyCode;

use hydrox::{load_binary, AudioSystem};

pub struct Engine {
    pub renderer: Renderer,
    pub game_state: GameState,
    pub ui_state: UIState,
    pub input_handler: Input,
    pub frame_state: FrameState,
    pub window: Arc<Window>,
    pub audio_state: AudioState, // todo just audio_system or sth once we have event system working better
}

impl Engine {
    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }
}

pub enum AudioState {
    Loaded(AudioSystem),
}

pub enum State {
    Uninitialized,
    Initialized(Engine),
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Note: This is more of a logical size than a physical size. https://docs.rs/bevy/latest/bevy/window/struct.WindowResolution.html
        // For example: System scale or web zoom can change physical size, but not this value. (we could have a menu to change this though.)
        // We want to have ownership of the zoom level ourselves. We therefore disregard the dpi ratio and always attempt to render the same image
        // Note: 0.0 would lead to error on x11 so we define a minimum size of 1 by 1
        #[allow(unused_mut)]
        let mut initial_width = 1.0;
        #[allow(unused_mut)]
        let mut initial_height = 1.0;

        let window_attributes;
        let cursor_binary = pollster::block_on(load_binary("cursor.webp")).unwrap();
        let cursor_source = CursorManager::load_cursor(cursor_binary.clone());
        let custom_cursor = event_loop.create_custom_cursor(cursor_source);

        let window_icon_binary = pollster::block_on(load_binary("kunst.webp")).unwrap();
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

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        if let Some(monitor) = window.current_monitor() {
            let fullscreen_video_mode = monitor.video_modes().next().unwrap();
            let _ = window.request_inner_size(fullscreen_video_mode.size());
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
        }

        let renderer_future = Renderer::new(window.clone());

        let renderer = pollster::block_on(renderer_future);
        let audio_system = pollster::block_on(AudioSystem::new());

        self.application_state = State::Initialized(Engine {
            renderer,
            game_state: GameState::new(),
            ui_state: UIState::new(),
            input_handler: Input::new(),
            frame_state: FrameState::new(),
            window: window.clone(),
            audio_state: AudioState::Loaded(audio_system),
        });
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
                    engine.window.inner_size().width as f32,
                    engine.window.inner_size().height as f32,
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                engine.input_handler.process_scroll(&delta);
            }
            // // Web uses custom resize event as web only sends event on dpi changes
            WindowEvent::Resized(physical_size) => {
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

                    Err(wgpu::SurfaceError::Other) => {
                        log::warn!("Other error");
                    }
                }
            }
            _ => {}
        }
    }
}
