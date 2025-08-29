use crate::state::components::{Entity, Hitbox};
use crate::state::frame_state::ActionEffect;
use crate::state::game_state::GameState;
use crate::systems::collision_manager::CollisionManager;
use cgmath::{ElementWise, Point3};

pub struct ItemPlacementSystem {}
impl ItemPlacementSystem {
    pub fn place_item(
        game_state: &mut GameState,
        action_effects: &mut Vec<ActionEffect>,
        item_unwrap: &str,
    ) {
        let storage_component = game_state.in_storage_components.get_mut(item_unwrap);
        if storage_component.is_none() {
            #[cfg(feature = "debug-logging")]
            log::error!("Tried to place item that's not in inventory"); // Interesting to maybe send this to server to keep track of
            action_effects.push(ActionEffect::PlaceItemNotInInventory);
            return;
        }

        let player_position = game_state.get_position("player").unwrap();
        let placed_position = Point3 {
            x: player_position.x - 1.1,
            y: player_position.y + 0.25,
            z: player_position.z - 1.1,
        };

        if !Self::is_placeable_area(game_state, &placed_position) {
            action_effects.push(ActionEffect::PlaceItemNonPlaceable);
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
                    game_state.get_hitbox(entity.as_str()).unwrap(),
                    &item_hitbox,
                )
            })
            .cloned()
            .collect();
        if !colliding_entities.is_empty() {
            action_effects.push(ActionEffect::PlaceItemCollidingItem);
            return;
        }

        action_effects.push(ActionEffect::PlaceItemSucceeded);
        game_state.create_position(item_unwrap, placed_position);
        game_state.create_hitbox(item_unwrap, item_hitbox);
        game_state.remove_in_storage(item_unwrap);
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
                    game_state.get_position(entity.as_str()).unwrap().x,
                    0.5,
                )
            }) // Assume 0.5 as half tile
            .any(|entity| {
                CollisionManager::check_in_dimension(
                    desired_position.z,
                    0.0,
                    game_state.get_position(entity).unwrap().z,
                    0.5,
                )
            }) // Assume 0.5 as half tile
    }
}
