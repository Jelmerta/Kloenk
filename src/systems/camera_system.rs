use crate::components::CameraTarget;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::UIState;
use cgmath::{InnerSpace, Point3, Vector3};
use winit::dpi::PhysicalSize;

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;
pub const CAMERA_MOVEMENT_SPEED: f32 = 3.0;
pub const CAMERA_BOTTOM_LIMIT: f32 = 280.0;
pub const CAMERA_TOP_LIMIT: f32 = 350.0;

pub struct CameraSystem {}

impl CameraSystem {
    pub fn update_camera(
        window_size: PhysicalSize<u32>,
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
    ) {
        Self::setup_camera_target(game_state, input);
        Self::setup_camera(game_state);
        let camera = game_state.get_camera_mut("camera").unwrap();
        camera.update_view_projection_matrix(window_size.width, window_size.height);
        // .update_view_projection_matrix(ui_state.window_size.width, ui_state.window_size.height);
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
}
