use crate::collision_manager::CollisionManager;
use crate::components::{CameraTarget, Entity, Hitbox};
use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::gui::{Payload, UIElement, UIState};
use crate::input::Input;
use crate::systems::audio_system::AudioSystem;
use crate::systems::item_pickup_system::ItemPickupSystem;
use crate::systems::movement_system::MovementSystem;
use crate::systems::object_detection_system::ObjectDetectionSystem;
use cgmath::{ElementWise, InnerSpace, Point2, Point3, Vector3};

pub struct GameSystem {}

pub const MIN_CAMERA_DISTANCE: f32 = 100.0;
pub const MAX_CAMERA_DISTANCE: f32 = 500.0;
pub const CAMERA_MOVEMENT_SPEED: f32 = 3.0;
pub const CAMERA_BOTTOM_LIMIT: f32 = 280.0;
pub const CAMERA_TOP_LIMIT: f32 = 350.0;
impl GameSystem {
    pub fn update(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        frame_state: &mut FrameState,
        audio_system: &mut AudioSystem,
    ) {
        *frame_state = FrameState::new();
        ObjectDetectionSystem::setup_detection_for_frame(game_state, ui_state, input, frame_state);

        Self::handle_inventory_click(game_state, ui_state, input, frame_state);
        ItemPickupSystem::handle_item_pickup_keyboard(game_state, ui_state, input, frame_state);
        ItemPickupSystem::handle_item_pickup_mouse(game_state, ui_state, input, frame_state);

        MovementSystem::resolve_movement(game_state, input, audio_system);

        // Visual stuff (pre-render)
        Self::handle_inventory(ui_state, input);
        Self::update_camera(game_state, ui_state, input);

        input.update_end_frame();
    }

    pub fn handle_item_placement(
        game_state: &mut GameState,
        action_text: &mut UIElement,
        inventory: &mut UIElement,
        click_point: Point2<f32>,
    ) {
        let mut found_item = None;
        for (entity, element) in &inventory.child_elements {
            if element.contains(click_point) {
                found_item = Some(entity.clone());
                break;
            }
        }

        if let Some(item) = found_item {
            Self::place_item(game_state, action_text, inventory, &item);
        }
    }

    fn place_item(
        game_state: &mut GameState,
        action_text: &mut UIElement,
        inventory: &mut UIElement,
        item_unwrap: &String,
    ) {
        let player_position = game_state.get_position(&"player".to_string()).unwrap();
        let placed_position = Point3 {
            x: player_position.x - 1.1,
            y: player_position.y - 0.25,
            z: player_position.z - 1.1,
        };

        if !Self::is_placeable_area(game_state, &placed_position) {
            action_text.payload = Payload::Text("Cannot place outside placeable area.".to_string());
            return;
        }

        // Generate a dynamic hitbox for the item to be placed
        let item_hitbox_min = placed_position.sub_element_wise(Point3::new(0.26, 0.26, 0.26));
        let item_hitbox_max = placed_position.add_element_wise(Point3::new(0.26, 0.26, 0.26));
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
                CollisionManager::check_collision(
                    game_state.get_hitbox(&(*entity).to_string()).unwrap(),
                    &item_hitbox,
                )
            })
            .cloned()
            .collect();
        if !colliding_entities.is_empty() {
            action_text.payload =
                Payload::Text("Found a colliding object.\nNot allowed to place there.".to_string());
            return;
        }

        action_text.payload = Payload::Text("You drop the item.".to_string());
        inventory.child_elements.remove(&item_unwrap.to_string());
        game_state.create_position(item_unwrap.to_string(), placed_position);
        game_state.create_hitbox(item_unwrap.to_string(), item_hitbox);
        game_state.remove_in_storage(&item_unwrap.to_string());
    }

    fn is_placeable_area(game_state: &GameState, desired_position: &Point3<f32>) -> bool {
        game_state
            .entities
            .iter()
            .filter(|entity| game_state.surface_components.contains(entity.as_str()))
            .filter(|entity| {
                CollisionManager::check_in_dimension(
                    desired_position.x,
                    0.0,
                    game_state.get_position(&(*entity).to_string()).unwrap().x,
                    0.5,
                )
            }) // Assume 0.5 as half tile
            .any(|entity| {
                CollisionManager::check_in_dimension(
                    desired_position.z,
                    0.0,
                    game_state.get_position(&entity.to_string()).unwrap().z,
                    0.5,
                )
            }) // Assume 0.5 as half tile
    }

    fn handle_inventory(ui_state: &mut UIState, input: &mut Input) {
        if input.i_pressed.is_toggled_on() {
            ui_state.inventory.toggle_visibility();
        }
    }

    fn handle_inventory_click(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        frame_state: &mut FrameState,
    ) {
        // Assume toggle is handled. Probably toggles should be handled before performing any
        // systems on them

        if !ui_state.inventory.is_visible {
            return;
        }

        if frame_state.handled_left_click {
            return;
        }

        if !input.left_mouse_clicked.is_toggled_on() {
            return;
        }

        let cursor_ndc = input.mouse_position_ndc;
        let cursor_ui_space = Point2::new(cursor_ndc.x / 2.0 + 0.5, -cursor_ndc.y / 2.0 + 0.5);

        if !ui_state.inventory.contains(cursor_ui_space) {
            return;
        }
        let cursor_inventory_space = ui_state.inventory.to_ui_element_space(cursor_ui_space);

        ui_state.inventory.trigger_click(
            cursor_inventory_space,
            game_state,
            &mut ui_state.action_text,
        );
        frame_state.handled_left_click = true;
    }

    fn update_camera(game_state: &mut GameState, ui_state: &mut UIState, input: &mut Input) {
        Self::setup_camera_target(game_state, input);
        Self::setup_camera(game_state);
        let camera = game_state.get_camera_mut("camera").unwrap();
        camera
            .update_view_projection_matrix(ui_state.window_size.width, ui_state.window_size.height);
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
}
