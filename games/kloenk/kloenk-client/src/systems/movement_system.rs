use crate::application::AudioState;
use crate::state::components::{Entity, Hitbox, Rotation};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::systems::collision_manager::CollisionManager;
use cgmath::{InnerSpace, Point3, Vector2};
use std::f32::consts::PI;
use std::ops::Sub;

pub const BASE_SPEED: f32 = 0.01;
pub const CHARACTER_ROTATION_SPEED_DEGREES: f32 = 5.0;

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

        let angle_option = Self::get_desired_angle(input);
        if angle_option.is_none() {
            return;
        }

        let angle = angle_option.unwrap();

        let player_position = game_state.get_position("player").unwrap();
        let desired_position = Point3 {
            x: player_position.x + movement_speed * angle.sin(),
            y: player_position.y,
            z: player_position.z + movement_speed * angle.cos(),
        };

        let player_hitbox = game_state.get_hitbox("player").unwrap();

        let desired_player_hitbox = Hitbox {
            box_corner_min: Point3::new(
                player_hitbox.box_corner_min.x + movement_speed * angle.sin(),
                player_hitbox.box_corner_min.y,
                player_hitbox.box_corner_min.z + movement_speed * angle.cos(),
            ),
            box_corner_max: Point3::new(
                player_hitbox.box_corner_max.x + movement_speed * angle.sin(),
                player_hitbox.box_corner_max.y,
                player_hitbox.box_corner_max.z + movement_speed * angle.cos(),
            ),
        };
        if Self::is_walkable(game_state, &desired_position)
            && !Self::is_colliding(&desired_player_hitbox, game_state, audio_state)
        {
            Self::update_rotation(game_state, desired_position);
            game_state.remove_position("player");
            game_state
                .position_components
                .insert("player".to_owned(), desired_position);
            Self::update_hitbox(game_state, desired_player_hitbox);
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
                    && game_state.get_hitbox(entity.as_str()).is_some()
                    && game_state.get_position(entity.as_str()).is_some()
            })
            .collect();

        for entity in interactable_entities {
            let entity_hitbox = game_state.get_hitbox(entity).unwrap();

            if CollisionManager::check_collision(desired_player_hitbox, entity_hitbox) {
                #[allow(irrefutable_let_patterns)]
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

    fn update_rotation(game_state: &mut GameState, desired_position: Point3<f32>) {
        let old_rotation = game_state
            .get_rotation("player")
            .expect("Old rotation should be present");
        let player_position = game_state
            .get_position("player")
            .expect("player position should exist");

        let direction_3d = desired_position.sub(player_position);
        // Player model is aimed at z-direction?
        let new_rotation = Self::calculate_movement_rotation_2d(
            Vector2::new(0.0, 1.0),
            Vector2::new(direction_3d.x, direction_3d.z),
        );
        let mut rotation_difference = new_rotation - old_rotation.degrees_y;
        if rotation_difference < 180.0 {
            rotation_difference += 360.0;
        }
        if rotation_difference > 180.0 {
            rotation_difference -= 360.0;
        }
        let rotation_difference_clamped = rotation_difference.clamp(
            -CHARACTER_ROTATION_SPEED_DEGREES,
            CHARACTER_ROTATION_SPEED_DEGREES,
        );
        let used_rotation = if rotation_difference_clamped < CHARACTER_ROTATION_SPEED_DEGREES
            && rotation_difference_clamped > -CHARACTER_ROTATION_SPEED_DEGREES
        {
            new_rotation
        } else {
            old_rotation.degrees_y + rotation_difference_clamped
        };

        game_state.rotation_components.remove("player");
        game_state.rotation_components.insert(
            "player".to_owned(),
            Rotation {
                degrees_y: used_rotation,
            },
        );
    }

    // Followed: https://wumbo.net/formulas/angle-between-two-vectors-2d/
    fn calculate_movement_rotation_2d(
        base_direction: Vector2<f32>,
        new_direction: Vector2<f32>,
    ) -> f32 {
        let angle = base_direction.dot(new_direction);
        let determinant = base_direction.x * new_direction.y - base_direction.y * new_direction.x;
        -f32::atan2(determinant, angle).to_degrees()
    }

    fn update_hitbox(game_state: &mut GameState, new_hitbox: Hitbox) {
        game_state.hitbox_components.remove("player");
        game_state
            .hitbox_components
            .insert("player".to_owned(), new_hitbox);
    }

    // Assumes for now Z-positive is 0 degrees
    fn get_desired_angle(input: &Input) -> Option<f32> {
        let mut x: f32 = 0.0;
        let mut z: f32 = 0.0;

        if input.w_pressed.is_pressed {
            x -= 1.0;
            z -= 1.0;
        }

        if input.s_pressed.is_pressed {
            // angle = 4
            // x5.0;
            x += 1.0;
            z += 1.0;
        }

        if input.a_pressed.is_pressed {
            x += 1.0;
            z -= 1.0;
        }

        if input.d_pressed.is_pressed {
            x -= 1.0;
            z += 1.0;
        }

        if x == 0.0 && z == 0.0 {
            return None;
        }

        let angle = z.atan2(x);
        Some((angle + 2.0 * PI) % (2.0 * PI))
    }
}
