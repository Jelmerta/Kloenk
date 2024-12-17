use crate::state::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::Point2;
use std::collections::HashMap;
use std::sync::Arc;
use winit::window::Window;

pub struct UIWindow {
    pub is_visible: bool,
    pub rect: UIElement,
}

impl UIWindow {
    pub fn new(is_visible: bool, rect: UIElement) -> UIWindow {
        Self { is_visible, rect }
    }
}

pub enum RenderCommand {
    Mesh {
        layer: u32,
        ui_element: UIElement,
        mesh_id: String,
    },
    Text {
        layer: u32,
        rect: UIElement,
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
pub struct UIElement {
    pub parent_middle_x: Option<f32>,
    pub top_left: Point2<f32>,
    pub bottom_right: Point2<f32>,
}

impl UIElement {
    pub fn new(
        top_left: Point2<f32>,
        bottom_right: Point2<f32>,
        parent_middle_x: Option<f32>,
    ) -> UIElement {
        Self {
            top_left,
            bottom_right,
            parent_middle_x,
        }
    }

    pub fn contains(&self, point: Point2<f32>) -> bool {
        point.x >= self.top_left.x
            && point.x < self.bottom_right.x
            && point.y >= self.top_left.y
            && point.y < self.bottom_right.y
    }

    pub fn middle(&self) -> Point2<f32> {
        Point2::new(
            (self.top_left.x + self.bottom_right.x) / 2.0,
            (self.top_left.y + self.bottom_right.y) / 2.0,
        )
    }

    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    // Removes percentages on sides to create inner rect
    pub fn inner_rect(&self, width_to_remove: f32, height_to_remove: f32) -> UIElement {
        UIElement {
            top_left: Point2::new(
                self.top_left.x + width_to_remove,
                self.top_left.y + height_to_remove,
            ),
            bottom_right: Point2::new(
                self.bottom_right.x - width_to_remove,
                self.bottom_right.y - height_to_remove,
            ),
            parent_middle_x: None,
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
            UIElement::new(Point2::new(0.6, 0.6), Point2::new(0.95, 0.95), None),
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

    pub fn clip_space_x_min(window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;
        -width
    }

    pub fn clip_space_x_max(window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;
        width
    }

    pub fn convert_clip_space_x(ui_element: UIElement, window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;

        let middle_scaling;
        if ui_element.parent_middle_x.is_some() {
            middle_scaling = -width
                + 2.0 * width * ui_element.parent_middle_x.unwrap()
                + Self::convert_scale_x(
                    ui_element.middle().x - ui_element.parent_middle_x.unwrap(),
                ); // * (window.inner_size().height as f32 / window.inner_size().width as f32);
        } else {
            middle_scaling = -width + 2.0 * width * ui_element.middle().x;
        }
        let adjusted_width_scaling = UIState::convert_scale_x(ui_element.width() / 2.0);
        middle_scaling - adjusted_width_scaling
    }

    pub fn convert_scale_x(value: f32) -> f32 {
        let scale = 1.0;
        let resolution = 16.0 / 9.0;
        let width = scale * resolution;
        value * 2.0 * width
    }

    pub fn convert_clip_space_y(value: f32) -> f32 {
        let scale = 1.0;
        let height = scale;
        height - 2.0 * value * height
    }

    pub fn convert_scale_y(value: f32) -> f32 {
        let scale = 1.0;
        let height = scale;
        value * height * 2.0
    }
}
