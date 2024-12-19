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
        let point_clip = UIState::ui_to_clip(point, window);
        // log::warn!("hi");
        // log::warn!("Point clip: {:?}", point_clip);

        // let left = self.ui_element.ui_coordinate_origin.x - (self.ui_element.width() / 2.0) * (16.0 / 9.0) * (window.inner_size().height as f32 / window.inner_size().width as f32);
        // log::warn!("{:?}", self);
        let origin_clip = UIState::ui_to_clip(self.ui_coordinate_origin, window);
        // log::warn!("Origin clip: {:?}", origin_clip);
        let clip_left = origin_clip.x - self.top_left.x; // UIState::convert_scale_x(self.top_left.x); // - UIState::convert_scale_x(self.width() / 2.0);
        let clip_right = origin_clip.x + self.bottom_right.x; // + UIState::convert_scale_x(self.bottom_right.x); // + UIState::convert_scale_x(self.width() / 2.0);
        let clip_top = origin_clip.y + self.top_left.y; //UIState::convert_scale_y(self.top_left.y); // + UIState::convert_scale_y(self.height() / 2.0);
        let clip_bottom = origin_clip.y - self.bottom_right.y; //UIState::convert_scale_y(self.bottom_right.y); // - UIState::convert_scale_y(self.height() / 2.0);;
                                                               // let clip_left = UIState::clip_space_element_position_x(*self, window);
                                                               // let clip_top = UIState::convert_clip_space_y(self.ui_coordinate_origin.y) - UIState::convert_scale_y(self.height() / 2.0);
                                                               // let clip_right = clip_left + UIState::convert_scale_x(self.width());
                                                               // let clip_bottom = clip_top + UIState::convert_scale_y(self.height());

        // log::warn!("Clip left: {}, clip top: {}, clip right: {}, clip bottom: {}", clip_left, clip_top, clip_right, clip_bottom);

        // log::warn!("contains: {:?}", point_clip.x >= clip_left
        //     && point_clip.x < clip_right
        //     && point_clip.y >= clip_bottom
        //     && point_clip.y < clip_top);
        point_clip.x >= clip_left
            && point_clip.x < clip_right
            && point_clip.y >= clip_bottom
            && point_clip.y < clip_top
    }

    pub fn top(&self) -> f32 {
        self.ui_coordinate_origin.y - self.top_left.y
    }

    pub fn bottom(&self) -> f32 {
        self.ui_coordinate_origin.y + self.bottom_right.y
    }

    pub fn left(&self) -> f32 {
        self.ui_coordinate_origin.x - self.top_left.x
    }

    pub fn right(&self) -> f32 {
        self.ui_coordinate_origin.x + self.bottom_right.x
    }

    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    // TODO probably need some method that maintains aspect ratio by just defining topleft + width or height i guess?
    // Element space is defined from 0 to 1 in both x and y, bound by top_left and bottom_right defined in UI space (element space can also just be seen as percentages of root element's size)
    pub fn inner_rect(
        &self,
        element_space_top_left: Point2<f32>,
        element_space_bottom_right: Point2<f32>,
    ) -> UIElement {
        // let top_left = self.ui_coordinate_origin - element_space_top_left; //self.element_to_ui_x(element_space_top_left.x);
        // let bottom_right = self.ui_coordinate_origin + element_space_bottom_right;
        let ui_x_min = self.element_to_ui_x(element_space_top_left.x);
        let ui_y_min = self.element_to_ui_y(element_space_top_left.y);
        let ui_x_max = self.element_to_ui_x(element_space_bottom_right.x);
        let ui_y_max = self.element_to_ui_y(element_space_bottom_right.y);

        UIElement {
            ui_coordinate_origin: self.ui_coordinate_origin,
            // top_left: top_left.to_vec(),
            // bottom_right: bottom_right.to_vec(),
            top_left: Vector2 {
                x: ui_x_min,
                y: ui_y_min,
            },
            bottom_right: Vector2 {
                x: ui_x_max,
                y: ui_y_max,
            },
        }
    }

    // TODO
    pub fn inner_rect_maintain_ratio_x(
        &self,
        element_space_top_left: Point2<f32>,
        element_width_percentage: f32,
    ) -> UIElement {
        let ui_x_min = self.element_to_ui_x(element_space_top_left.x);
        let ui_y_min = self.element_to_ui_y(element_space_top_left.y);

        let inner_rect_width = self.width() * element_width_percentage;

        UIElement {
            ui_coordinate_origin: self.ui_coordinate_origin,
            top_left: Vector2 {
                x: ui_x_min,
                y: ui_y_min,
            },
            bottom_right: Vector2 {
                x: ui_x_min + inner_rect_width,
                y: ui_y_min + inner_rect_width,
            },
        }
    }

    fn element_to_ui_x(&self, element_space_x: f32) -> f32 {
        self.ui_coordinate_origin.x + self.top_left.x + element_space_x * self.width()
    }

    fn element_to_ui_y(&self, element_space_y: f32) -> f32 {
        self.ui_coordinate_origin.y + self.top_left.y + element_space_y * self.height()
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
            UIElement::new_rect(Point2::new(0.775, 0.775), Point2::new(0.175, 0.175)),
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

    pub fn ui_to_clip(point: Point2<f32>, window: &Arc<Window>) -> Point2<f32> {
        let adjusted_value =
            Self::clip_space_x_min(window) + point.x * Self::clip_space_width(window);
        let percentage = adjusted_value / Self::clip_space_width(window);
        let x_clip = percentage * Self::clip_space_width(window);

        let y_clip = 1.0 - point.y * 2.0;
        Point2::new(x_clip, y_clip)
    }

    pub fn clip_space_width(window: &Arc<Window>) -> f32 {
        Self::clip_space_x_max(window) - Self::clip_space_x_min(window)
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

    pub fn clip_space_element_position_x(ui_element: UIElement, window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;
        let distance_left = Self::convert_scale_x(ui_element.top_left.x); // TODO not 16:9 for top_left part?
        -width + 2.0 * width * ui_element.ui_coordinate_origin.x - distance_left
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
