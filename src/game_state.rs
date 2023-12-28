pub struct game_state {
    player_position_x: f64,
    player_position_y: f64,
}

impl game_state {
    pub fn new() -> Self {
        Self {
            player_position_x: 0.0,
            player_position_y: 0.0,
        }
    }
}

// pub(crate) fn new() -> _ {
//     todo!()
// }