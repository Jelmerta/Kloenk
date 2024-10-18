use crate::audio_system::AudioSystem;
use crate::components::{CameraTarget, Entity, Hitbox, ItemShape, Storable, Storage};
use crate::game_state::GameState;
use crate::gui::UIState;
use crate::input::Input;
use cgmath::num_traits::{Float, ToPrimitive};
use cgmath::{ElementWise, InnerSpace, Point3, Vector3, Vector4};
use std::collections::HashMap;

pub struct GameSystem {}

pub const BASE_SPEED: f32 = 0.01;

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;
pub const CAMERA_MOVEMENT_SPEED: f32 = 3.0;
pub const CAMERA_BOTTOM_LIMIT: f32 = 280.0;
pub const CAMERA_TOP_LIMIT: f32 = 350.0;
pub const ITEM_PICKUP_RANGE: f32 = 1.5;

pub struct ItemPickupSystem {}

#[derive(Debug)]
struct Ray {
    origin: Point3<f32>,
    #[allow(dead_code)] // Other algorithm choice might use direction
    direction: Vector3<f32>,
    direction_inverted: Vector3<f32>,
}

impl GameSystem {
    pub fn update(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        audio_system: &mut AudioSystem,
    ) {
        ItemPickupSystem::handle_item_pickup(game_state, ui_state, input);
        Self::handle_item_placement(game_state, ui_state, input);
        Self::handle_inventory(ui_state, input);
        Self::resolve_movement(game_state, input, audio_system);
        Self::update_camera(game_state, input);
        Self::find_world_object_on_cursor(game_state, input);
    }

    fn handle_item_placement(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
    ) {
        if input.left_mouse_clicked.is_toggled_on() {
            let item = StorageManager::find_in_storage(game_state, &"player".to_string());
            if item.is_none() {
                ui_state.text = "No items in inventory to place.".to_string();
                return;
            }
            let item_unwrap = item.unwrap().clone();

            let player_position = game_state.get_position(&"player".to_string()).unwrap();
            let placed_position = Point3 {
                x: player_position.x - 1.1,
                y: player_position.y,
                z: player_position.z - 1.1,
            };

            if !Self::is_placeable_area(game_state, &placed_position) {
                ui_state.text = "Cannot place outside placeable area.".to_string();
                return;
            }

            // Generate a dynamic hitbox for the item to be placed
            let item_hitbox_min = placed_position.sub_element_wise(Point3::new(0.51, 0.51, 0.51));
            let item_hitbox_max = placed_position.add_element_wise(Point3::new(0.51, 0.51, 0.51));
            let item_hitbox = Hitbox {
                box_corner_min: item_hitbox_min,
                box_corner_max: item_hitbox_max,
            };

            let colliding_entities: Vec<Entity> = game_state
                .entities
                .iter()
                .filter(|entity| game_state.hitbox_components.contains_key(entity.as_str()))
                .filter(|entity| game_state.position_components.contains_key(entity.as_str()))
                .filter(|entity| *entity != "player")
                .filter(|entity| {
                    Self::check_collision(
                        game_state.get_hitbox(&(*entity).to_string()).unwrap(),
                        &item_hitbox,
                    )
                })
                .cloned()
                .collect();
            if !colliding_entities.is_empty() {
                ui_state.text =
                    "Found a colliding object.\nNot allowed to place there.".to_string();
                return;
            }

            ui_state.text = "You drop the item.".to_string();
            game_state.create_position(item_unwrap.to_string(), placed_position);
            game_state.create_hitbox(item_unwrap.to_string(), item_hitbox);
            game_state.remove_in_storage(&item_unwrap.to_string());
        }
    }

    fn is_placeable_area(game_state: &GameState, desired_position: &Point3<f32>) -> bool {
        game_state
            .entities
            .iter()
            .filter(|entity| game_state.surface_components.contains(entity.as_str()))
            .filter(|entity| {
                Self::check_in_dimension(
                    desired_position.x,
                    0.0,
                    game_state.get_position(&(*entity).to_string()).unwrap().x,
                    0.5,
                )
            }) // Assume 0.5 as half tile
            .any(|entity| {
                Self::check_in_dimension(
                    desired_position.z,
                    0.0,
                    game_state.get_position(&entity.to_string()).unwrap().z,
                    0.5,
                )
            }) // Assume 0.5 as half tile
    }

    fn handle_inventory(ui_state: &mut UIState, input: &mut Input) {
        if input.i_pressed.is_toggled_on() {
            ui_state.inventory_open = !ui_state.inventory_open;
        }
    }

    fn update_camera(game_state: &mut GameState, input: &mut Input) {
        Self::setup_camera_target(game_state, input);
        Self::setup_camera(game_state);
        let camera = game_state.get_camera_mut("camera").unwrap();
        camera.update_view_projection_matrix();
        camera.update_inverse_matrix();
    }

