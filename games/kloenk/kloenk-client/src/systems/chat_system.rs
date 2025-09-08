use crate::state::ui_state::{RenderCommand, UserAction};
use crate::state::{
    input::Input,
    ui_state::{InputState, UIState},
    update_state::UpdateState,
};
use std::sync::Arc;
use winit::window::Window;

pub struct ChatSystem {}

impl ChatSystem {
    pub fn handle_chat(
        window: &Arc<Window>,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut UpdateState,
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

        if let Some(new_state) = new_input_state {
            ui_state.input_state = new_state;
            frame_state.handled_enter_click = true;
            let chat_window = ui_state
                .windows
                .get_mut("chat")
                .expect("Chat window should exist");
            chat_window.is_visible = matches!(ui_state.input_state, InputState::Chat);
        }

        Self::display_chat(window, ui_state, input, frame_state);
    }

    fn display_chat(
        window: &Arc<Window>,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut UpdateState,
    ) {
        let chat_window = ui_state.windows.get("chat").unwrap();
        if !chat_window.is_visible {
            return;
        }
        match frame_state
            .gui
            .button_handle(window, chat_window.rect, input)
        {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }

        let chat_render_command = RenderCommand::Model {
            layer: 500,
            ui_element: chat_window.rect,
            model_id: "black_square".to_owned(),
        };
        frame_state.gui.render_commands.push(chat_render_command);

        // TODO Interact with
        if let InputState::Normal = ui_state.input_state {
            #[allow(clippy::needless_return)]
            return;
        }
    }
}
