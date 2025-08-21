use crate::state::frame_state::{ActionEffect, FrameState};
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::systems::position_manager::PositionManager;
use crate::systems::storage_manager::StorageManager;

pub const ITEM_PICKUP_RANGE: f32 = 1.5;

pub struct ItemPickupSystem {}

impl ItemPickupSystem {
    pub fn handle_item_pickup_keyboard(
        game_state: &mut GameState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let player = "player";

        log::error!("handle item_pickup_keyboard");
        if input.e_pressed.is_toggled_on() && !frame_state.handled_e_click {
            log::error!("toggled!"); // TODO what if its like: yes we did see pickup this frame but its only visible next frame? lets check that
            let near_pickup = PositionManager::find_nearest_pickup(
                &game_state.position_components,
                &game_state.storable_components,
                &game_state.entities,
                player,
            );

            if near_pickup.is_none() {
                frame_state
                    .action_effects
                    .push(ActionEffect::PickupNoItemInRange); // Might not want to show this, just ignore cause there may be other actions to handle
                return;
            }
            if Self::item_pickup(game_state, frame_state, &near_pickup.unwrap()) {
                frame_state.handled_e_click = true;
            }
        }
    }

    pub fn handle_item_pickup_mouse(
        game_state: &mut GameState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if !input.left_mouse_clicked.is_toggled_on() {
            return;
        }

        if frame_state.handled_left_click {
            return;
        }

        if let Some(nearest_object) = frame_state.get_nearest_object_on_cursor() {
            let nearest_object_clone = nearest_object.to_owned();
            Self::item_pickup(game_state, frame_state, &nearest_object_clone);
            frame_state.handled_left_click = true;
        }
    }

    // TODO want more errors for mouse less for keyboard
    pub fn item_pickup(
        game_state: &mut GameState,
        frame_state: &mut FrameState,
        near_pickup: &str,
    ) -> bool {
        let pickup = game_state.storable_components.get(near_pickup);
        if pickup.is_none() {
            frame_state
                .action_effects
                .push(ActionEffect::PickupItemNotStorable);
            return false;
        }

        let item_position = game_state.get_position(near_pickup);
        if item_position.is_none() {
            frame_state
                .action_effects
                .push(ActionEffect::PickupNoItemInRange);
            return false;
        }

        let player = "player";
        if !PositionManager::in_range(
            game_state.get_position(player).unwrap(),
            item_position.unwrap(),
            ITEM_PICKUP_RANGE,
        ) {
            frame_state
                .action_effects
                .push(ActionEffect::PickupNoItemInRange);
            return false;
        }

        let inventory = game_state.get_storage(player).unwrap();
        let inventory_items = StorageManager::get_in_storage(game_state, player);
        if !StorageManager::has_space(game_state, inventory, &inventory_items, near_pickup) {
            frame_state
                .action_effects
                .push(ActionEffect::PickupNoInventorySpace);
            return false;
        }
        let empty_spot =
            StorageManager::find_empty_spot(game_state, inventory, &inventory_items, near_pickup)
                .unwrap();

        game_state.remove_position(near_pickup);
        game_state.remove_hitbox(near_pickup);
        game_state.create_in_storage(player, near_pickup, empty_spot);
        true
    }
}
