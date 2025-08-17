use crate::state::components::{Entity, Storable};
use crate::state::game_state::GameState;
use cgmath::num_traits::ToPrimitive;
use cgmath::Point3;
use std::collections::HashMap;

pub struct PositionManager {}

impl PositionManager {
    pub fn find_nearest_pickup(
        positions: &HashMap<Entity, Point3<f32>>,
        storables: &HashMap<Entity, Storable>,
        entities: &[Entity],
        entity: &str,
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

    pub fn distance_2d(position1: &Point3<f32>, position2: &Point3<f32>) -> f32 {
        ((position2.x - position1.x).powi(2) + (position2.z - position1.z).powi(2)).sqrt()
    }

    pub fn find_nearest_dialog(game_state: &GameState) -> Option<&str> {
        game_state
            .entities
            .iter()
            .filter(|e| {
                game_state.position_components.contains_key(e.as_str())
                    && game_state.dialogue_components.contains_key(e.as_str())
            })
            .min_by_key(|e| {
                Self::distance_2d(
                    game_state.position_components.get("player").unwrap(),
                    game_state.position_components.get(e.as_str()).unwrap(),
                )
                    .round()
                    .to_u32()
            })
            .map(|e| e.as_str())
    }

    pub fn distance_3d(point1: &Point3<f32>, point2: &Point3<f32>) -> f32 {
        ((point2.x - point1.x).powi(2)
            + (point2.y - point1.y).powi(2)
            + (point2.z - point1.z).powi(2))
            .sqrt()
    }

    pub fn in_range(position1: &Point3<f32>, position2: &Point3<f32>, distance: f32) -> bool {
        PositionManager::distance_2d(position1, position2) < distance
    }
}
