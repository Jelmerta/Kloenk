use crate::state::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::{ElementWise, Point2};
use std::collections::HashMap;
use std::sync::Arc;
use winit::window::Window;

const DEFAULT_RESOLUTION_HEIGHT: f32 = 2160.0; // Using a default resolution to scale by, as dpi/pixelratio is independent of window size
const DEFAULT_RESOLUTION_WIDTH: f32 = 3840.0; // Using a default resolution to scale by, as dpi/pixelratio is independent of window size

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

    // Removes percentages on sides to create inner rect
    pub fn inner_rect(&self, width_to_remove: f32, height_to_remove: f32) -> Rect {
        Rect {
            top_left: Point2::new(
                self.top_left.x + width_to_remove,
                self.top_left.y + height_to_remove,
            ),
            bottom_right: Point2::new(
                self.bottom_right.x - width_to_remove,
                self.bottom_right.y - height_to_remove,
            ),
        }
    }
}

pub enum DialogueState {
    Closed,
    Npc {
        mouse_position: Point2<f32>,
        npc_entity_id: Entity,
        dialogue_id: String,
    },
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
    pub dialogue_state: DialogueState,

    pub cursor_bytes: Vec<u8>, // Not sure what a good place to store this data is
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
            dialogue_state: DialogueState::Closed,
            action_text: String::new(),
            selected_text: String::new(),
            cursor_bytes: cursor,
        }
    }

    pub fn convert_clip_space_x(value: f32, window: &Arc<Window>) -> f32 {
        let scale = window.inner_size().height as f32 / DEFAULT_RESOLUTION_HEIGHT;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;
        -width + 2.0 * width * value
    }

    pub fn scale_resolution(value: f32, window: &Arc<Window>) -> f32 {
        let scale = window.inner_size().height as f32 / DEFAULT_RESOLUTION_HEIGHT;
        value * scale
    }

    pub fn convert_scale_x(value: f32, window: &Arc<Window>) -> f32 {
        let scale = window.inner_size().height as f32 / DEFAULT_RESOLUTION_HEIGHT;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;
        value * 2.0 * width
    }

    pub fn convert_clip_space_y(value: f32, window: &Arc<Window>) -> f32 {
        let scale = window.inner_size().height as f32 / DEFAULT_RESOLUTION_HEIGHT;
        let height = scale;
        height - 2.0 * value * height
    }

    pub fn convert_scale_y(value: f32, window: &Arc<Window>) -> f32 {
        let scale = window.inner_size().height as f32 / DEFAULT_RESOLUTION_HEIGHT;
        let height = scale;
        value * height * 2.0
    }
}
