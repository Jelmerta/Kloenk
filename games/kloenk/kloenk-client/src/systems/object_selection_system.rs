use crate::state::components::Entity;
use crate::state::frame_state::{ActionEffect, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::Closed;
use crate::state::ui_state::{MenuState, UIElement, UIState, UserAction};
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
            ui_state.menu_state = MenuState::World {
                mouse_position: input.mouse_position_ui,
                item: selected_objects.first().unwrap().clone(),
            };
            frame_state.handled_right_click = true;
        }

        let mut new_menu_state = ui_state.menu_state.clone();
        if let MenuState::World {
            mouse_position,
            item,
        } = &ui_state.menu_state
        {
            let pickup_menu_rect = UIElement::new_rect(
                Point2::new(mouse_position.x + 0.015, mouse_position.y + 0.005),
                Point2::new(0.065, 0.025),
            );

            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state.gui.color_button(
                window,
                200,
                pickup_menu_rect,
                input,
                "black",
            ) {
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        return;
                    }
                    ItemPickupSystem::item_pickup(game_state, frame_state, item);
                    new_menu_state = Closed;
                    frame_state.handled_left_click = true;
                }
                UserAction::None => {}
                UserAction::Hover => text_color = [0.8, 0.8, 0.0],
                UserAction::RightClick => {}
            }
            frame_state.gui.text(
                300,
                pickup_menu_rect.inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                "Pick up item",
                text_color,
            );

            let examine_menu_rect = UIElement::new_rect(
                Point2::new(mouse_position.x + 0.015, mouse_position.y + 0.055),
                Point2::new(0.065, 0.025),
            );
            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state.gui.color_button(
                window,
                200,
                examine_menu_rect,
                input,
                "black",
            ) {
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        return;
                    }
                    let examine_text = game_state.description_components.get(item).unwrap();
                    frame_state.action_effects.push(ActionEffect::Examine {
                        text: examine_text.text.clone(),
                    });
                    new_menu_state = Closed;
                    frame_state.handled_left_click = true;
                }
                UserAction::None => {}
                UserAction::Hover => text_color = [0.8, 0.8, 0.0],
                UserAction::RightClick => {}
            }
            frame_state.gui.text(
                300,
                examine_menu_rect.inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                "Examine item",
                text_color,
            );
        }
        ui_state.menu_state = new_menu_state;
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
