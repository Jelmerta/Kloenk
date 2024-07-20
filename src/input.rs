use winit::event::{ElementState, VirtualKeyCode, MouseScrollDelta};
use winit::dpi::PhysicalPosition;

#[derive(Debug, Default)]
pub struct input {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,

    pub u_pressed: bool,
    pub j_pressed: bool,
    pub h_pressed: bool,
    pub k_pressed: bool,

    pub left_shift_pressed: bool,

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

            VirtualKeyCode::LShift => {
                self.left_shift_pressed = is_pressed;
            }

            VirtualKeyCode::U => {
                self.u_pressed = is_pressed;
            }

            VirtualKeyCode::J => {
                self.j_pressed = is_pressed;
            }

            VirtualKeyCode::H => {
                self.h_pressed = is_pressed;
            }
            
            VirtualKeyCode::K => {
                self.k_pressed = is_pressed;
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
