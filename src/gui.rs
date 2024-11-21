use crate::state::input::Input;
use crate::state::ui_state::{Rect, RenderCommand, UIState, UserAction};
use cgmath::{Point2, Vector3};

pub struct Gui {
    pub render_commands: Vec<RenderCommand>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            render_commands: Vec::new(),
        }
    }

    pub fn image(&mut self, layer: u32, rect: Rect, image_name: String) {
        let image_command = RenderCommand::Mesh {
            layer,
            rect,
            mesh_id: image_name,
        };
        self.render_commands.push(image_command);
    }

    pub fn image_button(
        &mut self,
        layer: u32,
        rect: Rect,
        image_name: String,
        input: &Input,
    ) -> UserAction {
        let image_command = RenderCommand::Mesh {
            layer,
            rect,
            mesh_id: image_name,
        };
        self.render_commands.push(image_command);
        if rect.contains(input.mouse_position_ui) && input.left_mouse_clicked.is_toggled_on() {
            return UserAction::LeftClick;
        }
        if rect.contains(input.mouse_position_ui) && input.right_mouse_clicked.is_toggled_on() {
            return UserAction::RightClick;
        }
        UserAction::None
        // TODO probably on release
    }

    pub fn color_button(
        &mut self,
        layer: u32,
        rect: Rect,
        color: Vector3<f32>,
        input: &Input,
    ) -> UserAction {
        let image_command = RenderCommand::Mesh {
            layer,
            rect,
            mesh_id: "black".to_string(), // TODO hardcoded
        };

        self.render_commands.push(image_command);
        if rect.contains(input.mouse_position_ui) && input.left_mouse_clicked.is_toggled_on() {
            return UserAction::LeftClick;
        }
        if rect.contains(input.mouse_position_ui) && input.right_mouse_clicked.is_toggled_on() {
            return UserAction::RightClick;
        }
        UserAction::None
        // TODO probably on release
    }

    pub fn text(&mut self, layer: u32, rect: Rect, text: String) {
        let text_command = RenderCommand::Text { layer, rect, text };
        self.render_commands.push(text_command);
    }

    pub fn add_text_render_commands(&mut self, ui_state: &UIState) {
        self.text(
            1000,
            Rect::new(Point2::new(0.05, 0.6), Point2::new(0.2, 0.8)),
            ui_state.action_text.clone(),
        );

        self.text(
            1000,
            Rect::new(Point2::new(0.05, 0.1), Point2::new(0.2, 0.2)),
            ui_state.selected_text.clone(),
        );
    }
}
