use crate::components::Entity;

pub struct FrameState {
    objects_on_cursor: Vec<Entity>,
    nearest_object: Option<Entity>, //In orthographic we can't just calculate this by ray distance (all objects on plane will be same distance)
}

impl FrameState {
    pub fn new() -> FrameState {
        Self {
            objects_on_cursor: Vec::new(),
            nearest_object: None,
        }
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

    pub fn get_nearest_object_on_cursor(&self) -> Option<&Entity> {
        self.nearest_object.as_ref()
    }
}
