use crate::components::Entity;
use crate::input::Input;
use cgmath::{ElementWise, Point2};
use std::collections::HashMap;

pub struct UIWindow {
    // pub name: String,
    pub is_visible: bool,
    // pub layer: u32,
    pub rect: Rect,
}

impl UIWindow {
    pub fn new(
        // name: String,
        is_visible: bool,
        // layer: u32,
        // position_top_left: Point2<f32>,
        // position_bottom_right: Point2<f32>,
        rect: Rect,
    ) -> UIWindow {
        Self {
            // name,
            is_visible,
            // layer,
            rect,
        }
    }
}

pub enum RenderCommand {
    Image { rect: Rect, image_name: String },
    Text { rect: Rect, text: String },
}

// impl RenderCommand {
//     pub fn new(position_top_left: Point2<f32>, position_bottom_right: Point2<f32>) -> Renderable {
//         Self {
//             position_top_left,
//             position_bottom_right,
//         }
//     }
// }

pub enum UserAction {
    None,
    LeftClick,
    RightClick,
}

pub struct Gui {
    pub render_commands: Vec<RenderCommand>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            render_commands: Vec::new(),
        }
    }

    pub fn image(&mut self, rect: Rect, image_name: String) {
        let image_command = RenderCommand::Image { rect, image_name };
        self.render_commands.push(image_command);
    }

    pub fn image_button(&mut self, rect: Rect, image_name: String, input: &Input) -> UserAction {
        let image_command = RenderCommand::Image { rect, image_name };
        self.render_commands.push(image_command);
        if rect.contains(input.mouse_position_ui) && input.left_mouse_clicked.is_toggled_on() {
            return UserAction::LeftClick;
        }
        if rect.contains(input.mouse_position_ui) && input.right_mouse_clicked.is_toggled_on() {
            return UserAction::RightClick;
        }
        UserAction::None
        // TODO probably on release
    }

    pub fn text(&mut self, rect: Rect, text: String) {
        let text_command = RenderCommand::Text { rect, text };
        self.render_commands.push(text_command);
    }

    pub fn add_text_render_commands(&mut self, ui_state: &UIState) {
        self.text(
            Rect::new(Point2::new(0.05, 0.6), Point2::new(0.65, 1.0)),
            ui_state.action_text.clone(),
        );

        self.text(
            Rect::new(Point2::new(0.05, 0.1), Point2::new(0.65, 0.15)),
            ui_state.selected_text.clone(),
        );
    }
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

pub struct UIState {
    pub window_size: WindowSize,
    pub windows: HashMap<String, UIWindow>,
    pub selected_objects_for_object_menu: Vec<Entity>, // Probably not the right place for this. Maybe in the ui element as payload?
    // pub elements: Vec<UIElement>,

    // pub inventory: UIElement,
    pub action_text: String,
    pub selected_text: String,
    //
    // pub object_menu: Option<UIElement>,
    pub object_menu_open: bool,
    pub object_menu_mouse_position: Point2<f32>,
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
            // inventory: UIElement::new_image(
            //     "inventory".to_string(),
            //     50,
            //     "inventory".to_string(),
            //     false,
            //     Point2::new(0.6, 0.6),
            //     Point2::new(0.95, 0.95),
            // ),

            // TODO We can probably store items here on a signal when inv changes. That
            // way we do not need to calculate inventory every frame when inventory is
            // shown
            // action_text: UIElement::new_text(
            //     "action_text".to_string(),
            //     1000,
            //     "".to_string(),
            //     true,
            //     Point2::new(0.05, 0.6),
            //     Point2::new(0.65, 1.0),
            // ),

            // selected_text: UIElement::new_text(
            //     "selected_text".to_string(),
            //     1000,
            //     "".to_string(),
            //     true,
            //     Point2::new(0.05, 0.1),
            //     Point2::new(0.65, 0.15),
            // ),

            // object_menu: None,
            selected_objects_for_object_menu: Vec::new(),
            object_menu_open: false,
            object_menu_mouse_position: Point2::new(0.0, 0.0),
            // elements: vec![
            //     UIElement::new_image(
            //         "inventory".to_string(),
            //         50,
            //         "inventory".to_string(),
            //         false,
            //         Point2::new(0.6, 0.6),
            //         Point2::new(0.95, 0.95),
            //     ),
            //     UIElement::new_text(
            //         "action_text".to_string(),
            //         1000,
            //         "".to_string(),
            //         true,
            //         Point2::new(0.05, 0.6),
            //         Point2::new(0.65, 1.0),
            //     ),
            //     UIElement::new_text(
            //         "selected_text".to_string(),
            //         1000,
            //         "".to_string(),
            //         true,
            //         Point2::new(0.05, 0.1),
            //         Point2::new(0.65, 0.15),
            //     ),
            // ],
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

pub enum Payload {
    Text(String),
    // Container,
    #[allow(dead_code)]
    Image(String),
}

pub struct UIElement {
    pub name: String,
    pub is_visible: bool,

    pub layer: u32,
    pub position_top_left: Point2<f32>,
    pub position_bottom_right: Point2<f32>,
    pub width: f32, // Could be calculated field (or bottom right could be)
    pub height: f32,

    pub payload: Payload,
    pub child_elements: HashMap<String, UIElement>, // Basically entity mapping... But think we want to separate ECS/UI
}

impl UIElement {
    pub fn new_text(
        name: String,
        layer: u32,
        text: String,
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            name,
            is_visible,
            layer,
            position_top_left,
            position_bottom_right,
            width: position_bottom_right.x - position_top_left.x,
            height: position_bottom_right.y - position_top_left.y,
            payload: Payload::Text(text),
            child_elements: HashMap::new(),
        }
    }

    pub fn new_image(
        name: String,
        layer: u32,
        image: String,
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            name,
            layer,
            is_visible,
            position_top_left,
            position_bottom_right,
            width: position_bottom_right.x - position_top_left.x,
            height: position_bottom_right.y - position_top_left.y,
            payload: Payload::Image(image),
            child_elements: HashMap::new(),
        }
    }

    pub fn contains(&self, position: Point2<f32>) -> bool {
        // TODO inclusion exclusion not really sure yet... probably not a big deal
        position.x >= self.position_top_left.x
            && position.x < self.position_bottom_right.x
            && position.y >= self.position_top_left.y
            && position.y < self.position_bottom_right.y
    }

    pub fn to_ui_element_space(&self, external_point: Point2<f32>) -> Point2<f32> {
        if !self.contains(external_point) {
            panic!("Unintended call. Can only map from point within ui element");
        }

        let relative_x = external_point.x - self.position_top_left.x;
        let relative_y = external_point.y - self.position_top_left.y;

        let normalized_x = relative_x / (self.position_bottom_right.x - self.position_top_left.x);
        let normalized_y = relative_y / (self.position_bottom_right.y - self.position_top_left.y);

        Point2::new(normalized_x, normalized_y)
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}
