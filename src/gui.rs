pub struct UIState {
    pub inventory_open: bool,
    pub inventory_position_x: f32,
    pub inventory_position_y: f32,
    pub inventory_width: f32,
    pub inventory_height: f32,

    pub text: String,
    pub text_position_x: f32,
    pub text_position_y: f32,
    pub text_width: f32,
    pub text_height: f32,
}

impl UIState {
    pub fn new() -> Self {
        UIState {
            inventory_open: false,
            
            inventory_position_x: 0.6,
            inventory_position_y: 0.6,
            inventory_width: 0.35,
            inventory_height: 0.35,

            // TODO We can probably store items here on a signal when inv changes. That
            // way we do not need to calculate inventory every frame when inventory is
            // shown

            text: "".to_string(),
            text_position_x: 0.05,
            text_position_y: 0.6,
            text_width: 0.6,
            text_height: 0.4,
        }
    }

    // Maps 0 (left of screen) to -800/600 (pixel values) and 1 to 800/600 
    pub fn to_clip_space_x(value: f32) -> f32 {// Would it be better to use NDC?
        -800.0/600.0 + 2.0 * (800.0/600.0) * value
    }

    pub fn to_scale_x(value: f32) -> f32 {
        value * 2.0 * (800.0 / 600.0) 
    }

    // Maps 0 (top of screen) to 1 and 1 to -1
    pub fn to_clip_space_y(value: f32) -> f32 {
        1.0 - 2.0 * value
    }

    pub fn to_scale_y(value: f32) -> f32 {
        value * 2.0
    }
}
