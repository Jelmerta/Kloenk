use crate::state::components::Entity;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::Closed;
use crate::state::ui_state::{MenuState, Rect, UIState, UserAction};
use crate::systems::item_pickup_system::ItemPickupSystem;
use cgmath::Point2;

pub struct ObjectSelectionSystem();

impl ObjectSelectionSystem {
    pub fn handle_object_selection(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let selected_objects = &frame_state.objects_on_cursor;

        if Self::should_open_menu(input, frame_state, selected_objects) {
            ui_state.menu_state = MenuState::World {
                mouse_position: input.mouse_position_ui,
                item: selected_objects.get(0).unwrap().clone(),
            };
        }

        match &ui_state.menu_state {
            MenuState::World {
                mouse_position,
                item,
            } => {
                let object_selection_rect = Rect::new(
                    Point2::new(mouse_position.x - 0.05, mouse_position.y - 0.05),
                    Point2::new(mouse_position.x + 0.15, mouse_position.y + 0.05),
                );

                // TODO Specifically show text to pickup item
                match frame_state.gui.image_button(
                    200,
                    object_selection_rect,
                    "sword_inventory".to_string(),
                    input,
                ) {
                    UserAction::LeftClick => {
                        if frame_state.handled_left_click {
                            return;
                        }
                        ItemPickupSystem::item_pickup(game_state, frame_state, item.clone());
                        ui_state.menu_state = Closed;
                        frame_state.handled_left_click = true;
                    }
                    _ => (),
                }

                frame_state.handled_right_click = true;
            }
            _ => (),
        }
    }

    fn should_open_menu(
        input: &Input,
        frame_state: &FrameState,
        selected_objects: &Vec<Entity>,
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
