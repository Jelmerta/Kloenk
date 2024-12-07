use crate::state::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::{ElementWise, Point2};
use std::collections::HashMap;
use std::sync::Arc;
use winit::window::Window;

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

#[derive(Clone)]
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

impl MenuState {}

pub struct UIState {
    pub windows: HashMap<String, UIWindow>,

    pub action_text: String,
    pub selected_text: String,

    pub menu_state: MenuState,

    pub cursor: Vec<u8>, // Not sure what a good place to store this data is
}

impl UIState {
    pub fn new(cursor: Vec<u8>) -> Self {
        let inventory_window = UIWindow::new(
            false,
            Rect::new(Point2::new(0.6, 0.6), Point2::new(0.95, 0.95)),
        );

        let mut windows = HashMap::new();
        windows.insert("inventory".to_string(), inventory_window);
        UIState {
            windows,
            menu_state: Closed,
            action_text: String::new(),
            selected_text: String::new(),
            cursor,
        }
    }

    // Maps 0 (left of screen) to -800/600 (pixel values) and 1 to 800/600
    pub fn convert_clip_space_x(value: f32, window: &Arc<Window>) -> f32 {
        // Would it be better to use NDC?
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        -resolution + 2.0 * resolution * value
    }

    pub fn convert_scale_x(value: f32, window: &Arc<Window>) -> f32 {
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        value * 2.0 * resolution
    }

    // Maps 0 (top of screen) to 1 and 1 to -1
    pub fn convert_clip_space_y(value: f32) -> f32 {
        1.0 - 2.0 * value
    }

    pub fn convert_scale_y(value: f32) -> f32 {
        value * 2.0
    }
}
