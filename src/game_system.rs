use web_sys::console;

use crate::{camera, game_state};
use crate::game_state::{Entity, GameState, Position};
use crate::input::input;

pub struct GameSystem {}

impl GameSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(game_state: &mut GameState, input: &mut input) {
        // let entities = &mut game_state.entities;
        Self::update_camera(game_state, input);
        Self::resolve_movement(game_state, input);
    }

    fn update_camera(game_state: &mut GameState, input: &mut input) {
        log::warn!("Updated camera");
        game_state.camera.previous_position = Position {
            x: game_state.camera.position.x.clone(),
            y: game_state.camera.position.y.clone(),
            z: game_state.camera.position.z.clone(),
        };

        game_state.camera.position.x += input.scrolled_amount;
        game_state.camera.position.y += input.scrolled_amount;
        game_state.camera.position.z += input.scrolled_amount;

        input.scrolled_amount = 0.0;
    }

    fn resolve_movement(game_state: &mut GameState, input: &input) {
        if input.up_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x -= 0.01;
            game_state.player.position.y -= 0.01;
            resolve_collisions(game_state)
        }

        if input.down_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x += 0.01;
            game_state.player.position.y += 0.01;
            resolve_collisions(game_state)
        }

        if input.left_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x -= 0.01;
            game_state.player.position.y += 0.01;
            resolve_collisions(game_state)
        }

        if input.right_pressed {
            game_state.player.previous_position = Position {
                x: game_state.player.position.x.clone(),
                y: game_state.player.position.y.clone(),
                z: game_state.player.position.z.clone(),
            };
            game_state.player.position.x += 0.01;
            game_state.player.position.y -= 0.01;
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
