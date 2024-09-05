use std::collections::HashMap;

use cgmath::num_traits::ToPrimitive;

use crate::components::{CameraTarget, Entity, Hitbox, InStorage, Position, Storable, Storage};
use crate::game_state::*;
use crate::input::Input;

pub struct GameSystem {}

pub const BASE_SPEED: f32 = 0.01;

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;
pub const CAMERA_MOVEMENT_SPEED: f32 = 3.0;
pub const CAMERA_BOTTOM_LIMIT: f32 = 280.0;
pub const CAMERA_TOP_LIMIT: f32 = 350.0;
pub const ITEM_PICKUP_RANGE: f32 = 1.5;

pub struct ItemPickupSystem {}

impl ItemPickupSystem {
    fn handle_item_pickup(game_state: &mut GameState, input: &mut Input) {
        // mut input just for is
        // toggled on. could possibly be changed
        if input.e_pressed.is_toggled_on() {
            let player = "player".to_string();
            let near_pickup = PositionManager::find_nearest_pickup(
                &game_state.position_components,
                &game_state.storable_components,
                &game_state.entities,
                player.clone(),
            );
            if near_pickup.is_none() {
                // "No item around in the world to pick up."
                return;
            }
            let near_pickup = near_pickup.unwrap();

            if !Self::in_range(
                game_state.get_position(player.clone()).unwrap(),
                game_state.get_position(near_pickup.clone()).unwrap(),
            ) {
                // "No item around in the world to pick up."
                return;
            }

            let inventory = game_state.get_storage(player.clone()).unwrap();
            let inventory_items = StorageManager::get_in_storage(game_state, &player);
            if !StorageManager::has_space(inventory, &inventory_items) {
                // "There is no space left in your inventory to pick up this item."
                return;
            }
            let empty_spot = StorageManager::find_empty_spot(inventory, &inventory_items).unwrap();

            // WorldManager? Remove object:
            // 0. Unregister world component?
            // 0.5 Delete world component?
            //
            // InventoryManager? Add item:
            // 1. Create inventory item component for entity
            // 2. Register component
            // 3. Link component to inventory?

            // TODO unregister and register components instead?
            game_state.remove_position(near_pickup.clone());
            // Maybe use like an InventoryManager or smth? something that deals with
            // managing component data
            game_state.create_in_storage(player.clone(), near_pickup.clone(), empty_spot);
        }
    }

    fn in_range(position1: &Position, position2: &Position) -> bool {
        return PositionManager::distance_2d(position1, position2) < ITEM_PICKUP_RANGE;
    }
}

struct StorageManager {}

impl StorageManager {
    pub fn has_space(storage: &Storage, in_storages: &Vec<&InStorage>) -> bool {
        return Self::find_empty_spot(storage, in_storages).is_some();
    }

    pub fn find_empty_spot(storage: &Storage, in_storages: &Vec<&InStorage>) -> Option<(u8, u8)> {
        let dynamic_storage = Self::generate_dynamic_storage_space(storage, in_storages);
        for row in 0..storage.number_of_rows {
            for column in 0..storage.number_of_columns {
                if !dynamic_storage[row as usize][column as usize] {
                    return Some((row, column));
                }
            }
        }
        return None;
    }

    fn generate_dynamic_storage_space(
        storage: &Storage,
        in_storages: &Vec<&InStorage>,
    ) -> Vec<Vec<bool>> {
        let mut storage_spots =
            vec![vec![false; storage.number_of_rows.into()]; storage.number_of_columns.into()];

        for in_storage in in_storages {
            storage_spots[in_storage.position_y as usize][in_storage.position_x as usize] = true
        }
        storage_spots
    }

    pub fn get_in_storage<'a>(game_state: &'a GameState, entity: &Entity) -> Vec<&'a InStorage> {
        game_state
            .entities
            .iter()
            .filter_map(|e| game_state.in_storage_components.get(&e.to_string()))
            .filter(|in_storage| in_storage.storage_entity == entity.to_string())
            .collect()
    }

    pub fn get_in_storage_entities<'a>(
        game_state: &'a GameState,
        entity: &Entity,
    ) -> Vec<&'a Entity> {
        game_state
            .entities
            .iter()
            .filter(|e| {
                game_state
                    .in_storage_components
                    .get(&e.to_string())
                    .is_some()
            })
            .filter(|e| {
                game_state
                    .in_storage_components
                    .get(&e.to_string())
                    .unwrap()
                    .storage_entity
                    == entity.to_string()
            })
            .collect()
    }

    pub fn find_in_storage(game_state: &GameState, entity: Entity) -> Option<&Entity> {
        let storage_entities = StorageManager::get_in_storage_entities(game_state, &entity);
        storage_entities.first().copied()
    }
}

