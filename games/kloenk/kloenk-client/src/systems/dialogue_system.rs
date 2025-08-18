use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{DialogueState, UIElement, UIState, UserAction};
use crate::systems::dialogue_manager::DialogueManager;
use crate::systems::position_manager::PositionManager;
use cgmath::Point2;
use std::sync::Arc;
use winit::window::Window;

const DIALOGUE_RANGE: f32 = 1.5;

pub struct DialogueSystem {}

impl DialogueSystem {
    pub fn handle_open_dialogue_keyboard(
        game_state: &GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if input.e_pressed.is_toggled_on()
            && !frame_state.handled_e_click
            && let Some(near_dialog_interactable) = PositionManager::find_nearest_dialog(game_state)
        {
            if !PositionManager::in_range(
                game_state.get_position("player").expect("Player position should exist"),
                game_state.get_position(near_dialog_interactable).expect("Nearest dialogue was found and should have position"),
                DIALOGUE_RANGE,
            ) {
                // Not in range
                return;
            }

            // Open dialogue

            let dialogue = game_state
                .dialogue_components
                .get(near_dialog_interactable)
                .unwrap();

            ui_state.dialogue_state = DialogueState::Npc {
                mouse_position: input.mouse_position_ui,
                npc_entity_id: near_dialog_interactable.to_owned(),
                dialogue_id: dialogue.clone().dialogue_id,
            };
            frame_state.handled_e_click = true;
        }
    }

    pub fn display_dialogue(
        window: &Arc<Window>,
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
            let dialogue_rect = UIElement::new_rect(
                Point2::new(mouse_position.x + 0.01, mouse_position.y + 0.05),
                Point2::new(0.16, 0.05),
            );

            match frame_state
                .gui
                .color_button(window, 150, dialogue_rect, input, "black")
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
                &dialogue_text.clone().text,
                [0.8, 0.8, 0.0],
            );

            let close_button_rect =
                dialogue_rect.inner_rect_maintain_ratio_x(Point2::new(0.9, 0.05), 0.10);
            match frame_state.gui.image_button(
                window,
                310,
                close_button_rect,
                "close_button",
                input,
            ) {
                UserAction::None => {}
                UserAction::Hover => {
                    // Feels kinda silly/hacky to overlay hover image
                    match frame_state.gui.image_button(
                        window,
                        311,
                        close_button_rect,
                        "close_button_hover",
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
                    if !frame_state.handled_left_click {
                        new_dialogue_state = Some(DialogueState::Closed);
                        frame_state.handled_left_click = true;
                    }
                }
                UserAction::RightClick => {}
            }

            if !PositionManager::in_range(
                game_state.get_position("player").expect("Player position should exist"),
                game_state.get_position(npc_entity_id).expect("Interacted NPC position should exist"),
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
