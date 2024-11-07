use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::gui::{UIElement, UIState};
use crate::input::Input;
use cgmath::Point2;

pub struct InventorySystem {}

impl InventorySystem {
    pub fn handle_inventory_click(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        frame_state: &mut FrameState,
    ) {
        // Assume toggle is handled. Probably toggles should be handled before performing any
        // systems on them

        if !ui_state.inventory.is_visible {
            return;
        }

        if frame_state.handled_left_click {
            return;
        }

        if !input.left_mouse_clicked.is_toggled_on() {
            return;
        }

        let cursor_ndc = input.mouse_position_ndc;
        let cursor_ui_space = Point2::new(cursor_ndc.x / 2.0 + 0.5, -cursor_ndc.y / 2.0 + 0.5);

        if !ui_state.inventory.contains(cursor_ui_space) {
            return;
        }

        UIElement::inventory_trigger_click(game_state, ui_state, input);
        frame_state.handled_left_click = true;
    }

    pub fn handle_inventory(ui_state: &mut UIState, input: &mut Input) {
        if input.i_pressed.is_toggled_on() {
            ui_state.inventory.toggle_visibility();
        }
    }
}
