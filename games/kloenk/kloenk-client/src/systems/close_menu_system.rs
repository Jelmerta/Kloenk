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

        match &ui_state.menu_state {
            MenuState::WorldAction {
                render_position: _render_position,
                item: _item,
            } => {
                frame_state.handled_left_click = true;
                ui_state.menu_state = Closed;
            }
            MenuState::InventoryAction {
                render_position: _render_position,
                item: _item,
            } => {
                frame_state.handled_left_click = true;
                ui_state.menu_state = Closed;
            }
            Closed => {}
        }
    }
}
