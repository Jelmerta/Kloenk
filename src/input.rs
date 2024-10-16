use cgmath::Point2;
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

#[derive(Debug)]
pub struct Input {
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

    pub mouse_position_ndc: Point2<f32>,
    pub right_mouse_clicked: KeyPress,
    pub left_mouse_clicked: KeyPress,

    pub scrolled_amount: f32,
}

impl Input {
    pub fn new() -> Self {
        Input {
            w_pressed: Default::default(),
            s_pressed: Default::default(),
            a_pressed: Default::default(),
            d_pressed: Default::default(),

            i_pressed: Default::default(),
            e_pressed: Default::default(),

            up_pressed: Default::default(),
            down_pressed: Default::default(),
            left_pressed: Default::default(),
            right_pressed: Default::default(),

            left_shift_pressed: Default::default(),

            mouse_position_ndc: Point2::new(0.0, 0.0), // TODO what's a default position for the mouse?
            right_mouse_clicked: Default::default(),
            left_mouse_clicked: Default::default(),

            scrolled_amount: 0.0,
        }
    }

    pub fn update(&mut self, keycode: KeyCode, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;

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

    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;

        match button {
            MouseButton::Right => {
                self.right_mouse_clicked.set_press_state(is_pressed);
            }

            MouseButton::Left => {
                self.left_mouse_clicked.set_press_state(is_pressed);
            }

            _ => {}
        }
    }

    pub fn process_mouse_movement(&mut self, mouse_position: PhysicalPosition<f64>) {
        // Convert to value between 0 and 1
        self.mouse_position_ndc = Point2 {
            x: (2.0 * mouse_position.x) as f32 / 800.0, // -1.0?
            y: 1.0 - (2.0 * mouse_position.y) as f32 / 600.0,
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
