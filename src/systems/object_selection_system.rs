use crate::components::Entity;
use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::gui::{Rect, UIState, UserAction};
use crate::input::Input;
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
        let mut show_object_menu = false;
        if ui_state.object_menu_open {
            show_object_menu = true;
        }

        let selected_objects = &frame_state.objects_on_cursor;

        if Self::should_open_menu(input, frame_state, selected_objects) {
            show_object_menu = true;
            ui_state.object_menu_open = true;
            ui_state.object_menu_mouse_position = input.mouse_position_ui;
            ui_state.selected_objects_for_object_menu = selected_objects.clone();
        }

        if !show_object_menu {
            return;
        }

        let object_selection_rect = Rect::new(
            Point2::new(
                ui_state.object_menu_mouse_position.x - 0.05,
                ui_state.object_menu_mouse_position.y - 0.05,
            ),
            Point2::new(
                ui_state.object_menu_mouse_position.x + 0.15,
                ui_state.object_menu_mouse_position.y + 0.05,
            ),
        );

        // TODO Specifically show text to pickup item
        match frame_state.gui.image_button(
            object_selection_rect,
            "sword_inventory".to_string(),
            input,
        ) {
            UserAction::LeftClick => {
                if frame_state.handled_left_click {
                    return;
                }
                ItemPickupSystem::item_pickup(
                    game_state,
                    frame_state,
                    ui_state
                        .selected_objects_for_object_menu
                        .get(0)
                        .unwrap()
                        .clone(),
                );
                ui_state.object_menu_open = false;
                frame_state.handled_left_click = true;
                // TODO stay open how probably bool on uistate?
            }
            _ => (),
        }

        frame_state.handled_right_click = true;
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
