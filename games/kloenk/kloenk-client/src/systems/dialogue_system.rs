use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{DialogueState, RenderCommand, UIElement, UIState, UserAction};
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
                game_state
                    .get_position("player")
                    .expect("Player position should exist"),
                game_state
                    .get_position(near_dialog_interactable)
                    .expect("Nearest dialogue was found and should have position"),
                DIALOGUE_RANGE,
            ) {
                // Not in range
                return;
            }

            // Open dialogue
            let dialogue = game_state
                .dialogue_components
                .get(near_dialog_interactable)
                .expect("Dialogue component should exist");

            ui_state.dialogue_state = DialogueState::Npc {
                render_position: input.mouse_position_ui,
                npc_entity_id: near_dialog_interactable.to_owned(),
                dialogue_id: dialogue.dialogue_id.clone(),
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
        let mut dialogue_render_commands = Vec::new();
        let mut new_dialogue_state = None;
        if let DialogueState::Npc {
            render_position,
            npc_entity_id,
            dialogue_id,
        } = &ui_state.dialogue_state
        {
            let dialogue_rect = UIElement::new_rect(
                Point2::new(render_position.x + 0.01, render_position.y + 0.05),
                Point2::new(0.16, 0.05),
            );
            let dialogue_render_command = RenderCommand::Model {
                layer: 150,
                ui_element: dialogue_rect,
                model_id: "black_square".to_owned(),
            };
            dialogue_render_commands.push(dialogue_render_command);

            match frame_state
                .gui
                .button_handle(window, dialogue_rect, input)
            {
                UserAction::None => {}
                UserAction::Hover => {}
                UserAction::LeftClick => {}
                UserAction::RightClick => {}
            }

            let dialogue_manager = DialogueManager::new();
            let dialogue_text = dialogue_manager.get_dialogue(dialogue_id).unwrap();

            let dialogue_text_render_command = frame_state.gui.build_text_render_command(
                300,
                dialogue_rect,
                &dialogue_text.text,
                [0.8, 0.8, 0.0],
            );
            dialogue_render_commands.push(dialogue_text_render_command);

            let close_button_rect =
                dialogue_rect.inner_rect_maintain_ratio_x(Point2::new(0.9, 0.05), 0.10);

            let close_button_render_command = RenderCommand::Model {
                layer: 310,
                ui_element: close_button_rect,
                model_id: "close_button".to_owned(),
            };
            dialogue_render_commands.push(close_button_render_command);
            match frame_state.gui.button_handle(
                window,
                close_button_rect,
                input,
            ) {
                UserAction::None | UserAction::RightClick => {}
                UserAction::Hover => {
                    // Feels kinda silly/hacky to overlay hover image
                    let close_button_hover_render_command = RenderCommand::Model {
                        layer: 311,
                        ui_element: close_button_rect,
                        model_id: "close_button_hover".to_owned(),
                    };
                    dialogue_render_commands.push(close_button_hover_render_command);
                    match frame_state.gui.button_handle(
                        window,
                        close_button_rect,
                        input,
                    ) {
                        UserAction::None | UserAction::Hover | UserAction::RightClick => {}
                        UserAction::LeftClick => {
                            new_dialogue_state = Some(DialogueState::Closed);
                        }
                    }
                }
                UserAction::LeftClick => {
                    if !frame_state.handled_left_click {
                        new_dialogue_state = Some(DialogueState::Closed);
                        frame_state.handled_left_click = true;
                    }
                }
            }

            if !PositionManager::in_range(
                game_state
                    .get_position("player")
                    .expect("Player position should exist"),
                game_state
                    .get_position(npc_entity_id)
                    .expect("Interacted NPC position should exist"),
                DIALOGUE_RANGE,
            ) {
                new_dialogue_state = Some(DialogueState::Closed);
            }
        }
        if let Some(new_state) = new_dialogue_state {
            match new_state {
                DialogueState::Closed => {}
                DialogueState::Npc { .. } => {
                    frame_state.gui.render_commands.append(&mut dialogue_render_commands);
                }
            }
            ui_state.dialogue_state = new_state;
        }
        frame_state.gui.render_commands.append(&mut dialogue_render_commands);
    }
}
