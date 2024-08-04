use winit::event::{ElementState, VirtualKeyCode, MouseScrollDelta};
use winit::dpi::PhysicalPosition;

#[derive(Debug, Default)]
pub struct KeyPress {
    pub is_pressed: bool,
    pub was_pressed: bool,
}

impl KeyPress {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_press_state(&mut self, new_state: bool) {
        self.was_pressed = self.is_pressed;
        self.is_pressed = new_state;
    }

    // TODO HMM different for movement and opening menus
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
    pub w_pressed: KeyPress,
    pub s_pressed: KeyPress,
    pub a_pressed: KeyPress,
    pub d_pressed: KeyPress,

    pub i_pressed: KeyPress,
    pub g_pressed: KeyPress,

    pub up_pressed: KeyPress,
    pub down_pressed: KeyPress,
    pub left_pressed: KeyPress,
    pub right_pressed: KeyPress,

    pub left_shift_pressed: KeyPress,

    pub scrolled_amount: f32,
}

impl Input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        let is_pressed = *state == ElementState::Pressed;

        match keycode {
            VirtualKeyCode::W => {
                self.w_pressed.set_press_state(is_pressed);
                return;
            }

            VirtualKeyCode::S => {
                self.s_pressed.set_press_state(is_pressed);
                return;
            }

            VirtualKeyCode::A => {
                self.a_pressed.set_press_state(is_pressed);
                return;
            }

            VirtualKeyCode::D => {
                self.d_pressed.set_press_state(is_pressed);
                return;
            }

            VirtualKeyCode::I => {
                self.i_pressed.set_press_state(is_pressed);
                return;
            }

            VirtualKeyCode::G => {
                self.g_pressed.set_press_state(is_pressed);
                return;
            }
            
            VirtualKeyCode::LShift => {
                self.left_shift_pressed.set_press_state(is_pressed);
            }

            VirtualKeyCode::Up => {
                self.up_pressed.set_press_state(is_pressed);
            }

            VirtualKeyCode::Down => {
                self.down_pressed.set_press_state(is_pressed);
            }

            VirtualKeyCode::Left => {
                self.left_pressed.set_press_state(is_pressed);
            }
            
            VirtualKeyCode::Right => {
                self.right_pressed.set_press_state(is_pressed);
            }
            _ => {}
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scrolled_amount = match delta {
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll, ..
            }) => *scroll as f32,
            _ => panic!("LineDelta not implemented"),
            // MouseScrollDelta::LineDelta(_, scroll) => *scroll,
        };
    }
}
