use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{UIElement, UserAction};
use cgmath::Point2;

pub struct HealthSystem {}

impl HealthSystem {
    pub fn display_health(game_state: &GameState, input: &Input, frame_state: &mut FrameState) {
        // Not really a button, but we can just re-use?
        let health_rect_outside = UIElement {
            top_left: Point2::new(0.05, 0.85),
            bottom_right: Point2::new(0.20, 0.95),
            parent_middle_x: None,
        };
        match frame_state
            .gui
            .color_button(300, health_rect_outside, input, "black".to_string())
        {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }

        let player_health = game_state.health_components.get("player").unwrap();
        let percentage_health = player_health.hitpoints as f32 / player_health.max_hitpoints as f32;
        let health_bar_width = 0.13;
        let percentage_health_bar = percentage_health * health_bar_width;

        let health_rect_inside = UIElement {
            top_left: Point2::new(0.05 + 0.01, 0.86),
            bottom_right: Point2::new(0.05 + 0.01 + percentage_health_bar, 0.94),
            parent_middle_x: None,
        };
        match frame_state
            .gui
            .color_button(350, health_rect_inside, input, "blood_red".to_string())
        {
            UserAction::None => {}
            UserAction::Hover => {}
            UserAction::LeftClick => {}
            UserAction::RightClick => {}
        }
    }
}
