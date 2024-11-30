use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::UIState;
use crate::systems::audio_system::AudioSystem;
use crate::systems::camera_system::CameraSystem;
use crate::systems::command_handle_system::CommandHandleSystem;
use crate::systems::inventory_system::InventorySystem;
use crate::systems::item_pickup_system::ItemPickupSystem;
use crate::systems::movement_system::MovementSystem;
use crate::systems::object_detection_system::ObjectDetectionSystem;
use crate::systems::object_selection_system::ObjectSelectionSystem;
use winit::dpi::PhysicalSize;

pub struct GameSystem {}

impl GameSystem {
    pub fn update(
        physical_size: PhysicalSize<u32>,
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        frame_state: &mut FrameState,
        audio_system: &mut AudioSystem,
    ) {
        *frame_state = FrameState::new();
        ObjectDetectionSystem::setup_detection_for_frame(game_state, input, frame_state);

        InventorySystem::display_inventory_item_menu(game_state, ui_state, input, frame_state);
        InventorySystem::handle_inventory(game_state, ui_state, input, frame_state);
        ObjectSelectionSystem::handle_object_selection(game_state, ui_state, input, frame_state);

        ItemPickupSystem::handle_item_pickup_keyboard(game_state, input, frame_state);
        ItemPickupSystem::handle_item_pickup_mouse(game_state, input, frame_state);

        MovementSystem::resolve_movement(game_state, input, audio_system);

        CommandHandleSystem::handle_action_requests(game_state, frame_state);
        CommandHandleSystem::handle_action_effects(ui_state, frame_state);
        frame_state.gui.add_text_render_commands(ui_state);

        // Visual stuff (pre-render)
        CameraSystem::update_camera(physical_size, game_state, input);

        input.update_end_frame();
    }
}
