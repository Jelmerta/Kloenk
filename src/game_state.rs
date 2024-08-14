use std::sync::atomic::{AtomicU32, Ordering};

pub const TOTAL_DISTANCE: f32 = 200000.; // Verify naming, probbaly not total distance

pub struct GameState {
    pub camera_distance: f32,
    pub camera_rotation_x_degrees: f32, // as seen on a sphere, to figure out the position of the camera. it is not
    // the direction the camera is pointed at
    pub camera_rotation_y_degrees: f32,
    pub current_entity_id: AtomicU32,
    pub inventory_toggled: bool,
    pub inventory_position: Position, // Could be 2d. values between -1 and 1?
    pub inventory_item_count: u32,    // should be an item list
                                      //
    pub entities: Vec<Entity>,
}

pub struct Entity {
    pub id: String,
    pub graphics_component: GraphicsComponent,
    pub position: Position,
    pub previous_position: Position,
    pub size: Position, // HAcky reuse of position just a [f32; 3]?
    pub hitbox: f32,
}

impl Entity {
    pub fn get_position(&self) -> &Position {
        &self.position
    }

    pub fn update_position(&mut self, position: Position) {
        self.position = position;
    }
}

pub struct GraphicsComponent {
    pub model_id: String,
    pub material_id: String,
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    pub fn get_z(&self) -> f32 {
        self.z
    }
}

impl GameState {
    pub fn new() -> Self {
        let player = Entity {
            // id: u32::MAX,
            id: "player".to_string(), // hacky, just should not be used. player probably does not need id (yet)
            graphics_component: GraphicsComponent {
                model_id: "character".to_string(),
                material_id: "character".to_string(),
            },
            position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            previous_position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            size: Position {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            hitbox: 0.5,
        };

        let camera_distance = f32::sqrt(TOTAL_DISTANCE / 3.0);

        let current_entity_id: AtomicU32 = AtomicU32::new(0);
        let sword = Entity {
            // id: current_entity_id.fetch_add(1, Ordering::SeqCst),
            id: "sword".to_string(),
            graphics_component: GraphicsComponent {
                model_id: "sword".to_string(),
                material_id: "sword".to_string(),
            },
            position: Position {
                x: 1.1,
                y: 1.1,
                z: 0.0,
            },
            previous_position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            size: Position {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            hitbox: 0.51,
        };


        let plane = Entity {
            // id: u32::MAX - 1,
            id: "plane".to_string(),
            graphics_component: GraphicsComponent {
                model_id: "grass".to_string(),
                material_id: "grass".to_string(),
            },
            position: Position {
                x: 0.0,
                y: 0.0,
                z: -0.5,
            },
            previous_position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            size: Position {
                x: 25.0,
                y: 25.0,
                z: 0.1,
            },
            hitbox: 0.0,
        };

        let mut entities = Vec::new();
        entities.push(player);
        entities.push(sword);
        entities.push(plane);

        Self {
            camera_distance,
            camera_rotation_x_degrees: 225.0,
            camera_rotation_y_degrees: 315.0,
            current_entity_id,
            inventory_toggled: false,
            inventory_position: Position {
                x: 1.33,
                y: -0.9,
                z: 0.0,
            },
            inventory_item_count: 1,
            entities,
        }
    }

    pub fn get_entities(&self) -> &Vec<Entity> {
        // &?
        &self.entities
    }

    // pub fn remove_entity_from_world(&mut self, entity_to_remove: u32) {
    //     self.entities.retain(|entity| entity.id != entity_to_remove);
    // }

    pub fn remove_entity_from_world(&mut self, entity_to_remove: String) {
        self.entities.retain(|entity| entity.id != entity_to_remove);
    }
    
    pub(crate) fn add_item_to_inventory(&mut self) {
        self.inventory_item_count = self.inventory_item_count + 1;
    }

    // pub fn get_entity(&self, id: u32) -> Option<&Entity> {
    //     self.entities.iter().find(|entity| entity.id == id)
    // }

    pub fn get_entity(&self, id: String) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }

    pub fn get_player_const(&self) -> &Entity {
        self.entities.iter().find(|entity| entity.id == "player").unwrap()
    }
    pub fn get_player(&mut self) -> &mut Entity {
        self.entities.iter_mut().find(|entity| entity.id == "player").unwrap()
    }
    
    pub fn new_entity(&self, placement_position: Position) -> Entity {
        return Entity {
            // id,
            id: self.current_entity_id.fetch_add(1, Ordering::SeqCst).to_string(),
            graphics_component: GraphicsComponent {
                model_id: "sword".to_string(),
                material_id: "sword".to_string(),
            },
            position: placement_position,
            previous_position: placement_position,
            size: Position {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            hitbox: 0.51,
        }
    }
}
