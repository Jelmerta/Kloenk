use crate::components::*;
use std::{collections::HashMap, sync::atomic::AtomicU32};
pub const TOTAL_DISTANCE: f32 = 200000.; // Verify naming, probbaly not total distance

pub struct GameState {
    pub current_entity_id: AtomicU32,
    // id: current_entity_id
    // .fetch_add(1, Ordering::SeqCst)
    // .to_string(),
    pub entities: Vec<Entity>,
    pub graphics_components: HashMap<Entity, Graphics>,
    pub position_components: HashMap<Entity, Position>,
    pub hitbox_components: HashMap<Entity, Hitbox>,
    pub camera_target_components: HashMap<Entity, CameraTarget>,
    pub storable_components: HashMap<Entity, Storable>,
    pub storage_components: HashMap<Entity, Storage>,
    pub in_storage_components: HashMap<Entity, InStorage>,
}

impl GameState {
    pub fn new() -> Self {
        // Initialise
        let current_entity_id: AtomicU32 = AtomicU32::new(0);

        let mut entities = Vec::new();
        let mut graphics_components = HashMap::new();
        let mut position_components = HashMap::new();
        let mut hitbox_components = HashMap::new();
        let mut camera_target_components = HashMap::new();
        let mut storable_components = HashMap::new();
        let mut storage_components = HashMap::new();
        let mut in_storage_components = HashMap::new();

        // Load player
        let player = "player".to_string();
        entities.push(player.clone());

        let player_graphics = Graphics {
            model_id: "character".to_string(),
            material_id: "character".to_string(),
        };
        graphics_components.insert(player.clone(), player_graphics);

        let player_position = Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        position_components.insert(player.clone(), player_position);

        let player_hitbox = Hitbox { hitbox: 0.5 };
        hitbox_components.insert(player.clone(), player_hitbox);

        let camera_target = CameraTarget {
            distance: f32::sqrt(TOTAL_DISTANCE / 3.0),
            rotation_x_degrees: 225.0,
            rotation_y_degrees: 315.0,
        };
        camera_target_components.insert(player.clone(), camera_target);

        let player_storage = Storage {
            number_of_rows: 8,
            number_of_columns: 8,
        };
        storage_components.insert(player.clone(), player_storage);

        // Load sword
        let sword = "sword".to_string();
        entities.push(sword.clone());

        let sword_graphics = Graphics {
            model_id: "sword".to_string(),
            material_id: "sword".to_string(),
        };
        graphics_components.insert(sword.clone(), sword_graphics);

        let sword_position = Position {
            x: 1.1,
            y: 1.1,
            z: 0.0,
        };
        position_components.insert(sword.clone(), sword_position);

        let sword_hitbox = Hitbox { hitbox: 0.51 };
        hitbox_components.insert(sword.clone(), sword_hitbox);

        let sword_storable = Storable {};
        storable_components.insert(sword.clone(), sword_storable);

        // let sword_in_player_inventory = InStorage {
            // storage_entity: player,
            // position_x: 0,
            // position_y: 0,
        // };
        // in_storage_components.insert(sword.clone(), sword_in_player_inventory);

        // Load tiles
        let map_x_min = -10;
        let map_x_max = 10;
        let map_y_min = -10;
        let map_y_max = 10;
        for x in map_x_min..map_x_max {
            for y in map_y_min..map_y_max {
                let plane = format!("plane{}{}", x, y); //todo copy?
                entities.push(plane.clone());

                let plane_graphics = Graphics {
                    model_id: "grass".to_string(),
                    material_id: "grass".to_string(),
                };
                graphics_components.insert(plane.clone(), plane_graphics);

                let plane_position = Position {
                    x: x as f32,
                    y: y as f32,
                    z: -1.0,
                };
                position_components.insert(plane.clone(), plane_position);

                //size?
                //hitbox 0?
            }
        }

        //     Self {
        //         camera_distance,
        //         camera_rotation_x_degrees: 225.0,
        //         camera_rotation_y_degrees: 315.0,
        //         current_entity_id,
        //         inventory_toggled: false,
        //         inventory_position: Position {
        //             x: 1.33,
        //             y: -0.9,
        //             z: 0.0,
        //         },
        //         inventory_item_count: 1,
        //         entities,
        //         world_object_components: HashMap::new(),
        //         inventory_components: HashMap::new(),
        //         inventory_item_components: HashMap::new(),
        //     }
        // }
        //

        Self {
            current_entity_id,
            entities,
            graphics_components,
            position_components,
            hitbox_components,
            camera_target_components,
            storable_components,
            storage_components,
            in_storage_components,
        }
    }

