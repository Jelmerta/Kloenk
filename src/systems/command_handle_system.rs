use crate::state::frame_state::{ActionEffect, ActionRequest, FrameState};
use crate::state::game_state::GameState;
use crate::state::ui_state::UIState;
use crate::systems::item_placement_system::ItemPlacementSystem;

pub struct CommandHandleSystem {}

impl CommandHandleSystem {
    pub fn handle_action_requests(game_state: &mut GameState, frame_state: &mut FrameState) {
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

    pub fn handle_action_effects(ui_state: &mut UIState, frame_state: &mut FrameState) {
        frame_state
            .action_effects
            .iter()
            .for_each(|command| match command {
                ActionEffect::PickupItemNotStorable => {
                    ui_state.action_text = "That cannot be picked up.".to_string();
                }
                ActionEffect::PickupNoItemInRange => {
                    ui_state.action_text = "No item found around you to pick up.".to_string();
                }
                ActionEffect::PlaceItemNonPlaceable => {
                    ui_state.action_text = "Cannot place outside placeable area.".to_string();
                }
                ActionEffect::PlaceItemCollidingItem => {
                    ui_state.action_text =
                        "Found a colliding object.\nNot allowed to place there.".to_string();
                }
                ActionEffect::PickupNoInventorySpace => {
                    ui_state.action_text =
                        "There is no space left in your\ninventory to pick up this item."
                            .to_string();
                }
                ActionEffect::PlaceItemSucceeded => {
                    ui_state.action_text = "You drop the item.".to_string();
                }
                ActionEffect::ItemSelected { found_objects_text } => {
                    ui_state.selected_text = found_objects_text.to_string();
                }
                ActionEffect::Examine { text } => {
                    ui_state.action_text = text.clone();
                }
            });
    }
}
