use crate::state::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::{EuclideanSpace, Point2, Vector2};
use std::collections::HashMap;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
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
    Model {
        layer: u32,
        ui_element: UIElement,
        model_id: String,
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

pub const SCREEN_REFERENCE_WIDTH: f32 = 1920.0;
pub const SCREEN_REFERENCE_HEIGHT: f32 = 1080.0;

#[derive(Copy, Clone)]
pub struct UIElement {
    // pub ui_coordinate_origin: Point2<f32>,
    //
    // pub anchor_type: Anchor,

    pub anchor_x: f32,
    pub anchor_y: f32,
    pub anchor_offset_x: f32,
    pub anchor_offset_y: f32,
    pub width: f32,
    pub height: f32,

    pub scaled_anchor_x: f32,
    pub scaled_anchor_y: f32,
    pub scaled_x: f32,
    pub scaled_y: f32,
    pub scaled_width: f32,
    pub scaled_height: f32,
    // pub top: f32,
    // pub bottom: f32,
    // pub left: f32,
    // pub right: f32,

    // These fields adjust upon resize such that UI elements scale right without stretching. Dependent on resolution ratio of screen
    // pub left_viewport: f32,
    // pub right_viewport: f32,
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

    // TODO maybe eventually keep track of ui elements to reuse if possible with immediate mode gui such that we only need updates and dont need to pass window?
    // update_state.new_element()/update_element/remove_element? not removing the elements every update?
    fn new(
        ui_coordinate_origin: Point2<f32>,
        top_left: Vector2<f32>,
        bottom_right: Vector2<f32>,
    ) -> UIElement {
        Self {
            // ui_coordinate_origin,
            // top: top_left.y,
            // bottom: bottom_right.y,
            // left: top_left.x,
            // right: bottom_right.x,

            // Assume anchor top left rn
            anchor_x: (ui_coordinate_origin.x + top_left.x) * SCREEN_REFERENCE_WIDTH,
            anchor_y: (ui_coordinate_origin.y + top_left.y) * SCREEN_REFERENCE_HEIGHT,
            anchor_offset_x: 0.0,
            anchor_offset_y: 0.0,
            width: bottom_right.x * 2.0 * SCREEN_REFERENCE_WIDTH, // TODO assume half dimensions for now
            height: bottom_right.y * 2.0 * SCREEN_REFERENCE_HEIGHT, // TODO assume half dimensions for now

            scaled_anchor_x: 0.0,
            scaled_anchor_y: 0.0,
            scaled_x: 0.0,
            scaled_y: 0.0,
            scaled_width: 0.0,
            scaled_height: 0.0,
        }
    }

    // TODO scaling above 1080 also needs to happen: in the UI space the elements need to become smaller in order to remain same size in larger screen...
    pub fn update(&mut self, window_size: &PhysicalSize<u32>) {
        let scale_x = window_size.width as f32 / SCREEN_REFERENCE_WIDTH;
        let scale_y = window_size.height as f32 / SCREEN_REFERENCE_HEIGHT;
        // let scale = (window_size.width as f32 / SCREEN_REFERENCE_WIDTH)
        //     * (window_size.height as f32 / SCREEN_REFERENCE_HEIGHT); // TODO might not be the best: maybe scale a bit slower? idk
        self.scaled_anchor_x = self.anchor_x * scale_x;
        self.scaled_anchor_y = self.anchor_y * scale_y;
        self.scaled_x = self.anchor_offset_x * f32::min(1.0, scale_x);
        self.scaled_y = self.anchor_offset_y * f32::min(1.0, scale_y);
        self.scaled_width = f32::min(self.width, self.width * scale_x);
        self.scaled_height = f32::min(self.height, self.height * scale_y);
    }

    // We first map cursor to reference
    // TODO how does this work for ui elements with parents?
    // TODO can this be window-agnostic somehow?
    // TODO probably dont need to pass window: just pass cursor point in pixels (still needed for update though...)
    pub fn contains(&mut self, cursor_point: Point2<f32>, window: &Arc<Window>) -> bool {
        self.update(&window.inner_size()); // TODO probably dont update here
        let cursor_x = cursor_point.x * window.inner_size().width as f32;
        let cursor_y = cursor_point.y * window.inner_size().height as f32;
        cursor_x >= self.scaled_anchor_x + self.scaled_x
            && cursor_x < self.scaled_anchor_x + self.scaled_x + self.scaled_width
            && cursor_y >= self.scaled_anchor_y + self.scaled_y
            && cursor_y < self.scaled_anchor_y + self.scaled_y + self.scaled_height
        // cursor_point.x < self.right_ui(window) // TODO on resize, can we not calculate new right/left? such that windae is not needed
    }

    // pub fn top(&self) -> f32 {
    //     self.ui_coordinate_origin.y + self.top
    // }
    //
    // pub fn bottom(&self) -> f32 {
    //     self.ui_coordinate_origin.y + self.bottom
    // }

    // pub fn left_ui(&self, window: &Arc<Window>) -> f32 {
    //     self.ui_coordinate_origin.x
    //         + self.left
    //         * (window.inner_size().height as f32 / window.inner_size().width as f32)
    //         * (16.0 / 9.0)
    // }

    // pub fn right_ui(&self, window: &Arc<Window>) -> f32 {
    //     self.ui_coordinate_origin.x
    //         + self.right
    //         * (window.inner_size().height as f32 / window.inner_size().width as f32)
    //         * (16.0 / 9.0)
    // }

    // pub fn width(&self) -> f32 {
    //     self.right - self.left
    // }
    pub fn width(&self) -> f32 {
        self.width
    }

    // pub fn height(&self) -> f32 {
    //     self.bottom - self.top
    // }
    pub fn height(&self) -> f32 {
        self.height
    }

    // Element space is defined from 0 to 1 in both x and y, bound by top_left and bottom_right defined in UI space (element space can also just be seen as percentages of root element's size)
    pub fn inner_rect(
        &self,
        element_space_top_left: Point2<f32>,
        element_space_bottom_right: Point2<f32>,
    ) -> UIElement {
        // self.update()
        // TODO offset from anchor
        // let top = self.scaled_y + element_space_top_left.y * self.height;
        // let bottom =
        //     self.anchor_y + element_space_bottom_right.y * self.height();
        // let left = self.scaled_x + element_space_top_left.x * self.width;
        // let right =
        //     self.anchor_x + element_space_bottom_right.x * self.width();
        UIElement {
            anchor_x: self.anchor_x,
            anchor_y: self.anchor_y,
            anchor_offset_x: element_space_top_left.x * self.width,
            anchor_offset_y: element_space_top_left.y * self.height,
            width: (element_space_bottom_right.x - element_space_top_left.x) * self.width,
            height: (element_space_bottom_right.y - element_space_top_left.y) * self.height,

            scaled_anchor_x: 0.0,
            scaled_anchor_y: 0.0,
            scaled_x: 0.0,
            scaled_y: 0.0,
            scaled_width: 0.0,
            scaled_height: 0.0,
            // ui_coordinate_origin: self.ui_coordinate_origin,
            // top,
            // bottom,
            // left,
            // right,
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
        render_position: Point2<f32>,
        npc_entity_id: Entity,
        dialogue_id: String,
    },
}

#[derive(Clone)]
pub enum MenuState {
    Closed,
    WorldAction {
        render_position: Point2<f32>,
        item: Entity,
    },
    InventoryAction {
        render_position: Point2<f32>,
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
}

impl UIState {
    pub fn new() -> Self {
        let mut windows = HashMap::new();

        let inventory_window = UIWindow::new(
            false,
            UIElement::new_rect(Point2::new(0.775, 0.775), Point2::new(0.175, 0.175)),
        );
        windows.insert("inventory".to_owned(), inventory_window);

        let chat_window = UIWindow::new(
            false,
            UIElement::new_rect(Point2::new(0.25, 0.7), Point2::new(0.2, 0.1)),
        );
        windows.insert("chat".to_owned(), chat_window);

        UIState {
            windows,
            menu_state: Closed,
            dialogue_state: DialogueState::Closed,
            input_state: InputState::Normal,
            action_text: String::new(),
            selected_text: String::new(),
        }
    }

    // Reuse camera method for calculation ? TODO
    pub fn clip_space_element_position_x(ui_element: &UIElement, window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let viewport_half_width = scale * resolution;
        -viewport_half_width + Self::convert_scale_x(ui_element.scaled_anchor_x + ui_element.scaled_x, window)
    }

    pub fn clip_space_element_position_y(ui_element: &UIElement, window: &Arc<Window>) -> f32 {
        let viewport_half_height = 1.0;
        viewport_half_height - Self::convert_scale_y(ui_element.scaled_anchor_y + ui_element.scaled_y, window)
    }

    // TODO logic probably should be in ui element not in rendering, just calculate there upon window size change. also needed for contains
    pub fn convert_scale_x(value: f32, window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let viewport_width = 2.0 * scale * resolution;

        value / window.inner_size().width as f32 * viewport_width
    }

    pub fn convert_scale_y(value: f32, window: &Arc<Window>) -> f32 {
        let viewport_half_height = 1.0;

        value / window.inner_size().height as f32 * viewport_half_height * 2.0
    }
}
