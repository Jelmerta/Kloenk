use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::gui::UIState;
use crate::input::Input;
use crate::systems::audio_system::AudioSystem;
use crate::systems::camera_system::CameraSystem;
use crate::systems::inventory_system::InventorySystem;
use crate::systems::item_pickup_system::ItemPickupSystem;
use crate::systems::movement_system::MovementSystem;
use crate::systems::object_detection_system::ObjectDetectionSystem;
use crate::systems::object_selection_system::ObjectSelectionSystem;

pub struct GameSystem {}

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

        InventorySystem::handle_inventory_click(game_state, ui_state, input, frame_state);
        ItemPickupSystem::handle_item_pickup_keyboard(game_state, ui_state, input, frame_state);
        ItemPickupSystem::handle_item_pickup_mouse(game_state, ui_state, input, frame_state);

        MovementSystem::resolve_movement(game_state, input, audio_system);

        // Visual stuff (pre-render)
        ObjectSelectionSystem::handle_object_selection(game_state, ui_state, input, frame_state);
        InventorySystem::handle_inventory(ui_state, input);
        CameraSystem::update_camera(game_state, ui_state, input);

        input.update_end_frame();
    }
}
