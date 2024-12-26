use cgmath::Point2;

use crate::state::{
    frame_state::FrameState,
    input::Input,
    ui_state::{ChatState, UIElement, UIState, UIWindow},
};

pub struct ChatSystem {}

impl ChatSystem {
    pub fn display_chat(ui_state: &mut UIState, input: &Input, frame_state: &mut FrameState) {
        let mut new_chat_state = None;
        match ui_state.chat_state {
            crate::state::ui_state::ChatState::Closed => {
                new_chat_state = Some(ChatState::Open);
                // Visible vs chatstate? hmm
            }
            crate::state::ui_state::ChatState::Open => {
                if input.enter_pressed.is_pressed {
                    //Self::send_message(); Should close the chat as well?
                }
                // TODO Cancel options like escape to close?
            }
        }

        if new_chat_state.is_some() {
            ui_state.chat_state = new_chat_state.unwrap();
            frame_state.handled_enter_click = true;
        }
    }
}
