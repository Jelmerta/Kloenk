use web_sys::console;
use crate::game_state;
use crate::game_state::GameState;
use crate::input;

pub struct GameSystem {}

impl GameSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(game_state: &mut GameState, input: &input::input) {
        // let mut player = game_state.get_entities().get_mut(0).unwrap(); // Hacky
        // let mut player = game_state.get_entities().get(0).unwrap().borrow_mut(); // Hacky
        let mut player = game_state.entities.get_mut(0).unwrap(); // Hacky

        if input.up_pressed {
            player.position.x -= 0.01;
            player.position.y -= 0.01;
        }

        if input.down_pressed {
            player.position.x += 0.01;
            player.position.y += 0.01;
        }

        if input.left_pressed {
            player.position.x -= 0.01;
            player.position.y += 0.01;
        }

        if input.right_pressed {
            player.position.x += 0.01;
            player.position.y -= 0.01;
        }
    }
}