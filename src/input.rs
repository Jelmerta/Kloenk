use winit::event::{ElementState, VirtualKeyCode};

#[derive(Debug, Default)]
pub struct input {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
}

impl input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        let is_pressed = *state == ElementState::Pressed;

        match keycode {
            VirtualKeyCode::W => {
                self.up_pressed = is_pressed;
                return
            }

            VirtualKeyCode::S => {
                self.down_pressed = is_pressed;
                return
            }

            VirtualKeyCode::A => {
                self.left_pressed = is_pressed;
                return
            }

            VirtualKeyCode::D => {
                self.right_pressed = is_pressed;
                return
            }

            _ => {

            }
        }
    }
}