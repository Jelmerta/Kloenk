use crate::state::ui_state::UserAction;
use crate::state::{
    frame_state::FrameState,
    input::Input,
    ui_state::{InputState, UIState},
};
use std::sync::Arc;
use winit::window::Window;

pub struct ChatSystem {}

impl ChatSystem {
    pub fn handle_chat(
        window: &Arc<Window>,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let mut new_input_state = None;
        match ui_state.input_state {
            InputState::Normal => {
                if input.enter_pressed.is_toggled_on() {
                    new_input_state = Some(InputState::Chat);
                    // Visible vs chatstate? hmm
                }
            }
            InputState::Chat => {
                if input.enter_pressed.is_toggled_on() {
                    //Self::send_message(); Should close the chat as well?
                    new_input_state = Some(InputState::Normal);
                }
                // TODO Cancel options like escape to close?
            }
        }

        if new_input_state.is_some() {
            ui_state.input_state = new_input_state.unwrap();
            frame_state.handled_enter_click = true;
            let chat_window = ui_state.windows.get_mut("chat").unwrap();
            chat_window.is_visible = matches!(ui_state.input_state, InputState::Chat);
        }

        Self::display_chat(window, ui_state, input, frame_state);
    }

    fn display_chat(
        window: &Arc<Window>,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let chat_window = ui_state.windows.get("chat").unwrap();
        if !chat_window.is_visible {
            return;
        }
        match frame_state.gui.color_button(
            window,
            500,
            chat_window.rect,
            input,
            "black".to_string(),
        ) {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }

        // Interact with
        if let InputState::Normal = ui_state.input_state {
            return;
        }
    }
}
