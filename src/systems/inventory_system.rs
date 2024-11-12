use crate::input::Input;
use crate::state::frame_state::{ActionRequest, FrameState};
use crate::state::game_state::GameState;
use crate::state::ui_state::{Rect, UIState, UserAction};
use cgmath::Point2;

pub struct InventorySystem {}

impl InventorySystem {
    pub fn handle_inventory(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        frame_state: &mut FrameState,
    ) {
        let inventory_window = ui_state.windows.get_mut("inventory").unwrap();
        if input.i_pressed.is_toggled_on() {
            inventory_window.is_visible = !inventory_window.is_visible;
        }

        let inventory_graphics = game_state
            .get_graphics_inventory(&"sword1".to_string()) // TODO inventory graphics
            .unwrap();
        if !inventory_window.is_visible {
            return;
        }

        frame_state.gui.image(
            inventory_window.rect,
            inventory_graphics.model_id.to_string(),
        );

        let inventory_ecs = game_state.get_storage(&"player".to_string()).unwrap();
        let inventory_width =
            inventory_window.rect.bottom_right.x - inventory_window.rect.top_left.x;
        let inventory_height =
            inventory_window.rect.bottom_right.y - inventory_window.rect.top_left.y;
        let item_picture_scale_x = inventory_width / f32::from(inventory_ecs.number_of_columns);
        let item_picture_scale_y = inventory_height / f32::from(inventory_ecs.number_of_rows);

        let inventory_items = game_state.get_in_storages(&"player".to_string());
        for (entity, in_storage) in inventory_items.iter() {
            let storable = game_state.storable_components.get(*entity).unwrap();
            let item_image = game_state.get_graphics_inventory(entity).unwrap();
            let top_left = Point2::new(
                inventory_window.rect.top_left.x
                    + in_storage.position_x as f32 * item_picture_scale_x,
                inventory_window.rect.top_left.y
                    + in_storage.position_y as f32 * item_picture_scale_y,
            );
            let bottom_right = Point2::new(
                inventory_window.rect.top_left.x
                    + (in_storage.position_x + storable.shape.width) as f32 * item_picture_scale_x,
                inventory_window.rect.top_left.y
                    + (in_storage.position_y + storable.shape.height) as f32 * item_picture_scale_y,
            );

            match frame_state.gui.image_button(
                Rect::new(top_left, bottom_right),
                item_image.model_id.to_string(),
                input,
            ) {
                UserAction::None => {}
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        continue;
                    }
                    frame_state
                        .action_requests
                        .push(ActionRequest::ItemPlacement {
                            entity: entity.to_string(),
                        });
                    frame_state.handled_left_click = true;
                }
                UserAction::RightClick => {
                    if frame_state.handled_right_click {
                        continue;
                    }
                    // TODO Menu for placing
                    frame_state.handled_right_click = true;
                }
            }
        }
    }
}
