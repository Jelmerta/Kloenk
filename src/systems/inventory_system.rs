use crate::state::frame_state::{ActionRequest, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::Closed;
use crate::state::ui_state::{MenuState, Rect, UIState, UserAction};
use crate::systems::item_placement_system::ItemPlacementSystem;
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
            100,
            inventory_window.rect,
            inventory_graphics.material_id.to_string(),
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
                150,
                Rect::new(top_left, bottom_right),
                item_image.material_id.to_string(),
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

                    ui_state.menu_state = MenuState::Inventory {
                        mouse_position: input.mouse_position_ui,
                        item: entity.to_string(),
                    };

                    frame_state.handled_right_click = true;
                }
                UserAction::Hover => {
                    continue;
                }
            }
        }
    }

    pub fn display_inventory_item_menu(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if let MenuState::Inventory {
            mouse_position,
            item,
        } = &ui_state.menu_state
        {
            log::warn!("what");
            log::warn!("hello {:?}", mouse_position);
            let object_selection_rect = Rect::new(
                Point2::new(mouse_position.x - 0.05, mouse_position.y - 0.05),
                Point2::new(mouse_position.x + 0.15, mouse_position.y + 0.05),
            );

            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state
                .gui
                .color_button(200, object_selection_rect, input)
            {
                UserAction::None => {}
                UserAction::Hover => {
                    log::warn!("hello?? {:?}", mouse_position);
                    text_color = [0.8, 0.8, 0.0];
                }
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        return;
                    }
                    ItemPlacementSystem::place_item(
                        game_state,
                        &mut frame_state.action_effects,
                        item,
                    );
                    ui_state.menu_state = Closed;
                    frame_state.handled_left_click = true;
                }
                UserAction::RightClick => {}
            }

            frame_state.gui.text(
                300,
                object_selection_rect,
                "Drop item".to_string(),
                text_color,
            )
        }
    }
}