struct PositionManager {}

impl PositionManager {
    pub fn find_nearest_pickup(
        positions: &HashMap<Entity, Position>,
        storables: &HashMap<Entity, Storable>,
        entities: &Vec<Entity>,
        entity: Entity,
    ) -> Option<Entity> {
        entities
            .iter()
            .filter(|e| storables.contains_key(e.as_str()))
            .filter(|e| positions.contains_key(e.as_str()))
            .min_by_key(|e| {
                Self::distance_2d(
                    positions.get(&entity).unwrap(),
                    positions.get(e.as_str()).unwrap(),
                )
                .round()
                .to_u32()
            })
            .cloned()
    }

    // fn find_near_item(game_state: &GameState) -> Option<String> {
    //     fn find_near_item(game_state: &GameState) -> Option<&Entity> {
    //         let player_position: &Position = game_state.get_player_const().get_position();
    //         game_state
    //             .get_entities()
    //             .iter()
    //             .filter(|entity| entity.id != "player" && entity.graphics_component.material_id != "grass") // TODO hacky
    //             .min_by_key(|entity| {
    //                 Self::distance_2d(entity.get_position(), player_position)
    //                     .round()
    //                         .to_u32()
    //             })
    //             // .map(|entity| entity.id.clone()); // f32does not have trait ord, for now we just cast.
    //         // return near_id;
    //     }
    // }    // probably should be a method on player or something

    fn distance_2d(position1: &Position, position2: &Position) -> f32 {
        return ((position2.x - position1.x).powi(2) + (position2.y - position1.y).powi(2)).sqrt();
    }
}

#[cfg(tests)]
mod tests {
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

impl GameSystem {
    pub fn update(game_state: &mut GameState, input: &mut Input) {
        ItemPickupSystem::handle_item_pickup(game_state, input);
        Self::handle_item_placement(game_state, input);
        // Self::handle_inventory(game_state, input);
        Self::resolve_movement(game_state, input);
        Self::update_camera(game_state, input);
    }

    fn handle_item_placement(game_state: &mut GameState, input: &mut Input) {
        if input.right_mouse_clicked.is_toggled_on() {
            // Find item to place
            let item = StorageManager::find_in_storage(game_state, "player".to_string());
            if item.is_none() {
                // "No items in inventory"
                return;
            }
            let item_unwrap = item.unwrap().clone();
            // in_storage_entity = in_storage_entity.unwrap().clone();

            let player_position = game_state.get_position("player".to_string()).unwrap();
            let placed_position = Position {
                x: player_position.x - 1.1,
                y: player_position.y - 1.1,
                z: player_position.z,
            };

            game_state.create_position(item_unwrap.to_string(), placed_position);
            game_state.remove_in_storage(&item_unwrap.to_string());
        }
    }

    // fn handle_inventory(game_state: &mut GameState, input: &mut Input) {
    //     if input.i_pressed.is_toggled_on() {
    //         game_state.inventory_toggled = !game_state.inventory_toggled;
    //     }
    // }

