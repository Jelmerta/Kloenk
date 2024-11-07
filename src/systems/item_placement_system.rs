use crate::collision_manager::CollisionManager;
use crate::components::{Entity, Hitbox};
use crate::game_state::GameState;
use crate::gui::{Payload, UIElement, UIState};
use crate::input::Input;
use cgmath::{ElementWise, Point2, Point3};

pub struct ItemPlacementSystem {}
impl ItemPlacementSystem {
    // pub fn handle_item_placement(
    //     game_state: &mut GameState,
    //     action_text: &mut UIElement,
    //     inventory: &mut UIElement,
    //     click_point: Point2<f32>,
    // ) {
    //     let mut found_item = None;
    //     for (entity, element) in &inventory.child_elements {
    //         if element.contains(click_point) {
    //             found_item = Some(entity.clone());
    //             break;
    //         }
    //     }
    //
    //     if let Some(item) = found_item {
    //         Self::place_item(game_state, action_text, inventory, &item);
    //     }
    // }

    pub fn handle_item_placement(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
    ) {
        let cursor_ndc = input.mouse_position_ndc;
        let cursor_ui_space = Point2::new(cursor_ndc.x / 2.0 + 0.5, -cursor_ndc.y / 2.0 + 0.5);

        if !ui_state.inventory.contains(cursor_ui_space) {
            return;
        }
        let cursor_inventory_space = ui_state.inventory.to_ui_element_space(cursor_ui_space);

        let mut found_item = None;
        for (entity, element) in &ui_state.inventory.child_elements {
            if element.contains(cursor_inventory_space) {
                found_item = Some(entity.clone());
                break;
            }
        }

        if let Some(item) = found_item {
            Self::place_item(
                game_state,
                &mut ui_state.action_text,
                &mut ui_state.inventory,
                &item,
            );
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
}
