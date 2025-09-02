use crate::state::components::Entity;
use crate::state::frame_state::{ActionEffect, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::Closed;
use crate::state::ui_state::{MenuState, RenderCommand, UIElement, UIState, UserAction};
use crate::systems::item_pickup_system::ItemPickupSystem;
use cgmath::Point2;
use std::sync::Arc;
use winit::window::Window;

pub struct ObjectSelectionSystem();

impl ObjectSelectionSystem {
    pub fn handle_object_selection(
        window: &Arc<Window>,
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let selected_objects = &frame_state.objects_on_cursor;

        if Self::should_open_menu(input, frame_state, selected_objects) {
            ui_state.menu_state = MenuState::WorldAction {
                render_position: input.mouse_position_ui,
                item: selected_objects
                    .first()
                    .expect("Selected objects is not empty")
                    .clone(),
            };
            frame_state.handled_right_click = true;
        }

        let mut object_selection_render_commands = Vec::new();
        let mut new_menu_state = &ui_state.menu_state;
        if let MenuState::WorldAction {
            render_position,
            item,
        } = &ui_state.menu_state
        {
            let pickup_menu_rect = UIElement::new_rect(
                Point2::new(render_position.x + 0.015, render_position.y + 0.005),
                Point2::new(0.065, 0.025),
            );
            let pickup_menu_render_command = RenderCommand::Model {
                layer: 200,
                ui_element: pickup_menu_rect,
                model_id: "black_square".to_owned(),
            };
            object_selection_render_commands.push(pickup_menu_render_command);

            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state
                .gui
                .button_handle(window, pickup_menu_rect, input)
            {
                UserAction::None | UserAction::RightClick => {}
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        return;
                    }
                    ItemPickupSystem::item_pickup(game_state, frame_state, item);
                    new_menu_state = &Closed;
                    frame_state.handled_left_click = true;
                    // TODO just return
                }
                UserAction::Hover => text_color = [0.8, 0.8, 0.0],
            }

            let pickup_text_render_command = frame_state.gui.build_text_render_command(
                300,
                pickup_menu_rect.inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                "Pick up item",
                text_color,
            );
            object_selection_render_commands.push(pickup_text_render_command);

            let examine_menu_rect = UIElement::new_rect(
                Point2::new(render_position.x + 0.015, render_position.y + 0.055),
                Point2::new(0.065, 0.025),
            );
            let examine_menu_render_command = RenderCommand::Model {
                layer: 200,
                ui_element: examine_menu_rect,
                model_id: "black_square".to_owned(),
            };
            object_selection_render_commands.push(examine_menu_render_command);

            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state
                .gui
                .button_handle(window, examine_menu_rect, input)
            {
                UserAction::None | UserAction::RightClick => {}
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        return;
                    }
                    let examine_text = game_state.description_components.get(item).unwrap();
                    frame_state.action_effects.push(ActionEffect::Examine {
                        text: examine_text.text.clone(),
                    });
                    new_menu_state = &Closed;
                    frame_state.handled_left_click = true;
                }
                UserAction::Hover => text_color = [0.8, 0.8, 0.0],
            }
            let examine_text_render_command = frame_state.gui.build_text_render_command(
                300,
                examine_menu_rect.inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                "Examine item",
                text_color,
            );
            object_selection_render_commands.push(examine_text_render_command);
        }
        frame_state
            .gui
            .render_commands
            .append(&mut object_selection_render_commands);
        ui_state.menu_state = new_menu_state.clone(); // TODO prob not needed if return on close, as there's no change?
    }

    fn should_open_menu(
        input: &Input,
        frame_state: &FrameState,
        selected_objects: &[Entity],
    ) -> bool {
        if !input.right_mouse_clicked.is_toggled_on() {
            return false;
        }

        if frame_state.handled_right_click {
            return false;
        }

        if selected_objects.is_empty() {
            return false;
        }
        true
    }
}
