use crate::state::input::Input;
use crate::state::ui_state::{RenderCommand, UIElement, UIState, UserAction};
use cgmath::Point2;
use std::sync::Arc;
use winit::window::Window;

pub struct Gui {
    pub render_commands: Vec<RenderCommand>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            render_commands: Vec::new(),
        }
    }

    pub fn create_color_command(
        &mut self,
        layer: u32,
        ui_element: &UIElement,
        color: &str,
    ) -> RenderCommand {
        RenderCommand::Model {
            layer,
            ui_element: *ui_element,
            model_id: color.to_owned() + "_square",
        }
    }

    // TODO Reuse for health and stuff?
    pub fn add_color_command(&mut self, layer: u32, ui_element: &UIElement, color: &str) {
        let command = self.create_color_command(layer, ui_element, color);
        self.render_commands.push(command);
    }

    // pub fn image(&mut self, layer: u32, rect: UIElement, image_name: &str) {
    //     let image_command = RenderCommand::Model {
    //         layer,
    //         ui_element: rect,
    //         model_id: image_name.to_owned(),
    //     };
    //     self.render_commands.push(image_command);
    // }

    pub fn button_handle(
        &mut self,
        window: &Arc<Window>,
        mut ui_element: UIElement,
        input: &Input,
    ) -> UserAction {
        // let element_contains = ui_element.contains(input.mouse_position_ui, window);
        let element_contains = ui_element.contains(input.mouse_position_ui, window);
        if element_contains && input.left_mouse_clicked.is_toggled_on() {
            return UserAction::LeftClick;
        }
        if element_contains && input.right_mouse_clicked.is_toggled_on() {
            return UserAction::RightClick;
        }
        if element_contains {
            return UserAction::Hover;
        }
        UserAction::None
    }

    pub fn text_render(&mut self, layer: u32, rect: UIElement, text: &str, color: [f32; 3]) {
        let text_command = self.build_text_render_command(layer, rect, text, color);
        self.render_commands.push(text_command);
    }

    pub fn build_text_render_command(
        &mut self,
        layer: u32,
        rect: UIElement,
        text: &str,
        color: [f32; 3],
    ) -> RenderCommand {
        RenderCommand::Text {
            layer,
            rect,
            text: text.to_owned(),
            color,
        }
    }

    pub fn add_text_render_commands(&mut self, ui_state: &UIState) {
        self.text_render(
            1000,
            UIElement::new_rect(Point2::new(0.125, 0.7), Point2::new(0.075, 0.1)),
            &ui_state.action_text,
            [0.8, 0.8, 0.0],
        );

        self.text_render(
            1000,
            UIElement::new_rect(Point2::new(0.125, 0.15), Point2::new(0.075, 0.05)),
            &ui_state.selected_text,
            [0.8, 0.8, 0.0],
        );
    }
}