    fn setup_camera(game_state: &mut GameState) {
        let player = "player".to_string();
        let player_position = *game_state.get_position(&player).unwrap();
        let player_camera = *game_state.get_camera_target(&player).unwrap();

        let rad_x = f32::to_radians(player_camera.rotation_x_degrees);
        let rad_y = f32::to_radians(player_camera.rotation_y_degrees);

        let camera = game_state.get_camera_mut("camera").unwrap();
        camera.eye = Point3 {
            x: player_position.x + player_camera.distance * rad_y.sin() * rad_x.cos(),
            y: player_position.z + player_camera.distance * rad_y.cos(),
            z: player_position.y + player_camera.distance * rad_y.sin() * rad_x.sin(),
        };
        camera.target = Point3 {
            x: player_position.x,
            y: player_position.y,
            z: player_position.z,
        };
        let view_direction = (camera.target - camera.eye).normalize();
        let right = Vector3::unit_y().cross(view_direction).normalize();
        camera.up = view_direction.cross(right).normalize();
    }

    fn setup_camera_target(game_state: &mut GameState, input: &mut Input) {
        let player_camera: &mut CameraTarget = game_state
            .get_camera_target_mut(&"player".to_string())
            .unwrap();

        if input.up_pressed.is_pressed {
            player_camera.rotation_y_degrees += CAMERA_MOVEMENT_SPEED;
        }

        if input.down_pressed.is_pressed {
            player_camera.rotation_y_degrees -= CAMERA_MOVEMENT_SPEED;
        }

        if input.right_pressed.is_pressed {
            player_camera.rotation_x_degrees -= CAMERA_MOVEMENT_SPEED;
        }

        if input.left_pressed.is_pressed {
            player_camera.rotation_x_degrees += CAMERA_MOVEMENT_SPEED;
        }

        // We do this to keep the degrees in range of 0 to 359.99.. which modulo would not do...
        // does this matter though... seems the effect is the same...
        if player_camera.rotation_x_degrees < 0.0 {
            player_camera.rotation_x_degrees += 360.0;
        }

        if player_camera.rotation_x_degrees >= 360.0 {
            player_camera.rotation_x_degrees -= 360.0;
        }

        player_camera.rotation_y_degrees = player_camera
            .rotation_y_degrees
            .clamp(CAMERA_BOTTOM_LIMIT, CAMERA_TOP_LIMIT);

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

    fn resolve_movement(game_state: &mut GameState, input: &Input, audio_system: &mut AudioSystem) {
        let mut movement_speed: f32 = BASE_SPEED;
        if input.left_shift_pressed.is_pressed {
            movement_speed *= 2.5;
        }

        if input.w_pressed.is_pressed {
            let player_position = game_state.get_position(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x - movement_speed,
                y: player_position.y,
                z: player_position.z - movement_speed,
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, audio_system)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                update_hitbox(game_state, desired_position);
            }
        }

        if input.s_pressed.is_pressed {
            let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x + movement_speed,
                y: player_position.y,
                z: player_position.z + movement_speed,
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, audio_system)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                update_hitbox(game_state, desired_position);
            }
        }

        if input.a_pressed.is_pressed {
            let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x - movement_speed,
                y: player_position.y,
                z: player_position.z + movement_speed,
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, audio_system)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                update_hitbox(game_state, desired_position);
            }
        }

        if input.d_pressed.is_pressed {
            let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x + movement_speed,
                y: player_position.y,
                z: player_position.z - movement_speed,
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(game_state, audio_system)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                update_hitbox(game_state, desired_position);
            }
        }
    }

    fn is_walkable(game_state: &GameState, desired_position: &Point3<f32>) -> bool {
        game_state
            .entities
            .iter()
            .filter(|e| game_state.surface_components.contains(e.as_str()))
            .any(|e| {
                Self::check_walkable(
                    desired_position,
                    game_state.position_components.get(e.as_str()).unwrap(),
                )
            })
    }

    fn is_colliding(game_state: &GameState, audio_system: &mut AudioSystem) -> bool {
        let interactable_entities: Vec<&Entity> = game_state
            .entities
            .iter()
            .filter(|entity| {
                entity.as_str() != "player"
                    && game_state.get_hitbox(&(*entity).to_string()).is_some()
                    && game_state.get_position(&(*entity).to_string()).is_some()
            })
            .collect();

        let player_hitbox = game_state.hitbox_components.get("player").unwrap();
        for entity in interactable_entities {
            let entity_hitbox = game_state.get_hitbox(&entity.to_string()).unwrap();

            if Self::check_collision(player_hitbox, entity_hitbox) {
                audio_system.play_sound("bonk");

                return true;
            }
        }

        false
    }

    fn check_walkable(
        desired_position: &Point3<f32>,
        walkable_tile_position: &Point3<f32>,
    ) -> bool {
        let tile_size = 0.5; // Just hardcoded here for now.
        let is_walkable_x = desired_position.x >= walkable_tile_position.x - tile_size
            && walkable_tile_position.x + tile_size >= desired_position.x;

        let is_walkable_z = desired_position.z >= walkable_tile_position.z - tile_size
            && walkable_tile_position.z + tile_size >= desired_position.z;

        is_walkable_x && is_walkable_z
    }

    fn check_collision(bounding_box_one: &Hitbox, bounding_box_two: &Hitbox) -> bool {
        if bounding_box_one.box_corner_max.x < bounding_box_two.box_corner_min.x
            || bounding_box_one.box_corner_min.x > bounding_box_two.box_corner_max.x
        {
            return false;
        }

        if bounding_box_one.box_corner_max.y < bounding_box_two.box_corner_min.y
            || bounding_box_one.box_corner_min.y > bounding_box_two.box_corner_max.y
        {
            return false;
        }

        if bounding_box_one.box_corner_max.z < bounding_box_two.box_corner_min.z
            || bounding_box_one.box_corner_min.z > bounding_box_two.box_corner_max.z
        {
            return false;
        }

        true
    }

    fn check_in_dimension(position1: f32, boundary1: f32, position2: f32, boundary2: f32) -> bool {
        position1 + boundary1 >= position2 - boundary2
            && position2 + boundary2 >= position1 - boundary1
    }

    fn find_world_object_on_cursor(game_state: &mut GameState, input: &mut Input) {
        let camera = game_state.get_camera_mut("camera").unwrap();

        let ray_clip_near = Vector4::new(
            input.mouse_position_ndc.x,
            input.mouse_position_ndc.y,
            0.0, // ndc z goes from 0.0-1.0
            1.0,
        );

        // Very important! Orthographic projection means rays are parallel, and do not come from a single origin point.
        // This means that we cannot do something like ray_near - camera.eye but instead need two points in the world space to figure out the direction.
        // A perspective projection would not need this as it would have a single origin point.
        let ray_clip_far = Vector4::new(
            input.mouse_position_ndc.x,
            input.mouse_position_ndc.y,
            0.999999, // Hm, 1.0 (far plane) does not work
            1.0,
        );

        let point_near = camera.view_projection_matrix_inverted * ray_clip_near;
        let point_near_normalized =
            Point3::new(point_near.x, point_near.y, point_near.z) / point_near.w;
        let point_far = camera.view_projection_matrix_inverted * ray_clip_far;
        let point_far_normalized = Point3::new(point_far.x, point_far.y, point_far.z) / point_far.w;

        let ray_world = (point_far_normalized - point_near_normalized).normalize();

        let ray_direction_inverted = (1.0 / ray_world).normalize();

        let ray = Ray {
            origin: point_near_normalized, // Not camera origin! (orthographic)
            direction: ray_world,
            direction_inverted: ray_direction_inverted,
        };

        let mut found_objects = Vec::new();
        for (entity, hitbox) in game_state.hitbox_components.iter() {
            if Self::intersection(&ray, hitbox) {
                found_objects.push(entity.clone());
            }
        }
        // log::warn!("{:?}", found_objects);

        // TODO Find nearest?
    }

    fn intersection(ray: &Ray, hitbox: &Hitbox) -> bool {
        let mut t_min = 0.0;
        let mut t_max = f32::infinity();

        for dimension in 0..3 {
            let t1 = (hitbox.box_corner_min[dimension] - ray.origin[dimension])
                * ray.direction_inverted[dimension];
            let t2 = (hitbox.box_corner_max[dimension] - ray.origin[dimension])
                * ray.direction_inverted[dimension];

            t_min = f32::max(t_min, f32::min(t1, t2));
            t_max = f32::min(t_max, f32::max(t1, t2));
        }

        t_min < t_max
    }
}

