use crate::state::frame_state::{ActionEffect, ActionRequest, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::{Closed, Inventory};
use crate::state::ui_state::{MenuState, UIElement, UIState, UserAction};
use crate::systems::item_placement_system::ItemPlacementSystem;
use cgmath::Point2;
use std::sync::Arc;
use winit::window::Window;

pub struct InventorySystem {}

impl InventorySystem {
    pub fn handle_inventory(
        window: &Arc<Window>,
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

        let inventory_items = game_state.get_in_storages(&"player".to_string());

        for (entity, in_storage) in inventory_items.iter() {
            let storable = game_state.storable_components.get(*entity).unwrap();
            let item_image = game_state.get_graphics_inventory(entity).unwrap();

            let left = in_storage.position_x as f32 / inventory_ecs.number_of_columns as f32;
            let right = left + storable.shape.width as f32 / inventory_ecs.number_of_columns as f32;
            let top = in_storage.position_y as f32 / inventory_ecs.number_of_rows as f32;
            let bottom = top + storable.shape.height as f32 / inventory_ecs.number_of_rows as f32;

            let image_element = inventory_window
                .rect
                .inner_rect(Point2::new(left, top), Point2::new(right, bottom));

            match frame_state.gui.image_button(
                window,
                150,
                image_element,
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

                    ui_state.menu_state = Inventory {
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
        window: &Arc<Window>,
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let old_menu_state = ui_state.menu_state.clone();
        ui_state.menu_state = Self::handle_inventory_menu_state(
            window,
            game_state,
            old_menu_state,
            input,
            frame_state,
        );
    }

    fn handle_inventory_menu_state(
        window: &Arc<Window>,
        game_state: &mut GameState,
        menu_state: MenuState,
        input: &Input,
        frame_state: &mut FrameState,
    ) -> MenuState {
        let mut new_menu_state = menu_state.clone();

        if let Inventory {
            mouse_position,
            item,
        } = menu_state
        {
            // Drop button
            let drop_button_rect = UIElement::new_rect(
                Point2::new(mouse_position.x + 0.015, mouse_position.y + 0.005),
                Point2::new(0.065, 0.025),
            );

            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state.gui.color_button(
                window,
                200,
                drop_button_rect,
                input,
                "black".to_string(),
            ) {
                UserAction::None => {}
                UserAction::Hover => {
                    text_color = [0.8, 0.8, 0.0];
                }
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        return new_menu_state;
                    }
                    ItemPlacementSystem::place_item(
                        game_state,
                        &mut frame_state.action_effects,
                        &item,
                    );
                    new_menu_state = Closed;
                    frame_state.handled_left_click = true;
                }
                UserAction::RightClick => {}
            }

            frame_state.gui.text(
                300,
                drop_button_rect.inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                "Drop item".to_string(),
                text_color,
            );

            // Examine button
            if game_state.description_components.contains_key(&item) {
                let examine_button_rect = UIElement::new_rect(
                    Point2::new(mouse_position.x + 0.015, mouse_position.y + 0.055),
                    Point2::new(0.065, 0.025),
                );

                let mut text_color = [0.8, 0.8, 0.8];
                match frame_state.gui.color_button(
                    window,
                    200,
                    examine_button_rect,
                    input,
                    "black".to_string(),
                ) {
                    UserAction::None => {}
                    UserAction::Hover => {
                        text_color = [0.8, 0.8, 0.0];
                    }
                    UserAction::LeftClick => {
                        if frame_state.handled_left_click {
                            return new_menu_state;
                        }
                        let examine_text = game_state.description_components.get(&item).unwrap();
                        frame_state.action_effects.push(ActionEffect::Examine {
                            text: examine_text.text.clone(),
                        });

                        new_menu_state = Closed;
                        frame_state.handled_left_click = true;
                    }
                    UserAction::RightClick => {}
                }

                frame_state.gui.text(
                    300,
                    examine_button_rect
                        .inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                    "Examine item".to_string(),
                    text_color,
                );
            }
        }
        new_menu_state
    }
}
