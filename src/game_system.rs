use std::collections::HashMap;

use cgmath::num_traits::ToPrimitive;

use crate::components::{
    CameraTarget, Entity, Hitbox, Position, Storable, Storage, ItemShape,
};
use crate::game_state::{*};
use crate::gui::UIState;
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
            if !StorageManager::has_space(game_state, inventory, &inventory_items, &near_pickup) {
                // "There is no space left in your inventory to pick up this item."
                return;
            }
            let empty_spot = StorageManager::find_empty_spot(game_state, inventory, &inventory_items, &near_pickup).unwrap();

            game_state.remove_position(near_pickup.clone());
            game_state.create_in_storage(player.clone(), near_pickup.clone(), empty_spot);
        }
    }

    fn in_range(position1: &Position, position2: &Position) -> bool {
        return PositionManager::distance_2d(position1, position2) < ITEM_PICKUP_RANGE;
    }
}

struct StorageManager {}

impl StorageManager {
    pub fn has_space(game_state: &GameState, storage: &Storage, in_storage_entities: &Vec<&Entity>, near_pickup: &Entity) -> bool {
        return Self::find_empty_spot(game_state, storage, in_storage_entities, near_pickup).is_some();
    }

    pub fn find_empty_spot(game_state: &GameState, storage: &Storage, in_storage_entities: &Vec<&Entity>, near_pickup: &Entity) -> Option<(u8, u8)> {
        let dynamic_storage = Self::generate_dynamic_storage_space(game_state, storage, in_storage_entities);
        let item_shape = &game_state.storable_components.get(near_pickup).unwrap().shape;
        let mut padded_storage = vec![vec![true; 12]; 12];
        for x in 0..dynamic_storage.len() {
            for y in 0..dynamic_storage.len() {
                padded_storage[y][x] = dynamic_storage[y][x];
            }
        }

        for row in 0..storage.number_of_rows {
            for column in 0..storage.number_of_columns {
                if Self::check_empty_spot(&padded_storage, row, column, item_shape.clone()) {
                    return Some((column, row));
                }
            }
        }
        return None;
    }
    
    fn check_empty_spot(padded_storage: &Vec<Vec<bool>>, row: u8, column: u8, shape: ItemShape) -> bool {
        for x in column..column + shape.width {
            for y in row..row + shape.height {
                if padded_storage[y as usize][x as usize] == true {
                    return false;
                }
            }
        }
        return true;
    }

    fn generate_dynamic_storage_space(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
    ) -> Vec<Vec<bool>> {
        let mut storage_spots =
            vec![vec![false; storage.number_of_rows.into()]; storage.number_of_columns.into()];

        for in_storage_entity in in_storage_entities {
            let in_storage = game_state.in_storage_components.get(&in_storage_entity.to_string()).unwrap();
            let storable = game_state.storable_components.get(&in_storage_entity.to_string()).unwrap();
            for x in in_storage.position_x..in_storage.position_x + storable.shape.width {
                for y in in_storage.position_y..in_storage.position_y + storable.shape.height {
                    storage_spots[y as usize][x as usize] = true
                }
            }
        }
        storage_spots
    }

    pub fn get_in_storage<'a>(game_state: &'a GameState, entity: &Entity) -> Vec<&'a Entity> {
        game_state
            .entities
            .iter()
            .filter(|e| game_state.in_storage_components.contains_key(&e.to_string()))
            .filter(|e| game_state.in_storage_components.get(&e.to_string()).unwrap().storage_entity == entity.to_string())
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

    fn distance_2d(position1: &Position, position2: &Position) -> f32 {
        return ((position2.x - position1.x).powi(2) + (position2.y - position1.y).powi(2)).sqrt();
    }
}

