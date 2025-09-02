use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::UIState;
use crate::systems::camera_system::CameraSystem;
use crate::systems::chat_system::ChatSystem;
use crate::systems::close_menu_system::CloseMenuSystem;
use crate::systems::command_handle_system::CommandHandleSystem;
use crate::systems::dialogue_system::DialogueSystem;
use crate::systems::health_system::HealthSystem;
use crate::systems::inventory_system::InventorySystem;
use crate::systems::item_pickup_system::ItemPickupSystem;
use crate::systems::monitor_change_system::MonitorChangeSystem;
use crate::systems::movement_system::MovementSystem;
use crate::systems::object_detection_system::ObjectDetectionSystem;
use crate::systems::object_selection_system::ObjectSelectionSystem;
use hydrox::AudioSystem;
use std::sync::Arc;
use winit::window::Window;

pub struct GameSystem {}

impl GameSystem {
    pub fn update(
        window: &Arc<Window>,
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &mut Input,
        frame_state: &mut FrameState,
        audio_system: &mut AudioSystem,
    ) {
        frame_state.new_update();

        MonitorChangeSystem::update_monitor(input, window);

        InventorySystem::display_inventory_item_menu(
            window,
            game_state,
            ui_state,
            input,
            frame_state,
        );
        ObjectSelectionSystem::handle_object_selection(
            window,
            game_state,
            ui_state,
            input,
            frame_state,
        );
        CloseMenuSystem::check_to_close_menu(ui_state, input, frame_state);

        InventorySystem::handle_inventory(window, game_state, ui_state, input, frame_state);

        ItemPickupSystem::handle_item_pickup_keyboard(game_state, input, frame_state);
        ItemPickupSystem::handle_item_pickup_mouse(game_state, input, frame_state);

        DialogueSystem::handle_open_dialogue_keyboard(game_state, ui_state, input, frame_state);

        MovementSystem::resolve_movement(game_state, input, audio_system);

        // Visual stuff (pre-render)
        CameraSystem::update_3d_camera(window, game_state, input);

        DialogueSystem::display_dialogue(window, game_state, ui_state, input, frame_state);
        ChatSystem::handle_chat(window, ui_state, input, frame_state);

        ObjectDetectionSystem::setup_detection_for_frame(game_state, input, frame_state);
        CommandHandleSystem::handle_action_requests(game_state, frame_state);
        CommandHandleSystem::handle_action_effects(ui_state, frame_state);
        frame_state.gui.add_text_render_commands(ui_state);

        HealthSystem::display_health(window, game_state, input, frame_state);

        input.update_end_frame();
    }
}
