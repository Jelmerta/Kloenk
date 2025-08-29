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

    pub fn is_toggled_on(&self) -> bool {
        !self.was_pressed && self.is_pressed
    }

    pub fn update_end_frame(&mut self) {
        self.was_pressed = self.is_pressed;
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

    pub enter_pressed: KeyPress,

    pub left_shift_pressed: KeyPress,

    pub mouse_position_ndc: Point2<f32>,
    pub mouse_position_ui: Point2<f32>,
    pub right_mouse_clicked: KeyPress,
    pub left_mouse_clicked: KeyPress,

    pub scrolled_amount: f32,
}

impl Input {
    pub fn new() -> Self {
        Input {
            w_pressed: KeyPress::default(),
            s_pressed: KeyPress::default(),
            a_pressed: KeyPress::default(),
            d_pressed: KeyPress::default(),

            i_pressed: KeyPress::default(),
            e_pressed: KeyPress::default(),

            up_pressed: KeyPress::default(),
            down_pressed: KeyPress::default(),
            left_pressed: KeyPress::default(),
            right_pressed: KeyPress::default(),

            enter_pressed: KeyPress::default(),

            left_shift_pressed: KeyPress::default(),

            mouse_position_ndc: Point2::new(0.0, 0.0),
            mouse_position_ui: Point2::new(0.5, 0.5),
            right_mouse_clicked: KeyPress::default(),
            left_mouse_clicked: KeyPress::default(),

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

            KeyCode::Enter => {
                self.enter_pressed.set_press_state(is_pressed);
            }

            _ => {}
        }
    }

    pub fn update_end_frame(&mut self) {
        self.w_pressed.update_end_frame();
        self.s_pressed.update_end_frame();
        self.a_pressed.update_end_frame();
        self.d_pressed.update_end_frame();
        self.i_pressed.update_end_frame();
        self.e_pressed.update_end_frame();
        self.up_pressed.update_end_frame();
        self.down_pressed.update_end_frame();
        self.left_pressed.update_end_frame();
        self.right_pressed.update_end_frame();
        self.enter_pressed.update_end_frame();
        self.left_shift_pressed.update_end_frame();
        self.right_mouse_clicked.update_end_frame();
        self.left_mouse_clicked.update_end_frame();
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

    pub fn process_mouse_movement(
        &mut self,
        mouse_position: PhysicalPosition<f64>,
        window_width: u32,
        window_height: u32,
    ) {
        self.mouse_position_ndc = Point2 {
            x: ((2.0 * mouse_position.x) / window_width as f64 - 1.0) as f32,
            y: (1.0 - (2.0 * mouse_position.y) / window_height as f64) as f32,
        };
        self.mouse_position_ui = Point2::new(
            self.mouse_position_ndc.x / 2.0 + 0.5,
            -self.mouse_position_ndc.y / 2.0 + 0.5,
        );
    }

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