impl GameSystem {
    pub fn update(game_state: &mut GameState, ui_state: &mut UIState, input: &mut Input) {
        ItemPickupSystem::handle_item_pickup(game_state, input);
        Self::handle_item_placement(game_state, input);
        Self::handle_inventory(ui_state, input);
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

            if !Self::is_placeable(game_state, &placed_position) {
                // If desired position cannot be placed on, we abort.
                return;
            }

            let item_hitbox = game_state.get_hitbox(item_unwrap.to_string()).unwrap();

            // TODO inefficient loop over all entities and checking collision
            let colliding_entities: Vec<Entity> = game_state
                .entities
                .iter()
                .filter(|e| game_state.hitbox_components.contains_key(e.as_str()))
                .filter(|e| game_state.position_components.contains_key(e.as_str()))
                .filter(|e| e.to_string() != "player")
                .filter(|e| {
                    Self::check_collision(
                        &game_state.get_position(e.to_string()).unwrap(),
                        &game_state.get_hitbox(e.to_string()).unwrap(),
                        &placed_position,
                        &item_hitbox,
                    )
                })
                .cloned()
                .collect();
            if !colliding_entities.is_empty() {
                // Found a colliding object. Not allowed.
                return;
            }

            game_state.create_position(item_unwrap.to_string(), placed_position);
            game_state.remove_in_storage(&item_unwrap.to_string());
        }
    }

    fn is_placeable(game_state: &GameState, desired_position: &Position) -> bool {
        game_state.entities
            .iter()
            .filter(|entity| game_state.surface_components.contains_key(entity.as_str()))
            .filter(|entity| Self::check_in_dimension(desired_position.x, 0.0, game_state.get_position(entity.to_string()).unwrap().x, 0.5)) // Assume 0.5 as half tile 
            .filter(|entity| Self::check_in_dimension(desired_position.y, 0.0, game_state.get_position(entity.to_string()).unwrap().y, 0.5)) // Assume 0.5 as half tile 
            .next()
            .is_some()
    }

    fn handle_inventory(ui_state: &mut UIState, input: &mut Input) {
        if input.i_pressed.is_toggled_on() {
            ui_state.inventory_open = !ui_state.inventory_open;
        }
    }

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
        if input.left_shift_pressed.is_pressed {
            movement_speed *= 2.5;
        }

        if input.w_pressed.is_pressed {
            let player_position = game_state.get_position("player".to_string()).unwrap();
            let desired_position = Position {
                x: player_position.x.clone() - movement_speed,
                y: player_position.y.clone() - movement_speed,
                z: player_position.z.clone(),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, &desired_position)
            {
                let player_position = game_state.get_position_mut("player".to_string()).unwrap();
                *player_position = desired_position;
            }
        }

        if input.s_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            let desired_position = Position {
                x: player_position.x.clone() + movement_speed,
                y: player_position.y.clone() + movement_speed,
                z: player_position.z.clone(),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, &desired_position)
            {
                let player_position = game_state.get_position_mut("player".to_string()).unwrap();
                *player_position = desired_position;
            }
        }

        if input.a_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            let desired_position = Position {
                x: player_position.x.clone() - movement_speed,
                y: player_position.y.clone() + movement_speed,
                z: player_position.z.clone(),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, &desired_position)
            {
                let player_position = game_state.get_position_mut("player".to_string()).unwrap();
                *player_position = desired_position;
            }
        }

        if input.d_pressed.is_pressed {
            let player_position = game_state.get_position_mut("player".to_string()).unwrap();
            let desired_position = Position {
                x: player_position.x.clone() + movement_speed,
                y: player_position.y.clone() - movement_speed,
                z: player_position.z.clone(),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, &desired_position)
            {
                let player_position = game_state.get_position_mut("player".to_string()).unwrap();
                *player_position = desired_position;
            }
        }
    }

    fn is_walkable(game_state: &GameState, desired_position: &Position) -> bool {
        game_state
            .entities
            .iter()
            .filter(|e| game_state.surface_components.contains_key(e.as_str()))
            .filter(|e| {
                Self::check_walkable(
                    desired_position,
                    &game_state.position_components.get(e.as_str()).unwrap(),
                )
            })
            .next()
            .is_some()
    }

    fn is_colliding(game_state: &GameState, desired_position: &Position) -> bool {
        let interactible_entities: Vec<&Entity> = game_state
            .entities
            .iter()
            .filter(|entity| {
                entity.as_str() != "player"
                    && game_state.get_hitbox(entity.to_string()).is_some()
                    && game_state.get_position(entity.to_string()).is_some()
            })
            .collect();

        let player_hitbox = game_state.get_hitbox("player".to_string()).unwrap();

        for entity in interactible_entities {
            let entity_position = game_state.get_position(entity.to_string()).unwrap();
            let entity_hitbox = game_state.get_hitbox(entity.to_string()).unwrap();
            if Self::check_collision(
                desired_position,
                player_hitbox,
                entity_position,
                entity_hitbox,
            ) {
                return true;
            }
        }
        return false;
    }

    fn check_walkable(desired_position: &Position, walkable_tile_position: &Position) -> bool {
        let tile_size = 0.5; // Just hardcoded here for now.
        let is_walkable_x = desired_position.x
            >= walkable_tile_position.x - tile_size
            && walkable_tile_position.x + tile_size
                >= desired_position.x;

        let is_walkable_y = desired_position.y
            >= walkable_tile_position.y - tile_size
            && walkable_tile_position.y + tile_size
                >= desired_position.y;

        return is_walkable_x && is_walkable_y;
    }

    fn check_collision(
        position1: &Position,
        hitbox1: &Hitbox,
        position2: &Position,
        hitbox2: &Hitbox,
    ) -> bool {
        let is_collision_x = Self::check_in_dimension(position1.x, hitbox1.hitbox, position2.x, hitbox2.hitbox);
        let is_collision_y = Self::check_in_dimension(position1.y, hitbox1.hitbox, position2.y, hitbox2.hitbox);
        let is_collision_z = Self::check_in_dimension(position1.z, hitbox1.hitbox, position2.z, hitbox2.hitbox);

        return is_collision_x && is_collision_y && is_collision_z;
    }
    
    fn check_in_dimension(position1: f32, boundary1: f32, position2: f32, boundary2: f32) -> bool {
        position1 + boundary1 >= position2 - boundary2 && position2 + boundary2 >= position1 - boundary1
    }
}
