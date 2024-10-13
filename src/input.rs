use crate::audio_system::AudioSystem;
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::KeyCode;

#[derive(Debug, Default)]
pub struct KeyPress {
    pub is_pressed: bool,
    pub was_pressed: bool,
}

impl KeyPress {
    pub fn set_press_state(&mut self, new_state: bool) {
        self.was_pressed = self.is_pressed;
        self.is_pressed = new_state;
    }

    pub fn is_toggled_on(&mut self) -> bool {
        // Once pressed , we do not get n update every frame, so we just make sure that this method
        // is not called again every frame by setting was pressed
        let is_toggled = !self.was_pressed && self.is_pressed;
        self.was_pressed = true;
        is_toggled
    }
}

#[derive(Debug, Default)]
pub struct Input {
    user_has_gestured: bool,

    pub w_pressed: KeyPress,
    pub s_pressed: KeyPress,
    pub a_pressed: KeyPress,
    pub d_pressed: KeyPress,

    pub i_pressed: KeyPress,
    pub e_pressed: KeyPress,

    pub up_pressed: KeyPress,
    pub down_pressed: KeyPress,
    pub left_pressed: KeyPress,
    pub right_pressed: KeyPress,

    pub left_shift_pressed: KeyPress,

    pub right_mouse_clicked: KeyPress,

    pub scrolled_amount: f32,
}

impl Input {
    pub fn new() -> Self {
        Input::default()
    }

    pub fn update(
        &mut self,
        keycode: KeyCode,
        state: ElementState,
        audio_system: &Rc<RefCell<AudioSystem>>,
    ) {
        let is_pressed = state == ElementState::Pressed;

        // Yeah... Bit ugly... Thought of callback or observer pattern but that honestly seems way too complex compared to this.
        if !self.user_has_gestured && is_pressed {
            self.user_has_gestured = true;
            let audio_system_clone = audio_system.clone();
            spawn_local(async move {
                let mut audio_system_mut = audio_system_clone.borrow_mut();
                audio_system_mut.start().await;
            });
        }

        match keycode {
            KeyCode::KeyW => {
                self.w_pressed.set_press_state(is_pressed);
            }

            KeyCode::KeyS => {
                self.s_pressed.set_press_state(is_pressed);
            }

            KeyCode::KeyA => {
                self.a_pressed.set_press_state(is_pressed);
            }

            KeyCode::KeyD => {
                self.d_pressed.set_press_state(is_pressed);
            }

            KeyCode::KeyI => {
                self.i_pressed.set_press_state(is_pressed);
            }

            KeyCode::KeyE => {
                self.e_pressed.set_press_state(is_pressed);
            }

            KeyCode::ShiftLeft => {
                self.left_shift_pressed.set_press_state(is_pressed);
            }

            KeyCode::ArrowUp => {
                self.up_pressed.set_press_state(is_pressed);
            }

            KeyCode::ArrowDown => {
                self.down_pressed.set_press_state(is_pressed);
            }

            KeyCode::ArrowLeft => {
                self.left_pressed.set_press_state(is_pressed);
            }

            KeyCode::ArrowRight => {
                self.right_pressed.set_press_state(is_pressed);
            }
            _ => {}
        }
    }

    pub fn process_mouse_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
        audio_system: &Rc<RefCell<AudioSystem>>,
    ) {
        let is_pressed = state == ElementState::Pressed;

        // Yeah... Bit ugly... Thought of callback or observer pattern but that honestly seems way too complex compared to this.
        if !self.user_has_gestured && is_pressed {
            self.user_has_gestured = true;
            let audio_system_clone = audio_system.clone();
            spawn_local(async move {
                let mut audio_system_mut = audio_system_clone.borrow_mut();
                audio_system_mut.start().await;
            });
        }

        #[allow(clippy::single_match)]
        match button {
            MouseButton::Right => {
                self.right_mouse_clicked.set_press_state(is_pressed);
            }

            _ => {}
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scrolled_amount = match delta {
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                // Used by WASM
                y: scroll,
                ..
            }) => *scroll as f32,
            MouseScrollDelta::LineDelta(_, scroll) => *scroll * 100.0, // Used by standalone client
        };
    }
}
