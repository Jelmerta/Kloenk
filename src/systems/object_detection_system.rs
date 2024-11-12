use crate::components::{Entity, Hitbox};
use crate::input::Input;
use crate::state::frame_state::{ActionEffect, FrameState};
use crate::state::game_state::GameState;
use crate::systems::utility::distance_3d;
use cgmath::num_traits::Float;
use cgmath::{InnerSpace, Point3, Vector3, Vector4};
use itertools::Itertools;

#[derive(Debug)]
struct Ray {
    origin: Point3<f32>,
    #[allow(dead_code)] // Other algorithm choice might use direction
    direction: Vector3<f32>,
    direction_inverted: Vector3<f32>,
}

pub struct ObjectDetectionSystem {}

impl ObjectDetectionSystem {
    pub fn setup_detection_for_frame(
        game_state: &mut GameState,
        input: &mut Input,
        frame_state: &mut FrameState,
    ) {
        Self::find_world_object_on_cursor(game_state, input, frame_state);
        Self::set_nearest_object(game_state, frame_state);
    }

    fn find_world_object_on_cursor(
        game_state: &mut GameState,
        input: &mut Input,
        frame_state: &mut FrameState,
    ) {
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

        for (entity, hitbox) in game_state.hitbox_components.iter() {
            if Self::intersection(&ray, hitbox) {
                frame_state.add_object(entity.clone());
            }
        }

        let found_objects_text = frame_state.get_objects_on_cursor().join(", ");
        frame_state.action_effects.push(ActionEffect::ItemSelected {
            found_objects_text: found_objects_text.to_string(),
        })
    }

    fn set_nearest_object(game_state: &GameState, frame_state: &mut FrameState) {
        let player_position = game_state.get_position(&"player".to_string()).unwrap();
        let nearest_object: Option<Entity> = frame_state
            .get_objects_on_cursor()
            .iter()
            .filter(|entity| !(*entity).eq(&"player".to_string()))
            .map(|entity| {
                let object_position = game_state.get_position(entity).unwrap();
                let distance = distance_3d(object_position, player_position);
                (entity, distance)
            })
            .sorted_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap())
            .map(|(entity, _)| entity.clone())
            .next();

        frame_state.set_nearest_object_on_cursor(nearest_object);
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
