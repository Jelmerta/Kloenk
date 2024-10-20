use cgmath::Point2;

pub enum Payload {
    Text(String),
    // Container,
    #[allow(dead_code)]
    Image(String),
}

pub struct UIElement {
    pub is_visible: bool,
    pub position_top_left: Point2<f32>,
    pub position_bottom_right: Point2<f32>,
    pub width: f32, // Could be calculated field (or bottom right could be)
    pub height: f32,
    pub payload: Payload,
}

impl UIElement {
    pub fn new_text(
        text: String,
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            payload: Payload::Text(text),
            is_visible,
            position_top_left,
            position_bottom_right,
            width: position_bottom_right.x - position_top_left.x,
            height: position_bottom_right.y - position_top_left.y,
        }
    }

    pub fn new_image(
        image: String,
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            payload: Payload::Image(image),
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

    pub inventory: UIElement,
    pub action_text: UIElement,
    pub selected_text: UIElement,
}

impl UIState {
    pub fn new(width: u32, height: u32) -> Self {
        UIState {
            window_size: WindowSize { width, height },
            inventory: UIElement::new_image(
                "inventory".to_string(),
                false,
                Point2::new(0.6, 0.6),
                Point2::new(0.95, 0.95),
            ),

            // TODO We can probably store items here on a signal when inv changes. That
            // way we do not need to calculate inventory every frame when inventory is
            // shown
            action_text: UIElement::new_text(
                "".to_string(),
                false,
                Point2::new(0.05, 0.6),
                Point2::new(0.65, 1.0),
            ),

            selected_text: UIElement::new_text(
                "".to_string(),
                false,
                Point2::new(0.05, 0.1),
                Point2::new(0.65, 0.15),
            ),
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size.width = width;
        self.window_size.height = height;
    }

    // Maps 0 (left of screen) to -800/600 (pixel values) and 1 to 800/600
    pub fn convert_clip_space_x(value: f32, window_width: f32, window_height: f32) -> f32 {
        // Would it be better to use NDC?
        -window_width / window_height + 2.0 * (window_width / window_height) * value
    }

    pub fn convert_scale_x(value: f32, window_width: f32, window_height: f32) -> f32 {
        value * 2.0 * (window_width / window_height)
    }

    // Maps 0 (top of screen) to 1 and 1 to -1
    pub fn convert_clip_space_y(value: f32) -> f32 {
        1.0 - 2.0 * value
    }

    pub fn convert_scale_y(value: f32) -> f32 {
        value * 2.0
    }
}