    pub fn get_graphics(&self, entity: Entity) -> Option<&Graphics> {
        return self.graphics_components.get(&entity);
    }

    pub fn create_position(&mut self, entity: Entity, position: Position) {
        self.position_components.insert(entity, position);
    }

    pub fn get_position(&self, entity: Entity) -> Option<&Position> {
        return self.position_components.get(&entity);
    }

    pub fn get_position_mut(&mut self, entity: Entity) -> Option<&mut Position> {
        return self.position_components.get_mut(&entity);
    }

    // pub fn get_positions(&self) -> Vec<&Entity> {
    //     // TODO Return pairs of keys and values or not?
    //     return self.position_components.keys().collect();
    // }

    pub fn remove_position(&mut self, to_remove: Entity) {
        self.position_components.remove(&to_remove);
    }

    // pub fn create_hitbox(&mut self, entity: Entity, distance: f32) {
    //     let hitbox = Hitbox { hitbox: distance };
    //     self.hitbox_components.insert(entity, hitbox);
    // }

    pub fn get_hitbox(&self, entity: Entity) -> Option<&Hitbox> {
        return self.hitbox_components.get(&entity);
    }

    pub fn get_camera(&self, entity: Entity) -> Option<&CameraTarget> {
        return self.camera_target_components.get(&entity);
    }

    pub fn get_camera_mut(&mut self, entity: Entity) -> Option<&mut CameraTarget> {
        return self.camera_target_components.get_mut(&entity);
    }

    pub fn get_storage(&self, entity: Entity) -> Option<&Storage> {
        return self.storage_components.get(&entity);
    }

    pub fn create_in_storage(&mut self, storage_entity: Entity, to_store: Entity, spot: (u8, u8)) {
        let in_storage_component = InStorage {
            storage_entity: storage_entity.clone(),
            position_x: spot.0,
            position_y: spot.1,
        };
        self.in_storage_components
            .insert(to_store, in_storage_component);
    }

    pub fn remove_in_storage(&mut self, entity: &Entity) {
        self.in_storage_components.remove(entity);
    }
    // pub fn get_entities(&self) -> &Vec<Entity> {
    //     // &?
    //     &self.entities
    // }

    // pub fn get_world_objects(&self) -> Vec<WorldObjectComponent> {
    //     return self.world_object_components.values().collect();
    // }

    // pub fn get_world_object(&self, world_object_entity_id: String) -> &WorldObjectComponent {
    //     return self.world_object_components.get(&world_object_entity_id).unwrap();
    // }

    // pub fn get_inventory(&self, inventory_entity_id: String) -> &InventoryComponent {
    //     return self.inventory_components.get(&inventory_entity_id).unwrap();
    // }
    //
    // pub fn get_inventory_item(&self, inventory_item_entity_id: String) -> &InventoryItemComponent {
    //     return self.inventory_item_components.get(&inventory_item_entity_id).unwrap();
    // }
    // // pub fn remove_entity_from_world(&mut self, entity_to_remove: u32) {
    // //     self.entities.retain(|entity| entity.id != entity_to_remove);
    // // }

    // pub fn remove_entity_from_world(&mut self, entity_to_remove: String) {
    //     self.entities.retain(|entity| entity.id != entity_to_remove);
    // }

    // pub fn get_entity(&self, id: u32) -> Option<&Entity> {
    //     self.entities.iter().find(|entity| entity.id == id)
    // }

    // pub fn get_entity(&self, id: String) -> Option<&Entity> {
    //     self.entities.iter().find(|entity| entity.id == id)
    // }

    // pub fn get_player_const(&self) -> &Entity {
    //     self.entities
    //         .iter()
    //         .find(|entity| entity.id == "player")
    //         .unwrap()
    // }
    // pub fn get_player(&mut self) -> &mut Entity {
    //     self.entities
    //         .iter_mut()
    //         .find(|entity| entity.id == "player")
    //         .unwrap()
    // }

    // pub fn new_entity(&self, placement_position: Position) -> Entity {
    //     return Entity {
    //         // id,
    //         id: self
    //             .current_entity_id
    //             .fetch_add(1, Ordering::SeqCst)
    //             .to_string(),
    //         graphics_component: GraphicsComponent {
    //             model_id: "sword".to_string(),
    //             material_id: "sword".to_string(),
    //         },
    //         position: placement_position,
    //         previous_position: placement_position,
    //         size: Position {
    //             x: 1.0,
    //             y: 1.0,
    //             z: 1.0,
    //         },
    //         hitbox: 0.51,
    //     };
    // }

    //     pub fn get_inventory_item_components(&self) -> Vec<InventoryItemComponent> {
    //         self.inventory_item_components.values().collect()
    //     }
}
