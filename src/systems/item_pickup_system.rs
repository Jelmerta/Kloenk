use crate::state::components::{Entity, Storable};
use crate::state::frame_state::{ActionEffect, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::systems::storage_manager::StorageManager;
use cgmath::num_traits::ToPrimitive;
use cgmath::Point3;
use std::collections::HashMap;

pub const ITEM_PICKUP_RANGE: f32 = 1.5;

pub struct ItemPickupSystem {}

impl ItemPickupSystem {
    pub fn handle_item_pickup_keyboard(
        game_state: &mut GameState,
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
                frame_state
                    .action_effects
                    .push(ActionEffect::PickupNoItemInRange);
                return;
            }
            Self::item_pickup(game_state, frame_state, near_pickup.unwrap());
            frame_state.handled_e_click = true;
        }
    }

    pub fn handle_item_pickup_mouse(
        game_state: &mut GameState,
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
            Self::item_pickup(game_state, frame_state, nearest_object.clone());
            frame_state.handled_left_click = true;
        }
    }

    pub fn item_pickup(
        game_state: &mut GameState,
        frame_state: &mut FrameState,
        near_pickup: Entity,
    ) {
        let player = "player".to_string();

        let pickup = game_state.storable_components.get(&near_pickup);
        if pickup.is_none() {
            frame_state
                .action_effects
                .push(ActionEffect::PickupItemNotStorable);
            return;
        }

        let item_position = game_state.get_position(&near_pickup.clone());
        if item_position.is_none() {
            frame_state
                .action_effects
                .push(ActionEffect::PickupNoItemInRange);
            return;
        }

        if !Self::in_range(
            game_state.get_position(&player.clone()).unwrap(),
            item_position.unwrap(),
        ) {
            frame_state
                .action_effects
                .push(ActionEffect::PickupNoItemInRange);
            return;
        }

        let inventory = game_state.get_storage(&player).unwrap();
        let inventory_items = StorageManager::get_in_storage(game_state, &player);
        if !StorageManager::has_space(game_state, inventory, &inventory_items, &near_pickup) {
            frame_state
                .action_effects
                .push(ActionEffect::PickupNoInventorySpace);
            return;
        }
        let empty_spot =
            StorageManager::find_empty_spot(game_state, inventory, &inventory_items, &near_pickup)
                .unwrap();

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
