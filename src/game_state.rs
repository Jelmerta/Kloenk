use cgmath::num_traits::ToPrimitive;

use crate::components::{
    CameraTarget, Entity, Graphics2D, Graphics3D, Hitbox, InStorage, ItemShape, Position, Storable,
    Storage,
};
// use std::{collections::HashMap, sync::atomic::AtomicU32};
use std::collections::{HashMap, HashSet};
use std::f32;

pub const TOTAL_DISTANCE: f32 = 200_000.; // Verify naming, probably not total distance

pub struct GameState {
    // pub current_entity_id: AtomicU32,
    // id: current_entity_id
    // .fetch_add(1, Ordering::SeqCst)
    // .to_string(),
    pub entities: Vec<Entity>,
    pub graphics_3d_components: HashMap<Entity, Graphics3D>,
    pub graphics_2d_components: HashMap<Entity, Graphics2D>,
    pub position_components: HashMap<Entity, Position>,
    pub surface_components: HashSet<Entity>,
    pub hitbox_components: HashMap<Entity, Hitbox>,
    pub camera_target_components: HashMap<Entity, CameraTarget>,
    pub storable_components: HashMap<Entity, Storable>,
    pub storage_components: HashMap<Entity, Storage>,
    pub in_storage_components: HashMap<Entity, InStorage>,
}

impl GameState {
    pub fn new() -> Self {
        // Initialise
        // let current_entity_id: AtomicU32 = AtomicU32::new(0);

        let mut entities = Vec::new();
        let mut graphics_3d_components = HashMap::new();
        let mut graphics_2d_components = HashMap::new();
        let mut position_components = HashMap::new();
        let mut surface_components = HashSet::new();
        let mut hitbox_components = HashMap::new();
        let mut camera_target_components = HashMap::new();
        let mut storable_components = HashMap::new();
        let mut storage_components = HashMap::new();
        let in_storage_components = HashMap::new();

        Self::load_player(
            &mut entities,
            &mut graphics_3d_components,
            &mut position_components,
            &mut hitbox_components,
            &mut camera_target_components,
            &mut storage_components,
        );
        Self::load_shield(
            &mut entities,
            &mut graphics_3d_components,
            &mut graphics_2d_components,
            &mut position_components,
            &mut hitbox_components,
            &mut storable_components,
        );
        Self::load_swords(
            &mut entities,
            &mut graphics_3d_components,
            &mut graphics_2d_components,
            &mut position_components,
            &mut hitbox_components,
            &mut storable_components,
        );
        Self::load_tiles(
            &mut entities,
            &mut graphics_3d_components,
            &mut position_components,
            &mut surface_components,
        );
        Self::load_tree(
            &mut entities,
            &mut graphics_3d_components,
            &mut position_components,
            &mut hitbox_components,
        );

        Self {
            // current_entity_id,
            entities,
            graphics_3d_components,
            graphics_2d_components,
            position_components,
            surface_components,
            hitbox_components,
            camera_target_components,
            storable_components,
            storage_components,
            in_storage_components,
        }
    }

    fn load_tree(
        entities: &mut Vec<String>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        position_components: &mut HashMap<String, Position>,
        hitbox_components: &mut HashMap<String, Hitbox>,
    ) {
        let tree = "tree".to_string();
        entities.push(tree.clone());

        let tree_graphics = Graphics3D {
            model_id: "tree".to_string(),
        };
        graphics_3d_components.insert(tree.clone(), tree_graphics);

        let tree_position = Position {
            x: 2.0,
            y: -3.0,
            z: 0.0,
        };
        position_components.insert(tree.clone(), tree_position);

        let tree_hitbox = Hitbox { hitbox: 0.51 };
        hitbox_components.insert(tree.clone(), tree_hitbox);
    }

    fn load_tiles(
        entities: &mut Vec<String>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        position_components: &mut HashMap<String, Position>,
        surface_components: &mut HashSet<String>,
    ) {
        let plane_longitude_minimum: i8 = -10;
        let plane_longitude_maximum: i8 = 10;
        let plane_latitude_minimum: i8 = -10;
        let plane_latitude_maximum: i8 = 10;
        for x in plane_longitude_minimum..plane_longitude_maximum {
            for y in plane_latitude_minimum..plane_latitude_maximum {
                let plane = format!("plane{x}{y}"); //todo copy?
                entities.push(plane.clone());

                let plane_graphics = Graphics3D {
                    model_id: "grass".to_string(),
                };
                graphics_3d_components.insert(plane.clone(), plane_graphics);

                let plane_position = Position {
                    x: f32::from(x),
                    y: f32::from(y),
                    z: -1.0,
                };
                position_components.insert(plane.clone(), plane_position);

                surface_components.insert(plane.clone());
            }
        }
    }

