use crate::gui::Gui;
use crate::state::components::Entity;

pub struct FrameState {
    pub objects_on_cursor: Vec<Entity>,
    nearest_object: Option<Entity>, //In orthographic we can't just calculate this by ray distance (all objects on plane will be same distance)

    pub handled_left_click: bool,
    pub handled_right_click: bool,
    pub handled_e_click: bool,
    pub handled_enter_click: bool,

    pub gui: Gui,

    pub action_requests: Vec<ActionRequest>,
    pub action_effects: Vec<ActionEffect>,
}

impl FrameState {
    pub fn new() -> FrameState {
        Self {
            objects_on_cursor: Vec::new(),
            nearest_object: None,
            handled_left_click: false,
            handled_right_click: false,
            handled_e_click: false,
            handled_enter_click: false,

            gui: Gui::new(),

            action_requests: Vec::new(),
            action_effects: Vec::new(),
        }
    }

    pub fn new_frame(&mut self) {
        self.handled_left_click = false;
        self.handled_right_click = false;
        self.handled_e_click = false;
        self.handled_enter_click = false;
        self.gui = Gui::new();
        self.action_requests = Vec::new();
        self.action_effects = Vec::new();
    }

    pub fn add_object(&mut self, object: Entity) {
        self.objects_on_cursor.push(object);
    }

    pub fn get_objects_on_cursor(&self) -> &Vec<Entity> {
        &self.objects_on_cursor
    }

    pub fn set_nearest_object_on_cursor(&mut self, nearest_object: Option<Entity>) {
        self.nearest_object = nearest_object;
    }

    pub fn get_nearest_object_on_cursor(&self) -> Option<&str> {
        self.nearest_object.as_deref()
    }
}

pub enum ActionRequest {
    ItemPlacement { entity: Entity },
}

pub enum ActionEffect {
    PickupItemNotStorable,
    PickupNoItemInRange,
    PlaceItemNotInInventory,
    PlaceItemNonPlaceable,
    PlaceItemCollidingItem,
    PlaceItemSucceeded,
    ItemSelected { found_objects_text: String },
    PickupNoInventorySpace,
    Examine { text: String },
}
