use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{DialogueState, Rect, UIState, UserAction};
use crate::systems::dialogue_manager::DialogueManager;
use crate::systems::position_manager::PositionManager;
use cgmath::Point2;

const DIALOGUE_RANGE: f32 = 1.5;

pub struct DialogueSystem {}

impl DialogueSystem {
    pub fn handle_open_dialogue_keyboard(
        game_state: &GameState,
        ui_state: &mut UIState,
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

                // Open dialogue

                let dialogue = game_state
                    .dialogue_components
                    .get(&near_dialog_interactable)
                    .unwrap();

                ui_state.dialogue_state = DialogueState::Npc {
                    mouse_position: input.mouse_position_ui.clone(),
                    npc_entity_id: near_dialog_interactable,
                    dialogue_id: dialogue.clone().dialogue_id,
                };
                frame_state.handled_e_click = true;
            }
        }
    }

    pub fn display_dialogue(
        game_state: &GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let mut new_dialogue_state = None;
        if let DialogueState::Npc {
            mouse_position,
            npc_entity_id,
            dialogue_id,
        } = &ui_state.dialogue_state
        {
            let dialogue_rect = Rect::new(
                Point2::new(mouse_position.x - 0.15, mouse_position.y + 0.00),
                Point2::new(mouse_position.x + 0.17, mouse_position.y + 0.10),
            );

            match frame_state
                .gui
                .color_button(150, dialogue_rect, input, "black".to_string())
            {
                UserAction::None => {}
                UserAction::Hover => {}
                UserAction::LeftClick => {}
                UserAction::RightClick => {}
            }

            let dialogue_manager = DialogueManager::new();
            let dialogue_text = dialogue_manager.get_dialogue(dialogue_id).unwrap();

            frame_state.gui.text(
                300,
                dialogue_rect,
                dialogue_text.clone().text,
                [0.8, 0.8, 0.0],
            );

            let close_button_rect = Rect::new(
                Point2::new(
                    dialogue_rect.bottom_right.x - 0.03,
                    dialogue_rect.top_left.y + 0.01,
                ),
                Point2::new(
                    dialogue_rect.bottom_right.x - 0.01,
                    dialogue_rect.top_left.y + 0.03,
                ),
            );
            match frame_state.gui.image_button(
                310,
                close_button_rect,
                "close_button".to_string(),
                input,
            ) {
                UserAction::None => {}
                UserAction::Hover => {
                    // Feels kinda silly/hacky to overlay hover image
                    match frame_state.gui.image_button(
                        311,
                        close_button_rect,
                        "close_button_hover".to_string(),
                        input,
                    ) {
                        UserAction::None => {}
                        UserAction::Hover => {}
                        UserAction::LeftClick => {
                            new_dialogue_state = Some(DialogueState::Closed);
                        }
                        UserAction::RightClick => {}
                    }
                }
                UserAction::LeftClick => {
                    new_dialogue_state = Some(DialogueState::Closed);
                }
                UserAction::RightClick => {}
            }

            if !PositionManager::in_range(
                game_state.get_position(&"player".to_string()).unwrap(),
                game_state.get_position(&npc_entity_id).unwrap(),
                DIALOGUE_RANGE,
            ) {
                new_dialogue_state = Some(DialogueState::Closed);
            }
        }
        if let Some(new_state) = new_dialogue_state {
            ui_state.dialogue_state = new_state;
        }
    }
}
