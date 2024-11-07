use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::gui::UIState;
use crate::input::Input;
use cgmath::Point2;

pub struct ObjectSelectionSystem();

impl ObjectSelectionSystem {
    // Get selected
    // Close other menus?
    // Open selection menu
    // Visualize menu

    pub fn handle_object_selection(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        // Should we start using like an observer pattern?
        if !input.right_mouse_clicked.is_toggled_on() {
            return;
        }

        if frame_state.handled_right_click {
            return;
        }

        let selected_objects = &frame_state.objects_on_cursor;
        if selected_objects.is_empty() {
            return;
        }

        let cursor_ndc = input.mouse_position_ndc;
        let cursor_ui_space = Point2::new(cursor_ndc.x / 2.0 + 0.5, -cursor_ndc.y / 2.0 + 0.5);

        // let object_menu = UIElement::new_image(
        //     "inventory".to_string(), // todo image
        //     true,
        //     Point2::new(cursor_ui_space.x, cursor_ui_space.y),
        //     Point2::new(cursor_ui_space.x + 0.15, cursor_ui_space.y + 0.5),
        //     Some(
        //         |game_state: &mut GameState,
        //          ui_state: &mut UIState,
        //          input: &Input,
        //          frame_state: &mut FrameState| {
        //             ItemPickupSystem::handle_item_pickup_mouse(
        //                 game_state,
        //                 ui_state,
        //                 input,
        //                 frame_state,
        //             )
        //         },
        //     ),
        // );

        // ui_state.object_menu = Some(object_menu);
    }
}
