use crate::state::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::{EuclideanSpace, Point2, Vector2};
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
    pub ui_coordinate_origin: Point2<f32>,
    pub top_left: Vector2<f32>,
    pub bottom_right: Vector2<f32>,
}

impl UIElement {
    pub fn new_rect(ui_coordinate_origin: Point2<f32>, half_dimensions: Point2<f32>) -> UIElement {
        let half_dimensions_vec = half_dimensions.to_vec();
        Self::new(
            ui_coordinate_origin,
            -half_dimensions_vec,
            half_dimensions_vec,
        )
    }

    pub fn new(
        ui_coordinate_origin: Point2<f32>,
        top_left: Vector2<f32>,
        bottom_right: Vector2<f32>,
    ) -> UIElement {
        Self {
            ui_coordinate_origin,
            top_left,
            bottom_right,
        }
    }

    pub fn contains(&self, point: Point2<f32>, window: &Arc<Window>) -> bool {
        point.x < self.right_ui(window)
            && point.x >= self.left_ui(window)
            && point.y < self.bottom()
            && point.y >= self.top()
    }

    pub fn top(&self) -> f32 {
        self.ui_coordinate_origin.y + self.top_left.y
    }

    pub fn bottom(&self) -> f32 {
        self.ui_coordinate_origin.y + self.bottom_right.y
    }

    pub fn left(&self) -> f32 {
        self.ui_coordinate_origin.x + self.top_left.x
    }

    pub fn left_ui(&self, window: &Arc<Window>) -> f32 {
        self.ui_coordinate_origin.x
            + self.top_left.x
            * (window.inner_size().height as f32 / window.inner_size().width as f32)
            * (16.0 / 9.0)
    }

    pub fn right_ui(&self, window: &Arc<Window>) -> f32 {
        self.ui_coordinate_origin.x
            + self.bottom_right.x
            * (window.inner_size().height as f32 / window.inner_size().width as f32)
            * (16.0 / 9.0)
    }

    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    // Element space is defined from 0 to 1 in both x and y, bound by top_left and bottom_right defined in UI space (element space can also just be seen as percentages of root element's size)
    pub fn inner_rect(
        &self,
        element_space_top_left: Point2<f32>,
        element_space_bottom_right: Point2<f32>,
    ) -> UIElement {
        let top =
            self.top() + element_space_top_left.y * self.height() - self.ui_coordinate_origin.y;
        let bottom =
            self.top() + element_space_bottom_right.y * self.height() - self.ui_coordinate_origin.y;
        let left =
            self.left() + element_space_top_left.x * self.width() - self.ui_coordinate_origin.x;
        let right =
            self.left() + element_space_bottom_right.x * self.width() - self.ui_coordinate_origin.x;

        UIElement {
            ui_coordinate_origin: self.ui_coordinate_origin,
            top_left: Vector2::new(left, top),
            bottom_right: Vector2::new(right, bottom),
        }
    }

    pub fn inner_rect_maintain_ratio_x(
        &self,
        element_space_top_left: Point2<f32>,
        element_width_percentage: f32,
    ) -> UIElement {
        let inner_rect_width = self.width() * element_width_percentage;
        let element_height_percentage = inner_rect_width / self.height();

        self.inner_rect(
            element_space_top_left,
            Point2::new(
                element_space_top_left.x + element_width_percentage,
                element_space_top_left.y + element_height_percentage,
            ),
        )
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

pub enum InputState {
    Normal,
    Chat,
}

impl InputState {}

pub struct UIState {
    pub windows: HashMap<String, UIWindow>,

    pub action_text: String,
    pub selected_text: String,

    pub menu_state: MenuState,
    pub dialogue_state: DialogueState,
    pub input_state: InputState,

    // pub cursor_bytes: Vec<u8>, // Not sure what a good place to store this data is
    // why do we even need this again?
}

impl UIState {
    // pub fn new(cursor: Vec<u8>) -> Self {
    pub fn new() -> Self {
        let mut windows = HashMap::new();

        let inventory_window = UIWindow::new(
            false,
            UIElement::new_rect(Point2::new(0.775, 0.775), Point2::new(0.175, 0.175)),
        );
        windows.insert("inventory".to_string(), inventory_window);

        let chat_window = UIWindow::new(
            false,
            UIElement::new_rect(Point2::new(0.25, 0.7), Point2::new(0.2, 0.1)),
        );
        windows.insert("chat".to_string(), chat_window);

        UIState {
            windows,
            menu_state: Closed,
            dialogue_state: DialogueState::Closed,
            input_state: InputState::Normal,
            action_text: String::new(),
            selected_text: String::new(),
            // cursor_bytes: cursor,
        }
    }

    pub fn clip_space_element_position_x(ui_element: UIElement, window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;
        let distance_left = Self::convert_scale_x(ui_element.top_left.x);
        -width + 2.0 * width * ui_element.ui_coordinate_origin.x + distance_left
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
