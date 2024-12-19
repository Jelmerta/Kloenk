use crate::state::components::Entity;
use crate::state::ui_state::MenuState::Closed;
use cgmath::{Point2, Vector2};
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

// #[derive(Copy, Clone, Debug)]
// pub enum ElementType {
//     Root { ui_element_coordinate_origin: Point2<f32> },
//     Inherit { ui_element_coordinate_origin: Point2<f32>, origin_distance: Vector2<f32> },
// }

// Instead of defining top_left and bottom_right, we define an element by its middle point. This helps our rendering: we want to enforce 16:9 UI, but have a dynamic resolution. This would lead to not-so-obvious solution. We now position elements by middle point and then render the element in 16:9
// #[derive(Copy, Clone, Debug)]
// pub struct UIElement {
//     pub element_type: ElementType,
//     pub half_dimensions: Point2<f32>,
// }

// Element space is defined from -1 to 1 in both x and y, bound by top_left and bottom_right defined in UI space
#[derive(Copy, Clone, Debug)]
pub struct UIElement {
    pub element_space_coordinate_origin: Point2<f32>,
    pub top_left: Point2<f32>,
    pub bottom_right: Point2<f32>,
}

impl UIElement {
    pub fn new(
        element_space_coordinate_origin: Point2<f32>,
        top_left: Point2<f32>,
        bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            element_space_coordinate_origin,
            top_left,
            bottom_right,
        }
    }

    // pub fn new_root(middle: Point2<f32>, half_dimensions: Point2<f32>) -> UIElement {
    //     Self {
    //         element_type: ElementType::Root { ui_element_coordinate_origin: middle },
    //         half_dimensions,
    //     }
    // }

    // pub fn new_child(parent: UIElement, position: Point2<f32>, half_dimensions: Point2<f32>) -> UIElement {
    //     Self {
    //         element_type: ElementType::Inherit {
    //             ui_element_coordinate_origin: match parent.element_type {
    //                 ElementType::Root { ui_element_coordinate_origin: middle } => middle,
    //                 ElementType::Inherit { ui_element_coordinate_origin: parent_middle, .. } => parent_middle,
    //             },
    //             origin_distance:
    //             match parent.element_type {
    //                 ElementType::Root { ui_element_coordinate_origin: middle } => {
    //                     position.sub(middle)
    //                 }
    //                 ElementType::Inherit { ui_element_coordinate_origin: parent_middle, origin_distance: _unusedithink } => {
    //                     position.sub(parent_middle)
    //                 }
    //             },
    //
    //
    //             // Vector2 {
    //             //     x: match parent {
    //             //         UIElement { element_type: ElementType::Root { middle }, .. } => 0.0,
    //             //         UIElement { element_type: ElementType::Inherit { parent_middle_distance: Vector2 { x, .. }, .. }, .. } => x,
    //             //     },
    //             //     y: match parent {
    //             //         UIElement { element_type: ElementType::Root { middle }, .. } => 0.0,
    //             //         UIElement { element_type: ElementType::Inherit { parent_middle_distance: Vector2 { y, .. }, .. }, .. } => y,
    //             //     },
    //             // },
    //         },
    //         half_dimensions,
    //     }
    // }

    // pub fn top_left(&self) -> Point2<f32> {
    //     Point2::new(self.middle.x - self.half_dimensions.x, self.middle.y - self.half_dimensions.y)
    // }
    //
    // pub fn bottom_right(&self) -> Point2<f32> {
    //     Point2::new(self.middle.x + self.half_dimensions.x, self.middle.y + self.half_dimensions.y)
    // }

    // pub fn clip_top(&self) -> f32 {
    //     let top_ui = self.ui_element_coordinate_origin_y() - self.half_dimensions.y;
    //     UIState::convert_clip_space_y(top_ui)
    // }
    //
    // pub fn clip_bottom(&self) -> f32 {
    //     let bottom_ui = self.ui_element_coordinate_origin_y() + self.half_dimensions.y;
    //     UIState::convert_clip_space_y(bottom_ui)
    // }
    //
    // pub fn right(&self) -> f32 {
    //     match self.element_type {
    //         // todo conversions? spaces
    //         ElementType::Root { ui_element_coordinate_origin } => { ui_element_coordinate_origin.x + self.half_dimensions.x }
    //         ElementType::Inherit { ui_element_coordinate_origin, origin_distance } => { ui_element_coordinate_origin.x + origin_distance.x + self.half_dimensions.x }
    //     }
    // }
    //
    // pub fn ui_element_coordinate_origin_y(&self) -> f32 {
    //     self.ui_element_coordinate_origin().y
    // }
    //
    // pub fn ui_element_coordinate_origin(&self) -> Point2<f32> {
    //     match self.element_type {
    //         ElementType::Root { ui_element_coordinate_origin } => ui_element_coordinate_origin,
    //         ElementType::Inherit { ui_element_coordinate_origin, .. } => ui_element_coordinate_origin,
    //     }
    // }

    pub fn contains(&self, point: Point2<f32>, window: &Arc<Window>) -> bool {
        let point_clip = UIState::ui_to_clip(point, window);
        let left = UIState::clip_space_left(*self, window);
        let right = UIState::clip_space_right(*self, window);

        // let middle = UIState::clip_space_left(*self, window);
        // let left = middle - UIState::convert_scale_x(self.half_dimensions.x);
        // let right = middle + UIState::convert_scale_x(self.half_dimensions.x);
        // point.x >= self.top_left().x
        //     && point.x < self.bottom_right().x
        // point.x >= left
        //     && point.x < right
        //     && point.y >= self.top_left().y
        //     && point.y < self.bottom_right().y

        point_clip.x >= left
            && point_clip.x < right
            && point_clip.y >= self.clip_top()
            && point_clip.y < self.clip_bottom()
    }

    // pub fn width(&self) -> f32 {
    //     self.half_dimensions.x * 2.0
    // }
    //
    // pub fn height(&self) -> f32 {
    //     self.half_dimensions.y * 2.0
    // }

    pub fn top(&self) -> f32 {
        match self.element_type {
            ElementType::Root {
                ui_element_coordinate_origin,
            } => ui_element_coordinate_origin.y - self.half_dimensions.y,
            ElementType::Inherit {
                ui_element_coordinate_origin,
                origin_distance,
            } => ui_element_coordinate_origin.y - origin_distance.y - self.half_dimensions.y,
        }
    }

    // Removes percentages on sides to create inner rect
    pub fn inner_rect(&self, width_to_remove: f32, height_to_remove: f32) -> UIElement {
        UIElement {
            element_type: ElementType::Inherit {
                ui_element_coordinate_origin: self.ui_element_coordinate_origin(),
                origin_distance: Vector2::new(0.0, 0.0),
            },
            half_dimensions: Point2::new(
                self.half_dimensions.x - width_to_remove,
                self.half_dimensions.y - height_to_remove,
            ),
        }

        // UIElement {
        //     middle: Point2::new(
        //         self.middle.x,
        //         self.middle.y,
        //     ),
        //     half_dimensions: Point2::new(
        //         self.half_dimensions.x - width_to_remove,
        //         self.half_dimensions.y - height_to_remove,
        //     ),
        //     parent_middle_distance: None, // Or zero
        // }
    }

    pub fn inner_rect(&self, middle_x_percentage: f32, middle_y_percentage: f32) -> UIElement {
        let ui_x_to_move = self.half_dimensions.x * middle_x_percentage;
        let ui_y_to_move = self.half_dimensions.y * middle_y_percentage;

        match self.element_type {
            ElementType::Root {
                ui_element_coordinate_origin,
            } => ui_element_coordinate_origin,
            ElementType::Inherit {
                ui_element_coordinate_origin,
                origin_distance,
            } => {}
        }

        UIElement {
            element_type: ElementType::Inherit {
                ui_element_coordinate_origin: self.ui_element_coordinate_origin(),
                origin_distance: Vector2::new(0.0, 0.0),
            },
            half_dimensions: Point2::new(
                self.half_dimensions.x - width_to_remove,
                self.half_dimensions.y - height_to_remove,
            ),
        }

        // UIElement {
        //     middle: Point2::new(
        //         self.middle.x,
        //         self.middle.y,
        //     ),
        //     half_dimensions: Point2::new(
        //         self.half_dimensions.x - width_to_remove,
        //         self.half_dimensions.y - height_to_remove,
        //     ),
        //     parent_middle_distance: None, // Or zero
        // }
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
            UIElement::new_root(Point2::new(0.775, 0.775), Point2::new(0.175, 0.175)),
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

        let y_clip = -1.0 + 2.0 * point.y;
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

    pub fn clip_space_left(ui_element: UIElement, window: &Arc<Window>) -> f32 {
        let adjusted_width_scaling = UIState::convert_scale_x(ui_element.half_dimensions.x);
        Self::clip_space_middle_x(ui_element, window) - adjusted_width_scaling
    }

    pub fn clip_space_right(ui_element: UIElement, window: &Arc<Window>) -> f32 {
        let adjusted_width_scaling = UIState::convert_scale_x(ui_element.half_dimensions.x);
        Self::clip_space_middle_x(ui_element, window) + adjusted_width_scaling
    }

    pub fn clip_space_middle_x(ui_element: UIElement, window: &Arc<Window>) -> f32 {
        let scale = 1.0;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let width = scale * resolution;

        match ui_element.element_type {
            ElementType::Root {
                ui_element_coordinate_origin,
            } => {
                // middle_scaling = -width + 2.0 * width * ui_element.middle.x;
                -width + 2.0 * width * ui_element_coordinate_origin.x
            }
            ElementType::Inherit {
                ui_element_coordinate_origin,
                origin_distance,
            } => {
                // -width + 2.0 * width * (ui_element.middle.x - ui_element.parent_middle_distance.unwrap().x) + Self::convert_scale_x(ui_element.parent_middle_distance.unwrap().x);
                -width
                    + 2.0 * width * (ui_element_coordinate_origin.x - origin_distance.x)
                    + Self::convert_scale_x(ui_element_coordinate_origin.x)
            }
        }

        // if ui_element.parent_middle_distance.is_some() {
        //     middle_scaling = -width + 2.0 * width * (ui_element.middle.x - ui_element.parent_middle_distance.unwrap().x) + Self::convert_scale_x(ui_element.parent_middle_distance.unwrap().x);
        // } else {}
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
