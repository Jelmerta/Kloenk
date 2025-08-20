use crate::state::frame_state::{ActionEffect, ActionRequest, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::MenuState::{Closed, Inventory};
use crate::state::ui_state::{MenuState, RenderCommand, UIElement, UIState, UserAction};
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
            .get_graphics_inventory("sword1") // TODO inventory graphics
            .unwrap();
        if !inventory_window.is_visible {
            return;
        }

        frame_state
            .gui
            .image(100, inventory_window.rect, &inventory_graphics.material_id);

        let inventory_ecs = game_state.get_storage("player").unwrap();

        let inventory_items = game_state.get_in_storages("player");

        // TODO besides rendering, we can stop checking user input? mouse click can only be on one location?
        for (entity, in_storage) in &inventory_items {
            let storable = game_state.storable_components.get(entity.as_str()).unwrap();
            let item_image = game_state.get_graphics_inventory(entity).unwrap();

            let left = in_storage.position_x as f32 / inventory_ecs.number_of_columns as f32;
            let right = left + storable.shape.width as f32 / inventory_ecs.number_of_columns as f32;
            let top = in_storage.position_y as f32 / inventory_ecs.number_of_rows as f32;
            let bottom = top + storable.shape.height as f32 / inventory_ecs.number_of_rows as f32;

            let image_element = inventory_window
                .rect
                .inner_rect(Point2::new(left, top), Point2::new(right, bottom));
            let inventory_item_command = RenderCommand::Texture {
                layer: 150,
                ui_element: image_element,
                model_id: item_image.material_id.clone(),
            };
            frame_state.gui.render_commands.push(inventory_item_command);

            match frame_state.gui.button_handle(
                window,
                image_element,
                input,
            ) {
                UserAction::None | UserAction::Hover => {}
                UserAction::LeftClick => {
                    if frame_state.handled_left_click {
                        continue;
                    }
                    frame_state
                        .action_requests
                        .push(ActionRequest::ItemPlacement {
                            entity: (*entity).clone(),
                        });
                    frame_state.handled_left_click = true;
                }
                UserAction::RightClick => {
                    if frame_state.handled_right_click {
                        continue;
                    }

                    ui_state.menu_state = Inventory {
                        render_position: input.mouse_position_ui,
                        item: (*entity).clone(),
                    };

                    frame_state.handled_right_click = true;
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
        ui_state.menu_state = Self::handle_inventory_menu_state(
            window,
            game_state,
            ui_state,
            input,
            frame_state,
        );
    }

    // TODO maybe some gui.start_window() and commit() to create whole windows in place? too much logic around now. since we only know at the end if we should render the window or not. could have been closed
    fn handle_inventory_menu_state(
        window: &Arc<Window>,
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) -> MenuState {
        let mut new_menu_state = ui_state.menu_state.clone();

        // If inventory closes, we do not add the render commands
        let mut inventory_render_commands = Vec::new();

        if let Inventory {
            render_position,
            item,
        } = &ui_state.menu_state
        {
            let drop_button_rect = UIElement::new_rect(
                Point2::new(render_position.x + 0.015, render_position.y + 0.005),
                Point2::new(0.065, 0.025),
            );
            let drop_button_render_command = RenderCommand::Texture {
                layer: 200,
                ui_element: drop_button_rect,
                model_id: "black_square".to_owned(),
            };
            inventory_render_commands.push(drop_button_render_command);

            let mut text_color = [0.8, 0.8, 0.8];
            match frame_state
                .gui
                .button_handle(window, drop_button_rect, input)
            {
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

            let drop_item_text_render_command = frame_state.gui.build_text_render_command(
                300,
                drop_button_rect.inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                "Drop item",
                text_color,
            );
            inventory_render_commands.push(drop_item_text_render_command);

            // Examine button
            if game_state.description_components.contains_key(item) {
                let examine_button_rect = UIElement::new_rect(
                    Point2::new(render_position.x + 0.015, render_position.y + 0.055),
                    Point2::new(0.065, 0.025),
                );
                let examine_render_command = RenderCommand::Texture {
                    layer: 200,
                    ui_element: examine_button_rect,
                    model_id: "black_square".to_owned(),
                };
                inventory_render_commands.push(examine_render_command);

                let mut text_color = [0.8, 0.8, 0.8];
                match frame_state
                    .gui
                    .button_handle(window, examine_button_rect, input)
                {
                    UserAction::None | UserAction::RightClick => {}
                    UserAction::Hover => {
                        text_color = [0.8, 0.8, 0.0];
                    }
                    UserAction::LeftClick => {
                        if frame_state.handled_left_click {
                            return new_menu_state;
                        }
                        let examine_text = game_state.description_components.get(item).unwrap();
                        frame_state.action_effects.push(ActionEffect::Examine {
                            text: examine_text.text.clone(),
                        });

                        new_menu_state = Closed;
                        frame_state.handled_left_click = true;
                    }
                }

                let examine_text_render_command = frame_state.gui.build_text_render_command(
                    300,
                    examine_button_rect
                        .inner_rect(Point2::new(0.01, 0.01), Point2::new(0.99, 0.99)),
                    "Examine item",
                    text_color,
                );
                inventory_render_commands.push(examine_text_render_command);
            }
        }

        match &new_menu_state {
            Closed | MenuState::World { .. } => {}
            Inventory { .. } => {
                frame_state
                    .gui
                    .render_commands
                    .append(&mut inventory_render_commands);
            }
        }

        new_menu_state
    }
}
