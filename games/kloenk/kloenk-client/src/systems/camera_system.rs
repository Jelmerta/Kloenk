use crate::state::components::CameraTarget;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use cgmath::{InnerSpace, Point3, Vector3};
use std::sync::Arc;
use winit::window::Window;

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;
pub const CAMERA_MOVEMENT_SPEED: f32 = 3.0;
pub const CAMERA_BOTTOM_LIMIT: f32 = 280.0;
pub const CAMERA_TOP_LIMIT: f32 = 350.0;
const SCROLL_FACTOR: f32 = 0.3;

pub struct CameraSystem {}

impl CameraSystem {
    pub fn update_camera(window: &Arc<Window>, game_state: &mut GameState, input: &mut Input) {
        Self::setup_camera_target(game_state, input);
        Self::setup_camera(game_state);
        let camera = game_state.get_camera_mut("camera").expect("Camera should exist");
        camera.update_view_projection_matrix(window);
        camera.update_inverse_matrix();
    }

    fn setup_camera(game_state: &mut GameState) {
        let player = "player";
        let player_position = *game_state.get_position(player).expect("Player position should exist");
        let player_camera = *game_state.get_camera_target(player).expect("Player camera should exist");

        let rad_x = f32::to_radians(player_camera.rotation_x_degrees);
        let rad_y = f32::to_radians(player_camera.rotation_y_degrees);

        let camera = game_state.get_camera_mut("camera").expect("Camera should exist");
        camera.eye = Point3 {
            x: player_position.x + player_camera.distance * rad_y.sin() * rad_x.cos(),
            y: player_position.y + player_camera.distance * rad_y.cos(),
            z: player_position.z + player_camera.distance * rad_y.sin() * rad_x.sin(),
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
            .get_camera_target_mut("player")
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

        let normalised_scroll_amount: f32 = -input.scrolled_amount * SCROLL_FACTOR;

        if player_camera.distance + normalised_scroll_amount <= MIN_CAMERA_DISTANCE {
            player_camera.distance = MIN_CAMERA_DISTANCE;
        } else if player_camera.distance + normalised_scroll_amount >= MAX_CAMERA_DISTANCE {
            player_camera.distance = MAX_CAMERA_DISTANCE;
        } else {
            player_camera.distance += normalised_scroll_amount;
        }

        input.scrolled_amount = 0.0;
    }
}
