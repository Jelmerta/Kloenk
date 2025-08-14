use crate::state::frame_state::FrameState;
use crate::state::input::Input;
use crate::state::ui_state::UIElement;
use cgmath::Point2;
use winit::window::Window;

const CURSOR_HOTSPOT: [u8; 2] = [3, 3];

pub struct CursorSystem {}

// TODO fun to play around with, but cursor moves too slow rn... i guess the data for mouse is already outdated? should the event not have updated it? is rendering too slow? new monkey island seems to able to do this just fine...
// Maybe during rendering there still needs to be a reference to the real input data, and not a clone?
// or maybe we need a direct render option for cursor instead of a render command
impl CursorSystem {
    pub fn handle_cursor(window: &Window, input: &Input, frame_state: &mut FrameState) {
        let monitor_size = window.current_monitor().unwrap().size();
        let monitor_width = monitor_size.width as f32;
        let monitor_height = monitor_size.height as f32;
        let middle_x = input.mouse_position_ui.clone().x + (-3.0 + (61.0 / 2.0)) / monitor_width;
        let middle_y = input.mouse_position_ui.clone().y + (-3.0 + (60.0 / 2.0)) / monitor_height;

        // Maintain aspect ratio? (60/61) * (9/16) * width
        // let cursor_element = UIElement::new_rect(input.mouse_position_ui.clone(), Point2::new(61.0 / monitor_width, 60.0 / monitor_height));
        let cursor_element = UIElement::new_rect(Point2::new(middle_x, middle_y), Point2::new(61.0 / monitor_width, 60.0 / monitor_height)); // TODO dpi maybe?
        frame_state.gui.image(2000, cursor_element, "cursor".to_string())
    }

    //61x60 pixels... hmm maybe just make it like 60x60
}