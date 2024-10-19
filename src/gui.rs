use cgmath::Point2;

pub struct UIElement {
    pub is_visible: bool,
    pub position_top_left: Point2<f32>,
    pub position_bottom_right: Point2<f32>,
    pub width: f32, // Could be calculated field (or bottom right could be)
    pub height: f32,
}

impl UIElement {
    pub fn new(
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            is_visible,
            position_top_left,
            position_bottom_right,
            width: position_bottom_right.x - position_top_left.x,
            height: position_bottom_right.y - position_top_left.y,
        }
    }

    pub fn contains(&self, position: Point2<f32>) -> bool {
        // TODO inclusion exclusion not really sure yet... probably not a big deal
        position.x >= self.position_top_left.x
            && position.x < self.position_bottom_right.x
            && position.y >= self.position_top_left.y
            && position.y < self.position_bottom_right.y
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub struct UIState {
    pub window_size: WindowSize,

    //pub inventory_open: bool,
    //pub inventory_position_x: f32,
    //pub inventory_position_y: f32,
    //pub inventory_width: f32,
    //pub inventory_height: f32,
    //pub inventory_position_top_left: Point2<f32>,
    //pub inventory_position_bottom_right: Point2<f32>,
    pub inventory: UIElement,

    pub text: String,
    pub text_position_x: f32,
    pub text_position_y: f32,
    pub text_width: f32,
    pub text_height: f32,

    pub selected_text: String,
    pub selected_text_position_x: f32,
    pub selected_text_position_y: f32,
    pub selected_text_width: f32,
    pub selected_text_height: f32,
}

impl UIState {
    pub fn new(width: u32, height: u32) -> Self {
        UIState {
            window_size: WindowSize { width, height },
            inventory: UIElement::new(false, Point2::new(0.6, 0.6), Point2::new(0.95, 0.95)),
            //inventory_open: false,

            //inventory_position_top_left:
            //inventory_position_bottom_right: Point2::new(0.95, 0.95),
            //inventory_width: 0.35,
            //inventory_height: 0.35,

            // TODO We can probably store items here on a signal when inv changes. That
            // way we do not need to calculate inventory every frame when inventory is
            // shown
            text: String::new(),
            text_position_x: 0.05,
            text_position_y: 0.6,
            text_width: 0.6,
            text_height: 0.4,

            selected_text: String::new(),
            selected_text_position_x: 0.05,
            selected_text_position_y: 0.1,
            selected_text_width: 0.6,
            selected_text_height: 0.4,
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size.width = width;
        self.window_size.height = height;
    }

    // Maps 0 (left of screen) to -800/600 (pixel values) and 1 to 800/600
    pub fn convert_clip_space_x(value: f32) -> f32 {
        // Would it be better to use NDC?
        -800.0 / 600.0 + 2.0 * (800.0 / 600.0) * value
    }

    pub fn convert_scale_x(value: f32) -> f32 {
        value * 2.0 * (800.0 / 600.0)
    }

    // Maps 0 (top of screen) to 1 and 1 to -1
    pub fn convert_clip_space_y(value: f32) -> f32 {
        1.0 - 2.0 * value
    }

    pub fn convert_scale_y(value: f32) -> f32 {
        value * 2.0
    }
}
