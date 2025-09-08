use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::state::update_state::{ActionEffect, ActionRequest, UpdateState};
use crate::systems::item_placement_system::ItemPlacementSystem;

pub struct CommandHandleSystem {}

impl CommandHandleSystem {
    pub fn handle_action_requests(game_state: &mut GameState, frame_state: &mut UpdateState) {
        frame_state
            .action_requests
            .iter()
            .for_each(|command| match command {
                ActionRequest::ItemPlacement { entity } => {
                    ItemPlacementSystem::place_item(
                        game_state,
                        &mut frame_state.action_effects,
                        entity,
                    );
                }
            });
    }

    pub fn handle_action_effects(ui_state: &mut UIState, frame_state: &mut UpdateState) {
        frame_state
            .action_effects
            .iter()
            .for_each(|command| match command {
                ActionEffect::PickupItemNotStorable => {
                    "That cannot be picked up.".clone_into(&mut ui_state.action_text);
                }
                ActionEffect::PickupNoItemInRange => {
                    "No item found around you to pick up.".clone_into(&mut ui_state.action_text);
                }
                ActionEffect::PlaceItemNotInInventory => {
                    "Tried to place an item not in your inventory, how did that happen?"
                        .clone_into(&mut ui_state.action_text);
                }
                ActionEffect::PlaceItemNonPlaceable => {
                    "Cannot place outside placeable area.".clone_into(&mut ui_state.action_text);
                }
                ActionEffect::PlaceItemCollidingItem => {
                    "Found a colliding object.\nNot allowed to place there."
                        .clone_into(&mut ui_state.action_text);
                }
                ActionEffect::PickupNoInventorySpace => {
                    "There is no space left in your\ninventory to pick up this item."
                        .clone_into(&mut ui_state.action_text);
                }
                ActionEffect::PlaceItemSucceeded => {
                    "You drop the item.".clone_into(&mut ui_state.action_text);
                }
                ActionEffect::ItemSelected { found_objects_text } => {
                    found_objects_text.clone_into(&mut ui_state.selected_text);
                }
                ActionEffect::Examine { text } => text.clone_into(&mut ui_state.action_text),
            });
    }
}
