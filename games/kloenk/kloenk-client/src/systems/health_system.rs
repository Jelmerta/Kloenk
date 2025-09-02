use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{RenderCommand, UIElement, UserAction};
use cgmath::Point2;
use std::sync::Arc;
use winit::window::Window;

pub struct HealthSystem {}

impl HealthSystem {
    pub fn display_health(
        window: &Arc<Window>,
        game_state: &GameState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        let health_rect_outside =
            UIElement::new_rect(Point2::new(0.125, 0.90), Point2::new(0.075, 0.05));
        let dialogue_render_command = RenderCommand::Model {
            layer: 300,
            ui_element: health_rect_outside,
            model_id: "black_square".to_owned(),
        };
        frame_state
            .gui
            .render_commands
            .push(dialogue_render_command);
        match frame_state
            .gui
            .button_handle(window, health_rect_outside, input)
        {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }

        let player_health = game_state.health_components.get("player").unwrap();
        let percentage_health = player_health.hitpoints as f32 / player_health.max_hitpoints as f32;
        let health_bar_width = 0.90;
        let percentage_health_bar = percentage_health * health_bar_width;

        let health_rect_inside = health_rect_outside.inner_rect(
            Point2::new(0.05, 0.05),
            Point2::new(0.05 + percentage_health_bar, 0.95),
        );
        let dialogue_render_command = RenderCommand::Model {
            layer: 350,
            ui_element: health_rect_inside,
            model_id: "blood_red_square".to_owned(),
        };
        frame_state
            .gui
            .render_commands
            .push(dialogue_render_command);

        match frame_state
            .gui
            .button_handle(window, health_rect_inside, input)
        {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }
    }
}
