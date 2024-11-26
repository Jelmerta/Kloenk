use crate::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::{ElementWise, Point2};
use std::collections::HashMap;

pub struct UIWindow {
    pub is_visible: bool,
    pub rect: Rect,
}

impl UIWindow {
    pub fn new(is_visible: bool, rect: Rect) -> UIWindow {
        Self { is_visible, rect }
    }
}

pub enum RenderCommand {
    Mesh {
        layer: u32,
        rect: Rect,
        mesh_id: String,
    },
    Text {
        layer: u32,
        rect: Rect,
        text: String,
        color: [f32; 3],
    },
}

pub enum UserAction {
    None,
    Hover,
    LeftClick,
    RightClick,
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub top_left: Point2<f32>,
    pub bottom_right: Point2<f32>,
}

impl Rect {
    pub fn new(top_left: Point2<f32>, bottom_right: Point2<f32>) -> Rect {
        Self {
            top_left,
            bottom_right,
        }
    }

    pub fn contains(&self, point: Point2<f32>) -> bool {
        point.x >= self.top_left.x
            && point.x < self.bottom_right.x
            && point.y >= self.top_left.y
            && point.y < self.bottom_right.y
    }

    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    pub fn scale(&self, scale_factor_x: f32, scale_factor_y: f32) -> Rect {
        let scaled_top_left = self
            .top_left
            .mul_element_wise(Point2::new(scale_factor_x, scale_factor_y));
        let scaled_bottom_right = self
            .bottom_right
            .mul_element_wise(Point2::new(scale_factor_x, scale_factor_y));
        Rect::new(scaled_top_left, scaled_bottom_right)
    }
}

pub enum MenuState {
    Closed,
    World {
        mouse_position: Point2<f32>,
        item: Entity,
    },
    Inventory {
        mouse_position: Point2<f32>,
        item: Entity,
    },
}

pub struct UIState {
    pub window_size: WindowSize,
    pub windows: HashMap<String, UIWindow>,

    pub action_text: String,
    pub selected_text: String,

    pub menu_state: MenuState,
}

impl UIState {
    pub fn new(width: u32, height: u32) -> Self {
        let inventory_window = UIWindow::new(
            false,
            Rect::new(Point2::new(0.6, 0.6), Point2::new(0.95, 0.95)),
        );

        let mut windows = HashMap::new();
        windows.insert("inventory".to_string(), inventory_window);
        UIState {
            window_size: WindowSize { width, height },
            windows,
            menu_state: Closed,
            action_text: String::new(),
            selected_text: String::new(),
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

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}
