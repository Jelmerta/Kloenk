use web_sys::console;

use crate::{camera, game_state};
use crate::game_state::{Entity, GameState, Position};
use crate::input::input;

pub struct GameSystem {
    camera_distance: f32,
}

pub const BASE_SPEED: f32 = 0.01;

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;

impl GameSystem {
    pub fn new() -> Self {
        Self {
            camera_distance: 150.0,
        }
    }

    pub fn update(game_state: &mut GameState, input: &mut input) {
        // let entities = &mut game_state.entities;
        Self::resolve_movement(game_state, input);
        Self::update_camera(game_state, input);
    }

    fn update_camera(game_state: &mut GameState, input: &mut input) {
        let normalised_scroll_amount: f32 = -input.scrolled_amount * 0.1;
        // game_state.camera.previous_position = Position {
        //     x: game_state.camera.position.x.clone(),
        //     y: game_state.camera.position.y.clone(),
        //     z: game_state.camera.position.z.clone(),
        // };
        // 
        // if (game_state.camera.position.x + normalised_scroll_amount <= MIN_CAMERA_DISTANCE) {
        //     game_state.camera.position.x = MIN_CAMERA_DISTANCE;
        //     game_state.camera.position.y = MIN_CAMERA_DISTANCE;
        //     game_state.camera.position.z = MIN_CAMERA_DISTANCE;
        // } else if (game_state.camera.position.x + normalised_scroll_amount >= MAX_CAMERA_DISTANCE) {
        //     game_state.camera.position.x = MAX_CAMERA_DISTANCE;
        //     game_state.camera.position.y = MAX_CAMERA_DISTANCE;
        //     game_state.camera.position.z = MAX_CAMERA_DISTANCE;
        // } else {
        //     game_state.camera.position.x += normalised_scroll_amount;
        //     game_state.camera.position.y += normalised_scroll_amount;
        //     game_state.camera.position.z += normalised_scroll_amount;
        // }


        if (game_state.camera_distance + normalised_scroll_amount <= MIN_CAMERA_DISTANCE) {
            game_state.camera_distance = MIN_CAMERA_DISTANCE;
        } else if (game_state.camera_distance + normalised_scroll_amount >= MAX_CAMERA_DISTANCE) {
            game_state.camera_distance = MAX_CAMERA_DISTANCE;
        } else {
            game_state.camera_distance += normalised_scroll_amount;
        }



        // game_state.camera.position.x = game_state.player.position.x + self.camera_distance;
        input.scrolled_amount = 0.0;
    }

    fn resolve_movement(game_state: &mut GameState, input: &input) {
        let mut movement_speed: f32 = BASE_SPEED;
        if (input.left_shift_pressed) {
            movement_speed *= 2.5;
        }

        if input.up_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x -= movement_speed;
            game_state.player.position.y -= movement_speed;
            resolve_collisions(game_state)
        }

        if input.down_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x += movement_speed;
            game_state.player.position.y += movement_speed;
            resolve_collisions(game_state)
        }

        if input.left_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x -= movement_speed;
            game_state.player.position.y += movement_speed;
            resolve_collisions(game_state)
        }

        if input.right_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x += movement_speed;
            game_state.player.position.y -= movement_speed;
            resolve_collisions(game_state)
        }
    }
}

fn resolve_collisions(game_state: &mut GameState) {
    for entity in &game_state.entities {
        if check_collision(&game_state.player, &entity) { // don't match itself
            game_state.player.position = Position {
                x: game_state.player.previous_position.x.clone(),
                y: game_state.player.previous_position.y.clone(),
                z: game_state.player.previous_position.z.clone(),
            };
        }
    }
}

fn check_collision(player: &Entity, other_entity: &Entity) -> bool {
    let is_collision_x = player.position.x + player.hitbox >=
        other_entity.position.x - other_entity.hitbox &&
        other_entity.position.x + other_entity.hitbox >=
            player.position.x - player.hitbox;

    let is_collision_y = player.position.y + player.hitbox >=
        other_entity.position.y - other_entity.hitbox &&
        other_entity.position.y + other_entity.hitbox >=
            player.position.y - player.hitbox;

    let is_collision_z = player.position.z + player.hitbox >=
        other_entity.position.z - other_entity.hitbox &&
        other_entity.position.z + other_entity.hitbox >=
            player.position.z - player.hitbox;

    // collision only if on both axes
    return is_collision_x && is_collision_y && is_collision_z;
}