fn update_hitbox(game_state: &mut GameState, position: Point3<f32>) {
    let player_hitbox = Hitbox {
        box_corner_min: position.sub_element_wise(Point3::new(0.51, 0.51, 0.51)),
        box_corner_max: position.add_element_wise(Point3::new(0.51, 0.51, 0.51)),
    };
    game_state.hitbox_components.remove("player");
    game_state
        .hitbox_components
        .insert("player".to_string(), player_hitbox);
}

impl ItemPickupSystem {
    fn handle_item_pickup(game_state: &mut GameState, ui_state: &mut UIState, input: &mut Input) {
        // mut input just for is
        // toggled on. could possibly be changed
        if input.e_pressed.is_toggled_on() {
            let player = "player".to_string();
            let near_pickup = PositionManager::find_nearest_pickup(
                &game_state.position_components,
                &game_state.storable_components,
                &game_state.entities,
                &player,
            );
            if near_pickup.is_none() {
                ui_state.text = "No item found around you to pick up.".to_string();
                return;
            }
            let near_pickup = near_pickup.unwrap();

            if !Self::in_range(
                game_state.get_position(&player.clone()).unwrap(),
                game_state.get_position(&near_pickup.clone()).unwrap(),
            ) {
                ui_state.text = "No item found around you to pick up.".to_string();
                return;
            }

            let inventory = game_state.get_storage(&player).unwrap();
            let inventory_items = StorageManager::get_in_storage(game_state, &player);
            if !StorageManager::has_space(game_state, inventory, &inventory_items, &near_pickup) {
                ui_state.text =
                    "There is no space left in your\ninventory to pick up this item.".to_string();
                return;
            }
            let empty_spot = StorageManager::find_empty_spot(
                game_state,
                inventory,
                &inventory_items,
                &near_pickup,
            )
            .unwrap();

            ui_state.text = "You pick up the item!".to_string();
            game_state.remove_position(&near_pickup.clone());
            game_state.remove_hitbox(&near_pickup.clone());
            game_state.create_in_storage(&player, near_pickup.clone(), empty_spot);
        }
    }

