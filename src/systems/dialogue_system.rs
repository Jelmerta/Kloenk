use crate::state::frame_state::{ActionEffect, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::systems::position_manager::PositionManager;

const DIALOGUE_RANGE: f32 = 1.5;

pub struct DialogueSystem {}

impl DialogueSystem {
    pub fn handle_talking_keyboard(
        game_state: &GameState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if input.e_pressed.is_toggled_on() && !frame_state.handled_e_click {
            if let Some(near_dialog_interactable) = PositionManager::find_nearest_dialog(game_state)
            {
                if !PositionManager::in_range(
                    game_state.get_position(&"player".to_string()).unwrap(),
                    game_state.get_position(&near_dialog_interactable).unwrap(),
                    DIALOGUE_RANGE,
                ) {
                    // Not in range
                    return;
                }

                frame_state.action_effects.push(ActionEffect::Examine {
                    text: "nice meme dennis".to_string(),
                });
            }
        }
    }
}
