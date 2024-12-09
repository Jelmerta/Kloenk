use crate::application::AudioState;
use crate::state::components::{Entity, Hitbox};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::systems::collision_manager::CollisionManager;
use cgmath::{ElementWise, Point3};

pub const BASE_SPEED: f32 = 0.01;

pub struct MovementSystem {}

impl MovementSystem {
    pub fn resolve_movement(
        game_state: &mut GameState,
        input: &Input,
        audio_state: &mut AudioState,
    ) {
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
            let player_hitbox = game_state.get_hitbox(&"player".to_string()).unwrap();
            let desired_player_hitbox = Hitbox {
                box_corner_min: player_hitbox.box_corner_min.add_element_wise(Point3::new(
                    -movement_speed,
                    0.0,
                    -movement_speed,
                )),
                box_corner_max: player_hitbox.box_corner_max.add_element_wise(Point3::new(
                    -movement_speed,
                    0.0,
                    -movement_speed,
                )),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(&desired_player_hitbox, game_state, audio_state)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                Self::update_hitbox(game_state, desired_player_hitbox);
            }
        }

        if input.s_pressed.is_pressed {
            let player_position = game_state.get_position(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x + movement_speed,
                y: player_position.y,
                z: player_position.z + movement_speed,
            };
            let player_hitbox = game_state.get_hitbox(&"player".to_string()).unwrap();
            let desired_player_hitbox = Hitbox {
                box_corner_min: player_hitbox.box_corner_min.add_element_wise(Point3::new(
                    movement_speed,
                    0.0,
                    movement_speed,
                )),
                box_corner_max: player_hitbox.box_corner_max.add_element_wise(Point3::new(
                    movement_speed,
                    0.0,
                    movement_speed,
                )),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(&desired_player_hitbox, game_state, audio_state)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                Self::update_hitbox(game_state, desired_player_hitbox);
            }
        }

        if input.a_pressed.is_pressed {
            let player_position = game_state.get_position(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x - movement_speed,
                y: player_position.y,
                z: player_position.z + movement_speed,
            };
            let player_hitbox = game_state.get_hitbox(&"player".to_string()).unwrap();
            let desired_player_hitbox = Hitbox {
                box_corner_min: player_hitbox.box_corner_min.add_element_wise(Point3::new(
                    -movement_speed,
                    0.0,
                    movement_speed,
                )),
                box_corner_max: player_hitbox.box_corner_max.add_element_wise(Point3::new(
                    -movement_speed,
                    0.0,
                    movement_speed,
                )),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(&desired_player_hitbox, game_state, audio_state)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                Self::update_hitbox(game_state, desired_player_hitbox);
            }
        }

        if input.d_pressed.is_pressed {
            let player_position = game_state.get_position(&"player".to_string()).unwrap();
            let desired_position = Point3 {
                x: player_position.x + movement_speed,
                y: player_position.y,
                z: player_position.z - movement_speed,
            };
            let player_hitbox = game_state.get_hitbox(&"player".to_string()).unwrap();
            let desired_player_hitbox = Hitbox {
                box_corner_min: player_hitbox.box_corner_min.add_element_wise(Point3::new(
                    movement_speed,
                    0.0,
                    -movement_speed,
                )),
                box_corner_max: player_hitbox.box_corner_max.add_element_wise(Point3::new(
                    movement_speed,
                    0.0,
                    -movement_speed,
                )),
            };
            if Self::is_walkable(game_state, &desired_position)
                && !Self::is_colliding(&desired_player_hitbox, game_state, audio_state)
            {
                let player_position = game_state.get_position_mut(&"player".to_string()).unwrap();
                *player_position = desired_position;
                Self::update_hitbox(game_state, desired_player_hitbox);
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

    fn is_colliding(
        desired_player_hitbox: &Hitbox,
        game_state: &GameState,
        audio_state: &mut AudioState,
    ) -> bool {
        let interactable_entities: Vec<&Entity> = game_state
            .entities
            .iter()
            .filter(|entity| {
                entity.as_str() != "player"
                    && game_state.get_hitbox(&(*entity).to_string()).is_some()
                    && game_state.get_position(&(*entity).to_string()).is_some()
            })
            .collect();

        for entity in interactable_entities {
            let entity_hitbox = game_state.get_hitbox(&entity.to_string()).unwrap();

            if CollisionManager::check_collision(desired_player_hitbox, entity_hitbox) {
                if let AudioState::Loaded(audio_system) = audio_state {
                    audio_system.play_sound("bonk")
                } // else audio not loaded

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

    fn update_hitbox(game_state: &mut GameState, new_hitbox: Hitbox) {
        game_state.hitbox_components.remove("player");
        game_state
            .hitbox_components
            .insert("player".to_string(), new_hitbox);
    }
}
