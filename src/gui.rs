use crate::components::Entity;
use crate::frame_state::FrameState;
use crate::game_state::GameState;
use crate::input::Input;
use crate::systems::item_placement_system::ItemPlacementSystem;
use cgmath::Point2;
use std::collections::HashMap;

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
    pub child_elements: HashMap<String, UIElement>, // Basically entity mapping... But think we want to separate ECS/UI
    pub on_click: Option<Box<dyn FnMut(&mut GameState, &mut UIState, &Input, &mut FrameState)>>,
}

impl UIElement {
    pub fn new_text(
        text: String,
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
    ) -> UIElement {
        Self {
            is_visible,
            position_top_left,
            position_bottom_right,
            width: position_bottom_right.x - position_top_left.x,
            height: position_bottom_right.y - position_top_left.y,
            payload: Payload::Text(text),
            child_elements: HashMap::new(),
            on_click: None,
        }
    }

    pub fn new_image<F>(
        image: String,
        is_visible: bool,
        position_top_left: Point2<f32>,
        position_bottom_right: Point2<f32>,
        on_click: Option<F>,
    ) -> UIElement
    where
        F: FnMut(&mut GameState, &mut UIState, &Input, &mut FrameState) + 'static,
    {
        Self {
            is_visible,
            position_top_left,
            position_bottom_right,
            width: position_bottom_right.x - position_top_left.x,
            height: position_bottom_right.y - position_top_left.y,
            payload: Payload::Image(image),
            child_elements: HashMap::new(),
            on_click: on_click.map(|f| {
                Box::new(f) as Box<dyn FnMut(&mut GameState, &mut UIState, &Input, &mut FrameState)>
            }),
        }
    }

    pub fn inventory_trigger_click(
        game_state: &mut GameState,
        ui_state: &mut UIState,
        input: &Input,
        frame_state: &mut FrameState,
    ) {
        if let Some(mut on_click) = ui_state.inventory.on_click.take() {
            on_click(game_state, ui_state, input, frame_state);
            ui_state.inventory.on_click = Some(on_click);
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

pub struct UIState {
    pub window_size: WindowSize,

    pub inventory: UIElement,
    pub action_text: UIElement,
    pub selected_text: UIElement,

    pub object_menu: Option<UIElement>,
    pub selected_objects_for_object_menu: Vec<Entity>, // Probably not the right place for this. Maybe in the ui element as payload?
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
                Some(
                    |game_state: &mut GameState,
                     ui_state: &mut UIState,
                     input: &Input,
                     frame_state: &mut FrameState| {
                        ItemPlacementSystem::handle_item_placement(
                            game_state,
                            ui_state,
                            input,
                            frame_state,
                        )
                    },
                ),
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

            object_menu: None,
            selected_objects_for_object_menu: Vec::new(),
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
