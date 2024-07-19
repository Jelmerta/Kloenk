use winit::event::{ElementState, VirtualKeyCode, MouseScrollDelta};
use winit::dpi::PhysicalPosition;

#[derive(Debug, Default)]
pub struct input {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub scrolled_amount: f32,
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
                return;
            }

            VirtualKeyCode::S => {
                self.down_pressed = is_pressed;
                return;
            }

            VirtualKeyCode::A => {
                self.left_pressed = is_pressed;
                return;
            }

            VirtualKeyCode::D => {
                self.right_pressed = is_pressed;
                return;
            }

            _ => {}
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scrolled_amount = match delta {
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll, ..
            }) => *scroll as f32,
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
        };
    }
}
