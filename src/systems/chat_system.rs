use crate::state::{
    frame_state::FrameState,
    input::Input,
    ui_state::{InputState, UIState},
};

pub struct ChatSystem {}

impl ChatSystem {
    pub fn handle_chat(ui_state: &mut UIState, input: &Input, frame_state: &mut FrameState) {
        let mut new_input_state = None;
        match ui_state.input_state {
            InputState::Normal => {
                new_input_state = Some(InputState::Chat);
                // Visible vs chatstate? hmm
            }
            InputState::Chat => {
                if input.enter_pressed.is_pressed {
                    //Self::send_message(); Should close the chat as well?
                }
                // TODO Cancel options like escape to close?
            }
        }

        if new_input_state.is_some() {
            ui_state.input_state = new_input_state.unwrap();
            frame_state.handled_enter_click = true;
        }
    }

    fn display_chat() {}
}