    fn update_camera(game_state: &mut GameState, input: &mut Input) {
        let player_camera: &mut CameraTarget =
            game_state.get_camera_mut("player".to_string()).unwrap();

        if input.up_pressed.is_pressed {
            player_camera.rotation_y_degrees =
                player_camera.rotation_y_degrees + CAMERA_MOVEMENT_SPEED;
        }

        if input.down_pressed.is_pressed {
            player_camera.rotation_y_degrees =
                player_camera.rotation_y_degrees - CAMERA_MOVEMENT_SPEED;
        }

        if input.right_pressed.is_pressed {
            player_camera.rotation_x_degrees =
                player_camera.rotation_x_degrees - CAMERA_MOVEMENT_SPEED;
        }

        if input.left_pressed.is_pressed {
            player_camera.rotation_x_degrees =
                player_camera.rotation_x_degrees + CAMERA_MOVEMENT_SPEED;
        }

        // We do this to keep the degrees in range of 0 to 359.99.. which modulo would not do...
        // does this matter though... seems the effect is the same...
        if player_camera.rotation_x_degrees < 0.0 {
            player_camera.rotation_x_degrees += 360.0;
        }

        if player_camera.rotation_x_degrees >= 360.0 {
            player_camera.rotation_x_degrees -= 360.0;
        }

        if player_camera.rotation_y_degrees < CAMERA_BOTTOM_LIMIT {
            player_camera.rotation_y_degrees = CAMERA_BOTTOM_LIMIT;
        }

        if player_camera.rotation_y_degrees >= CAMERA_TOP_LIMIT {
            player_camera.rotation_y_degrees = CAMERA_TOP_LIMIT;
        }

        let normalised_scroll_amount: f32 = -input.scrolled_amount * 0.1;

        if player_camera.distance + normalised_scroll_amount <= MIN_CAMERA_DISTANCE {
            player_camera.distance = MIN_CAMERA_DISTANCE;
        } else if player_camera.distance + normalised_scroll_amount >= MAX_CAMERA_DISTANCE {
            player_camera.distance = MAX_CAMERA_DISTANCE;
        } else {
            player_camera.distance += normalised_scroll_amount;
        }

        input.scrolled_amount = 0.0;
    }

    fn resolve_movement(game_state: &mut GameState, input: &Input) {
        let mut movement_speed: f32 = BASE_SPEED;
        let player_position = game_state.get_position("player".to_string()).unwrap();
        let previous_position = Position {
            x: player_position.x.clone(),
            y: player_position.y.clone(),
            z: player_position.z.clone(),
        };
        if input.left_shift_pressed.is_pressed {
            movement_speed *= 2.5;
        }

        if input.w_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            player_position.x -= movement_speed;
            player_position.y -= movement_speed;
            Self::resolve_collisions(game_state, &previous_position)
        }

        if input.s_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            player_position.x += movement_speed;
            player_position.y += movement_speed;
            Self::resolve_collisions(game_state, &previous_position)
        }

        if input.a_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            player_position.x -= movement_speed;
            player_position.y += movement_speed;
            Self::resolve_collisions(game_state, &previous_position)
        }

        if input.d_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            player_position.x += movement_speed;
            player_position.y -= movement_speed;
            Self::resolve_collisions(game_state, &previous_position)
        }
    }

    fn resolve_collisions(game_state: &mut GameState, previous_position: &Position) {
        let interactible_entities: Vec<&Entity> = game_state
            .entities
            .iter()
            .filter(|entity| entity.as_str() != "player" && game_state.get_hitbox(entity.to_string()).is_some() && game_state.get_position(entity.to_string()).is_some())
            .collect();
        let player_position = game_state.get_position("player".to_string()).unwrap();
        let player_hitbox = game_state.get_hitbox("player".to_string()).unwrap();
        let mut should_update = false;
        for entity in interactible_entities {
            let entity_position = game_state.get_position(entity.to_string()).unwrap();
            let entity_hitbox = game_state.get_hitbox(entity.to_string()).unwrap();
            if Self::check_collision(
                player_position,
                player_hitbox,
                entity_position,
                entity_hitbox,
            ) {
                should_update = true;
            }
        }

        // vile... game state both borrowed as mut and not mut if updated directly... not sure good
        // way to write this code.
        if should_update {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            *player_position =
                // don't match itself
                Position {
                    x: previous_position.x.clone(),
                    y: previous_position.y.clone(),
                    z: previous_position.z.clone(),
                };
            // TODO still need update? position in map? or retrieve from map mut?
        }
    }

    fn check_collision(
        position1: &Position,
        hitbox1: &Hitbox,
        position2: &Position,
        hitbox2: &Hitbox,
    ) -> bool {
        let is_collision_x = position1.x + hitbox1.hitbox >= position2.x - hitbox2.hitbox
            && position2.x + hitbox2.hitbox >= position1.x - hitbox1.hitbox;

        let is_collision_y = position1.y + hitbox1.hitbox >= position2.y - hitbox2.hitbox
            && position2.y + hitbox2.hitbox >= position1.y - hitbox1.hitbox;

        let is_collision_z = position1.z + hitbox1.hitbox >= position2.z - hitbox2.hitbox
            && position2.z + hitbox2.hitbox >= position1.z - hitbox1.hitbox;

        return is_collision_x && is_collision_y && is_collision_z;
    }
}