    fn in_range(position1: &Point3<f32>, position2: &Point3<f32>) -> bool {
        PositionManager::distance_2d(position1, position2) < ITEM_PICKUP_RANGE
    }
}

struct StorageManager {}

impl StorageManager {
    pub fn has_space(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
        near_pickup: &Entity,
    ) -> bool {
        Self::find_empty_spot(game_state, storage, in_storage_entities, near_pickup).is_some()
    }

    pub fn find_empty_spot(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
        near_pickup: &Entity,
    ) -> Option<(u8, u8)> {
        let dynamic_storage =
            Self::generate_dynamic_storage_space(game_state, storage, in_storage_entities);
        let item_shape = &game_state
            .storable_components
            .get(near_pickup)
            .unwrap()
            .shape;
        let mut padded_storage = vec![vec![true; 12]; 12];
        for x in 0..dynamic_storage.len() {
            for y in 0..dynamic_storage.len() {
                padded_storage[y][x] = dynamic_storage[y][x];
            }
        }

        for row in 0..storage.number_of_rows {
            for column in 0..storage.number_of_columns {
                if Self::check_empty_spot(&padded_storage, row, column, item_shape) {
                    return Some((column, row));
                }
            }
        }
        None
    }

    fn check_empty_spot(
        padded_storage: &[Vec<bool>],
        row: u8,
        column: u8,
        shape: &ItemShape,
    ) -> bool {
        for x in column..column + shape.width {
            for y in row..row + shape.height {
                if padded_storage[y as usize][x as usize] {
                    return false;
                }
            }
        }
        true
    }

    fn generate_dynamic_storage_space(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
    ) -> Vec<Vec<bool>> {
        let mut storage_spots =
            vec![vec![false; storage.number_of_rows.into()]; storage.number_of_columns.into()];

        for in_storage_entity in in_storage_entities {
            let in_storage = game_state
                .in_storage_components
                .get(&(*in_storage_entity).to_string())
                .unwrap();
            let storable = game_state
                .storable_components
                .get(&(*in_storage_entity).to_string())
                .unwrap();
            for x in in_storage.position_x..in_storage.position_x + storable.shape.width {
                for y in in_storage.position_y..in_storage.position_y + storable.shape.height {
                    storage_spots[y as usize][x as usize] = true;
                }
            }
        }
        storage_spots
    }

    pub fn get_in_storage<'a>(game_state: &'a GameState, entity: &Entity) -> Vec<&'a Entity> {
        game_state
            .entities
            .iter()
            .filter(|e| {
                game_state
                    .in_storage_components
                    .contains_key(&(*e).to_string())
            })
            .filter(|e| {
                game_state
                    .in_storage_components
                    .get(&(*e).to_string())
                    .unwrap()
                    .storage_entity
                    == *entity
            })
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
                    .contains_key(&(*e).to_string())
            })
            .filter(|e| {
                game_state
                    .in_storage_components
                    .get(&(*e).to_string())
                    .unwrap()
                    .storage_entity
                    == *entity
            })
            .collect()
    }

    pub fn find_in_storage<'a>(game_state: &'a GameState, entity: &Entity) -> Option<&'a Entity> {
        let storage_entities = StorageManager::get_in_storage_entities(game_state, entity);
        storage_entities.first().copied()
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
