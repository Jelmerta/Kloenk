use crate::game_state::GameState; 

struct Panel {
    // widgets: Vec<Widget>,
}

struct InventoryPanel {
    panel: Panel,
}

pub struct UIState {
    pub inventory_open: bool,
    pub inventory_position_x: f32,
    pub inventory_position_y: f32,
    pub inventory_width: f32,
    pub inventory_height: f32,
}

impl UIState {
    pub fn new() -> Self {
        UIState {
            inventory_open: false,
            inventory_position_x: 1.33, // TODO these values dont makeMuch sense to me
            inventory_position_y: -0.9,
            inventory_width: 0.2,
            inventory_height: 0.2,
        }
    }
}

fn show_inventory(game_state: &GameState) {
    // let player_inventory = game_state.get_storage_component("player".to_string());
    // let items = game_state.get_in_inventory_components();

    // if 
}
