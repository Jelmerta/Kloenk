use cgmath::num_traits::ToPrimitive;

use crate::game_state::{Entity, GameState, Position};
use crate::input::Input;

pub struct GameSystem {}

pub const BASE_SPEED: f32 = 0.01;

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;
pub const CAMERA_MOVEMENT_SPEED: f32 = 3.0;
pub const CAMERA_BOTTOM_LIMIT: f32 = 280.0;
pub const CAMERA_TOP_LIMIT: f32 = 350.0;
pub const ITEM_PICKUP_RANGE: f32 = 1.5;

impl GameSystem {
    pub fn update(game_state: &mut GameState, input: &mut Input) {
        Self::handle_item_pickup(game_state, input);
        Self::handle_item_placement(game_state, input);
        Self::handle_inventory(game_state, input);
        Self::resolve_movement(game_state, input);
        Self::update_camera(game_state, input);
    }

    fn handle_item_pickup(game_state: &mut GameState, input: &mut Input) {
        // mut input just for is
        // toggled on. could possibly be changed
        if input.e_pressed.is_toggled_on() {
            let near_item_id = Self::find_near_item_id(game_state);
            if let Some(near_item_id_unpacked) = near_item_id {
                if !Self::in_item_pickup_range(&game_state, near_item_id_unpacked.clone()) {
                    return;
                } else {
                    game_state.remove_entity_from_world(near_item_id_unpacked.clone());
                    game_state.add_item_to_inventory();
                }
            } else {
                return;
            }
        }
    }

    fn find_near_item_id(game_state: &GameState) -> Option<String> {
        let player_position: &Position = game_state.get_player_const().get_position();
        let near_id = game_state
            .get_entities()
            .iter()
            .filter(|entity| entity.id != "player" && entity.id != "plane")
            .min_by_key(|entity| {
                Self::distance_2d(entity.get_position(), player_position)
                    .round()
                    .to_u32()
            })
            .map(|entity| entity.id.clone()); // f32does not have trait ord, for now we just cast.
        return near_id;
    }

    fn distance_2d(position1: &Position, position2: &Position) -> f32 {
        return ((position2.x - position1.x).powi(2) + (position2.y - position1.y).powi(2)).sqrt();
    }

    // probably should be a method on player or something
    fn in_item_pickup_range(game_state: &GameState, near_item_id: String) -> bool {
        return Self::distance_2d(
            game_state.get_player_const().get_position(),
            game_state
                .get_entity(near_item_id)
                .expect("entity should exist")
                .get_position(),
        ) < ITEM_PICKUP_RANGE;
    }

    fn handle_item_placement(game_state: &mut GameState, input: &mut Input) {
        if input.right_mouse_clicked.is_toggled_on() {
            if game_state.inventory_item_count > 0 {
                let placed_position = Position {
                    x: game_state.get_player().position.x - 1.1,
                    y: game_state.get_player().position.y - 1.1,
                    z: game_state.get_player().position.z,
                };
                let entity = game_state.new_entity(placed_position);
                game_state.entities.push(entity);
                game_state.inventory_item_count = game_state.inventory_item_count - 1;
            }
        }
    }

    fn handle_inventory(game_state: &mut GameState, input: &mut Input) {
        if input.i_pressed.is_toggled_on() {
            game_state.inventory_toggled = !game_state.inventory_toggled;
        }
    }

