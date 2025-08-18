use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{UIElement, UserAction};
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
        match frame_state
            .gui
            .color_button(window, 300, health_rect_outside, input, "black")
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

        match frame_state
            .gui
            .color_button(window, 350, health_rect_inside, input, "blood_red")
        {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }
    }
}
