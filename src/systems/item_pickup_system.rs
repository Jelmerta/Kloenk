use crate::components::{Entity, Storable};
use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::gui::{Payload, UIElement, UIState};
use crate::input::Input;
use crate::storage_manager::StorageManager;
use cgmath::num_traits::ToPrimitive;
use cgmath::{Point2, Point3};
use std::collections::HashMap;

pub const ITEM_PICKUP_RANGE: f32 = 1.5;

pub struct ItemPickupSystem {}

impl ItemPickupSystem {
    pub fn handle_item_pickup_keyboard(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let player = "player".to_string();

        if input.e_pressed.is_toggled_on() && !frame_state.handled_e_click {
            let near_pickup = PositionManager::find_nearest_pickup(
                &game_state.position_components,
                &game_state.storable_components,
                &game_state.entities,
                &player,
            );

            if near_pickup.is_none() {
                ui_state.action_text.payload =
                    Payload::Text("No item found around you to pick up.".to_string());
                return;
            }
            Self::item_pickup(game_state, ui_state, near_pickup.unwrap());
            frame_state.handled_e_click = true;
        }
    }

    pub fn handle_item_pickup_mouse(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if !input.left_mouse_clicked.is_toggled_on() {
            return;
        }

        if frame_state.handled_left_click {
            return;
        }

        if let Some(nearest_object) = frame_state.get_nearest_object_on_cursor() {
            Self::item_pickup(game_state, ui_state, nearest_object.clone());
        }

        frame_state.handled_left_click = true;
    }

    pub fn handle_item_pickup_menu(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let cursor_ndc = input.mouse_position_ndc;
        let cursor_ui_space = Point2::new(cursor_ndc.x / 2.0 + 0.5, -cursor_ndc.y / 2.0 + 0.5);
        if ui_state.object_menu.is_none() {
            return;
        }
        if !ui_state
            .object_menu
            .as_mut()
            .unwrap()
            .contains(cursor_ui_space)
        {
            return;
        }

        if frame_state.handled_left_click {
            return;
        }

        if !input.left_mouse_clicked.is_toggled_on() {
            return;
        }

        if let Some(mut on_click) = ui_state.object_menu.as_mut().unwrap().on_click.take() {
            on_click(game_state, ui_state, input, frame_state);
            ui_state.object_menu.as_mut().unwrap().on_click = Some(on_click);
        }

        frame_state.handled_left_click = true;
    }

    pub fn pickup_item_object_menu_callback(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        _input: &Input,
        _frame_state: &mut FrameState,
    ) {
        let selected_object = ui_state.selected_objects_for_object_menu.get(0).unwrap(); // Just get first item for now.
        Self::item_pickup(game_state, ui_state, selected_object.clone());
    }

    fn item_pickup(game_state: &mut GameState, ui_state: &mut UIState, near_pickup: Entity) {
        let player = "player".to_string();

        let pickup = game_state.storable_components.get(&near_pickup);
        if pickup.is_none() {
            ui_state.action_text.payload = Payload::Text("That cannot be picked up.".to_string());
            return;
        }

        if !Self::in_range(
            game_state.get_position(&player.clone()).unwrap(),
            game_state.get_position(&near_pickup.clone()).unwrap(),
        ) {
            ui_state.action_text.payload =
                Payload::Text("No item found around you to pick up.".to_string());
            return;
        }

        let inventory = game_state.get_storage(&player).unwrap();
        let inventory_items = StorageManager::get_in_storage(game_state, &player);
        if !StorageManager::has_space(game_state, inventory, &inventory_items, &near_pickup) {
            ui_state.action_text.payload = Payload::Text(
                "There is no space left in your\ninventory to pick up this item.".to_string(),
            );
            return;
        }
        let empty_spot =
            StorageManager::find_empty_spot(game_state, inventory, &inventory_items, &near_pickup)
                .unwrap();

        let x_min = empty_spot.0 as f32 / inventory.number_of_columns as f32;
        let y_min = empty_spot.1 as f32 / inventory.number_of_rows as f32;
        let ui_inventory_item = UIElement::new_image(
            "".to_string(),
            true,
            Point2::new(x_min, y_min),
            Point2::new(
                x_min + pickup.unwrap().shape.width as f32 / inventory.number_of_columns as f32,
                y_min + pickup.unwrap().shape.height as f32 / inventory.number_of_rows as f32,
            ),
            None::<fn(&mut GameState, &mut UIState, &Input, &mut FrameState)>,
        );

        ui_state.action_text.payload = Payload::Text("You pick up the item!".to_string());
        ui_state
            .inventory
            .child_elements
            .insert(near_pickup.clone(), ui_inventory_item);
        game_state.remove_position(&near_pickup.clone());
        game_state.remove_hitbox(&near_pickup.clone());
        game_state.create_in_storage(&player, near_pickup.clone(), empty_spot);
    }

    fn in_range(position1: &Point3<f32>, position2: &Point3<f32>) -> bool {
        PositionManager::distance_2d(position1, position2) < ITEM_PICKUP_RANGE
    }
}

struct PositionManager {}

impl PositionManager {
    pub fn find_nearest_pickup(
        positions: &HashMap<Entity, Point3<f32>>,
        storables: &HashMap<Entity, Storable>,
        entities: &[Entity],
        entity: &Entity,
    ) -> Option<Entity> {
        entities
            .iter()
            .filter(|e| storables.contains_key(e.as_str()))
            .filter(|e| positions.contains_key(e.as_str()))
            .min_by_key(|e| {
                Self::distance_2d(
                    positions.get(entity).unwrap(),
                    positions.get(e.as_str()).unwrap(),
                )
                .round()
                .to_u32()
            })
            .cloned()
    }

    fn distance_2d(position1: &Point3<f32>, position2: &Point3<f32>) -> f32 {
        ((position2.x - position1.x).powi(2) + (position2.z - position1.z).powi(2)).sqrt()
    }
}
