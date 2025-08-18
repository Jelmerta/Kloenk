use crate::state::frame_state::FrameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::Closed;
use crate::state::ui_state::{MenuState, UIState};

pub struct CloseMenuSystem {}

impl CloseMenuSystem {
    pub fn check_to_close_menu(
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if frame_state.handled_left_click {
            return;
        }

        if !input.left_mouse_clicked.is_toggled_on() {
            return;
        }

        let mut new_menu_state = ui_state.menu_state.clone();
        match &ui_state.menu_state {
            MenuState::World {
                mouse_position: _mouse_position,
                item: _item,
            } => {
                new_menu_state = Closed;
                frame_state.handled_left_click = true;
            }
            MenuState::Inventory {
                mouse_position: _mouse_position,
                item: _item,
            } => {
                new_menu_state = Closed;
                frame_state.handled_left_click = true;
            }
            Closed => {}
        }
        ui_state.menu_state = new_menu_state;
    }
}
