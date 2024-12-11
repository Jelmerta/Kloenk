use crate::state::input::Input;
use crate::state::ui_state::{Rect, RenderCommand, UIState, UserAction};
use cgmath::Point2;

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
        if rect.contains(input.mouse_position_ui) {
            return UserAction::Hover;
        }
        UserAction::None
    }

    pub fn color_button(
        &mut self,
        layer: u32,
        rect: Rect,
        input: &Input,
        // color: [f32; 3], // Probably want to dynamically generate meshes to draw at some time
        color: String,
    ) -> UserAction {
        let mouse_is_contained = rect.contains(input.mouse_position_ui);

        if mouse_is_contained && input.left_mouse_clicked.is_toggled_on() {
            let image_command = RenderCommand::Mesh {
                layer,
                rect,
                mesh_id: color.to_string(), // TODO hardcoded
            };
            self.render_commands.push(image_command);
            return UserAction::LeftClick;
        }
        if mouse_is_contained && input.right_mouse_clicked.is_toggled_on() {
            let image_command = RenderCommand::Mesh {
                layer,
                rect,
                mesh_id: color.to_string(), // TODO hardcoded
            };
            self.render_commands.push(image_command);
            return UserAction::RightClick;
        }
        if mouse_is_contained {
            let image_command = RenderCommand::Mesh {
                layer,
                rect,
                mesh_id: color.to_string(), // TODO hardcoded
            };
            self.render_commands.push(image_command);
            return UserAction::Hover;
        }

        let image_command = RenderCommand::Mesh {
            layer,
            rect,
            mesh_id: color.to_string(), // TODO hardcoded
        };
        self.render_commands.push(image_command);
        UserAction::None
    }

    pub fn text(&mut self, layer: u32, rect: Rect, text: String, color: [f32; 3]) {
        let text_command = RenderCommand::Text {
            layer,
            rect,
            text,
            color,
        };
        self.render_commands.push(text_command);
    }

    pub fn add_text_render_commands(&mut self, ui_state: &UIState) {
        self.text(
            1000,
            Rect::new(Point2::new(0.05, 0.6), Point2::new(0.2, 0.8)),
            ui_state.action_text.clone(),
            [0.8, 0.8, 0.0],
        );

        self.text(
            1000,
            Rect::new(Point2::new(0.05, 0.1), Point2::new(0.2, 0.2)),
            ui_state.selected_text.clone(),
            [0.8, 0.8, 0.0],
        );
    }
}