    fn update_camera(game_state: &mut GameState, input: &mut Input) {
        if input.up_pressed.is_pressed {
            game_state.camera_rotation_y_degrees =
                game_state.camera_rotation_y_degrees + CAMERA_MOVEMENT_SPEED;
        }

        if input.down_pressed.is_pressed {
            game_state.camera_rotation_y_degrees =
                game_state.camera_rotation_y_degrees - CAMERA_MOVEMENT_SPEED;
        }

        if input.right_pressed.is_pressed {
            game_state.camera_rotation_x_degrees =
                game_state.camera_rotation_x_degrees - CAMERA_MOVEMENT_SPEED;
        }

        if input.left_pressed.is_pressed {
            game_state.camera_rotation_x_degrees =
                game_state.camera_rotation_x_degrees + CAMERA_MOVEMENT_SPEED;
        }

        // We do this to keep the degrees in range of 0 to 359.99.. which modulo would not do...
        // does this matter though... seems the effect is the same...
        if game_state.camera_rotation_x_degrees < 0.0 {
            game_state.camera_rotation_x_degrees += 360.0;
        }

        if game_state.camera_rotation_x_degrees >= 360.0 {
            game_state.camera_rotation_x_degrees -= 360.0;
        }

        if game_state.camera_rotation_y_degrees < CAMERA_BOTTOM_LIMIT {
            game_state.camera_rotation_y_degrees = CAMERA_BOTTOM_LIMIT;
        }

        if game_state.camera_rotation_y_degrees >= CAMERA_TOP_LIMIT {
            game_state.camera_rotation_y_degrees = CAMERA_TOP_LIMIT;
        }

        let normalised_scroll_amount: f32 = -input.scrolled_amount * 0.1;

        if game_state.camera_distance + normalised_scroll_amount <= MIN_CAMERA_DISTANCE {
            game_state.camera_distance = MIN_CAMERA_DISTANCE;
        } else if game_state.camera_distance + normalised_scroll_amount >= MAX_CAMERA_DISTANCE {
            game_state.camera_distance = MAX_CAMERA_DISTANCE;
        } else {
            game_state.camera_distance += normalised_scroll_amount;
        }

        input.scrolled_amount = 0.0;
    }

    fn resolve_movement(game_state: &mut GameState, input: &Input) {
        let mut movement_speed: f32 = BASE_SPEED;
        if input.left_shift_pressed.is_pressed {
            movement_speed *= 2.5;
        }

        if input.w_pressed.is_pressed {
            let player = game_state.get_player();
            player.previous_position = Position {
                x: player.position.x.clone(),
                y: player.position.y.clone(),
                z: player.position.z.clone(),
            };
            player.position.x -= movement_speed;
            player.position.y -= movement_speed;
            Self::resolve_collisions(game_state)
        }

        if input.s_pressed.is_pressed {
            let player = game_state.get_player();
            player.previous_position = Position {
               x: player.position.x.clone(),
               y: player.position.y.clone(),
               z: player.position.z.clone(),
            };
            
            player.position.x += movement_speed;
            player.position.y += movement_speed;
            Self::resolve_collisions(game_state)
        }

        if input.a_pressed.is_pressed {
            let player = game_state.get_player();
            player.previous_position = Position {
                x: player.position.x.clone(),
                y: player.position.y.clone(),
                z: player.position.z.clone(),
            };
            player.position.x -= movement_speed;
            player.position.y += movement_speed;
            Self::resolve_collisions(game_state)
        }

        if input.d_pressed.is_pressed {
            let player = game_state.get_player();
            player.previous_position = Position {
                x: player.position.x.clone(),
                y: player.position.y.clone(),
                z: player.position.z.clone(),
            };
            player.position.x += movement_speed;
            player.position.y -= movement_speed;
            Self::resolve_collisions(game_state)
        }
    }

    fn resolve_collisions(game_state: &mut GameState) {
        let interactible_entities: Vec<&Entity> = game_state.entities.iter()
            .filter(|entity| entity.id != "player" && entity.id != "plane")
            .collect();
        let player_const = game_state.get_player_const();
        let mut should_update = false;
        for entity in interactible_entities {
            if Self::check_collision(&player_const, &entity) {
                should_update = true;
            }
        }

        // vile... game state both borrowed as mut and not mut if updated directly... not sure good
        // way to write this code.
        if should_update {
            let player = game_state.get_player();
                            player.update_position(
                // don't match itself
                Position {
                    x: player.previous_position.x.clone(),
                    y: player.previous_position.y.clone(),
                    z: player.previous_position.z.clone(),
                });
        }
    }

    fn check_collision(player: &Entity, other_entity: &Entity) -> bool {
        let is_collision_x = player.position.x + player.hitbox
            >= other_entity.position.x - other_entity.hitbox
            && other_entity.position.x + other_entity.hitbox >= player.position.x - player.hitbox;

        let is_collision_y = player.position.y + player.hitbox
            >= other_entity.position.y - other_entity.hitbox
            && other_entity.position.y + other_entity.hitbox >= player.position.y - player.hitbox;

        let is_collision_z = player.position.z + player.hitbox
            >= other_entity.position.z - other_entity.hitbox
            && other_entity.position.z + other_entity.hitbox >= player.position.z - player.hitbox;

        return is_collision_x && is_collision_y && is_collision_z;
    }
}
