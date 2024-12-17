use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::input::Input;
use crate::state::ui_state::{Rect, UIState, UserAction};
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
        // Not really a button, but we can just re-use?
        let health_rect_outside = Rect {
            top_left: Point2::new(0.05, 0.85),
            bottom_right: Point2::new(0.20, 0.95),
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

        let health_rect_inside = Rect {
            top_left: Point2::new(0.05 + UIState::undo_width_scaling(0.01, window), 0.86),
            bottom_right: Point2::new(
                0.05 + UIState::undo_width_scaling(0.01, window) + percentage_health_bar,
                0.94,
            ),
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