    fn load_swords(
        entities: &mut Vec<String>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        graphics_2d_components: &mut HashMap<String, Graphics2D>,
        position_components: &mut HashMap<String, Position>,
        hitbox_components: &mut HashMap<String, Hitbox>,
        storable_components: &mut HashMap<String, Storable>,
    ) {
        for i in 1..71 {
            let sword = "sword".to_string() + &i.to_string();
            entities.push(sword.clone());

            let sword_graphics = Graphics3D {
                model_id: "sword".to_string(),
            };
            graphics_3d_components.insert(sword.clone(), sword_graphics);

            let sword_graphics_inventory = Graphics2D {
                model_id: "sword_inventory".to_string(),
            };
            graphics_2d_components.insert(sword.clone(), sword_graphics_inventory);

            let sword_position = Position {
                x: i.to_f32().unwrap() + 0.1,
                y: i.to_f32().unwrap() + 0.1,
                z: 0.0,
            };
            position_components.insert(sword.clone(), sword_position);

            let sword_hitbox = Hitbox { hitbox: 0.51 };
            hitbox_components.insert(sword.clone(), sword_hitbox);

            let sword_storable = Storable {
                shape: ItemShape {
                    width: 1,
                    height: 1,
                },
            };
            storable_components.insert(sword.clone(), sword_storable);
        }
    }

    fn load_shield(
        entities: &mut Vec<String>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        graphics_2d_components: &mut HashMap<String, Graphics2D>,
        position_components: &mut HashMap<String, Position>,
        hitbox_components: &mut HashMap<String, Hitbox>,
        storable_components: &mut HashMap<String, Storable>,
    ) {
        let shield = "shield".to_string();
        entities.push(shield.clone());
        let shield_graphics = Graphics3D {
            model_id: "shield".to_string(),
        };
        graphics_3d_components.insert(shield.clone(), shield_graphics);

        let shield_graphics_inventory = Graphics2D {
            model_id: "shield_inventory".to_string(),
        };
        graphics_2d_components.insert(shield.clone(), shield_graphics_inventory);

        let shield_position = Position {
            x: -2.7,
            y: -2.7,
            z: 0.0,
        };
        position_components.insert(shield.clone(), shield_position);

        let shield_hitbox = Hitbox { hitbox: 0.51 };
        hitbox_components.insert(shield.clone(), shield_hitbox);

        let shield_storable = Storable {
            shape: ItemShape {
                width: 1,
                height: 2,
            },
        };
        storable_components.insert(shield.clone(), shield_storable);
    }

    fn load_player(
        entities: &mut Vec<String>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        position_components: &mut HashMap<String, Position>,
        hitbox_components: &mut HashMap<String, Hitbox>,
        camera_target_components: &mut HashMap<String, CameraTarget>,
        storage_components: &mut HashMap<String, Storage>,
    ) {
        let player = "player".to_string();
        entities.push(player.clone());

        let player_graphics = Graphics3D {
            model_id: "character".to_string(),
        };
        graphics_3d_components.insert(player.clone(), player_graphics);

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
    }

    pub fn get_graphics(&self, entity: &Entity) -> Option<&Graphics3D> {
        self.graphics_3d_components.get(entity)
    }

    #[allow(dead_code)]
    pub fn get_graphics_inventory(&self, entity: &Entity) -> Option<&Graphics2D> {
        self.graphics_2d_components.get(entity)
    }

    pub fn create_position(&mut self, entity: Entity, position: Position) {
        self.position_components.insert(entity, position);
    }

    pub fn get_position(&self, entity: &Entity) -> Option<&Position> {
        self.position_components.get(entity)
    }

    pub fn get_position_mut(&mut self, entity: &Entity) -> Option<&mut Position> {
        self.position_components.get_mut(entity)
    }

    pub fn remove_position(&mut self, to_remove: &Entity) {
        self.position_components.remove(to_remove);
    }

    pub fn get_hitbox(&self, entity: &Entity) -> Option<&Hitbox> {
        self.hitbox_components.get(entity)
    }

    pub fn get_camera(&self, entity: &Entity) -> Option<&CameraTarget> {
        self.camera_target_components.get(entity)
    }

    pub fn get_camera_mut(&mut self, entity: &Entity) -> Option<&mut CameraTarget> {
        self.camera_target_components.get_mut(entity)
    }

    pub fn get_storage(&self, entity: &Entity) -> Option<&Storage> {
        self.storage_components.get(entity)
    }

    pub fn create_in_storage(&mut self, storage_entity: &Entity, to_store: Entity, spot: (u8, u8)) {
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

    pub fn get_in_storages(&self, storage_entity: &Entity) -> HashMap<&Entity, &InStorage> {
        self.in_storage_components
            .iter()
            .filter(|(_, in_storage)| in_storage.storage_entity == *storage_entity)
            .collect()
    }
}
