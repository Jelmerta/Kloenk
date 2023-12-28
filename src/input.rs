use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug, Default)]
pub struct input {
    up_pressed: bool,
    down_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
}

impl input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, key: KeyboardInput) {
        let keycode = key.virtual_keycode.unwrap();
        let state = key.state;
        let is_pressed = state == ElementState::Pressed;

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