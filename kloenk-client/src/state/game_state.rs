use cgmath::num_traits::ToPrimitive;

use crate::render::camera::Camera;
use crate::state::components::{
    CameraTarget, Description, Dialogue, Entity, Graphics2D, Graphics3D, Health, Hitbox, InStorage,
    ItemShape, Rotation, Size, Storable, Storage,
};
use cgmath::{ElementWise, Point3};
use std::collections::{HashMap, HashSet};

pub const ROUGHLY_CAMERA_DISTANCE: f32 = 200_000.;

pub struct GameState {
    pub entities: Vec<Entity>,
    pub graphics_3d_components: HashMap<Entity, Graphics3D>,
    pub graphics_2d_components: HashMap<Entity, Graphics2D>,
    pub position_components: HashMap<Entity, Point3<f32>>,
    pub surface_components: HashSet<Entity>,
    pub size_components: HashMap<Entity, Size>,
    pub rotation_components: HashMap<Entity, Rotation>,
    pub hitbox_components: HashMap<Entity, Hitbox>,
    pub health_components: HashMap<Entity, Health>,
    pub camera_components: HashMap<Entity, Camera>,
    pub camera_target_components: HashMap<Entity, CameraTarget>,
    pub storable_components: HashMap<Entity, Storable>,
    pub storage_components: HashMap<Entity, Storage>,
    pub in_storage_components: HashMap<Entity, InStorage>,
    pub description_components: HashMap<Entity, Description>,
    pub dialogue_components: HashMap<Entity, Dialogue>,
}

impl GameState {}

impl GameState {
    pub fn new() -> Self {
        let mut entities = Vec::new();
        let mut graphics_3d_components = HashMap::new();
        let mut graphics_2d_components = HashMap::new();
        let mut position_components = HashMap::new();
        let mut surface_components = HashSet::new();
        let mut size_components = HashMap::new();
        let mut rotation_components = HashMap::new();
        let mut hitbox_components = HashMap::new();
        let mut health_components = HashMap::new();
        let mut camera_components = HashMap::new();
        let mut camera_target_components = HashMap::new();
        let mut storable_components = HashMap::new();
        let mut storage_components = HashMap::new();
        let in_storage_components = HashMap::new();
        let mut description_components = HashMap::new();
        let mut dialogue_components = HashMap::new();

        Self::load_player(
            &mut entities,
            &mut graphics_3d_components,
            &mut position_components,
            &mut rotation_components,
            &mut hitbox_components,
            &mut health_components,
            &mut camera_target_components,
            &mut storage_components,
            &mut description_components,
        );
        Self::load_npc(
            &mut entities,
            &mut graphics_3d_components,
            &mut position_components,
            &mut hitbox_components,
            &mut description_components,
            &mut dialogue_components,
        );
        Self::load_camera(&mut entities, &mut camera_components);
        Self::load_camera_ui(&mut entities, &mut camera_components);

        Self::load_shield(
            &mut entities,
            &mut graphics_3d_components,
            &mut graphics_2d_components,
            &mut position_components,
            &mut size_components,
            &mut hitbox_components,
            &mut storable_components,
            &mut description_components,
        );
        Self::load_swords(
            &mut entities,
            &mut graphics_3d_components,
            &mut graphics_2d_components,
            &mut position_components,
            &mut size_components,
            &mut hitbox_components,
            &mut storable_components,
            &mut description_components,
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
            &mut description_components,
        );

        Self {
            entities,
            graphics_3d_components,
            graphics_2d_components,
            position_components,
            surface_components,
            size_components,
            rotation_components,
            hitbox_components,
            health_components,
            camera_components,
            camera_target_components,
            storable_components,
            storage_components,
            in_storage_components,
            description_components,
            dialogue_components,
        }
    }

    fn load_tree(
        entities: &mut Vec<Entity>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        position_components: &mut HashMap<String, Point3<f32>>,
        hitbox_components: &mut HashMap<String, Hitbox>,
        description_components: &mut HashMap<String, Description>,
    ) {
        let tree = "tree".to_string();
        entities.push(tree.clone());

        let tree_graphics = Graphics3D {
            mesh_id: "tree".to_string(),
        };
        graphics_3d_components.insert(tree.clone(), tree_graphics);

        let tree_position = Point3 {
            x: 2.0,
            y: 1.0,
            z: -3.0,
        };

        position_components.insert(tree.clone(), tree_position);

        let tree_hitbox_min = tree_position.sub_element_wise(Point3::new(0.51, 0.51, 0.51));
        let tree_hitbox_max = tree_position.add_element_wise(Point3::new(0.51, 0.51, 0.51));
        let tree_hitbox = Hitbox {
            box_corner_min: tree_hitbox_min,
            box_corner_max: tree_hitbox_max,
        };
        hitbox_components.insert(tree.clone(), tree_hitbox);

        description_components.insert(
            tree.clone(),
            Description {
                text: "Tree of life".to_string(),
            },
        );
    }

    fn load_tiles(
        entities: &mut Vec<Entity>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        position_components: &mut HashMap<String, Point3<f32>>,
        surface_components: &mut HashSet<String>,
    ) {
        let plane_longitude_minimum: i8 = -10;
        let plane_longitude_maximum: i8 = 10;
        let plane_latitude_minimum: i8 = -10;
        let plane_latitude_maximum: i8 = 10;
        for x in plane_longitude_minimum..plane_longitude_maximum {
            for z in plane_latitude_minimum..plane_latitude_maximum {
                let plane = format!("plane{x}{z}"); //todo copy?
                entities.push(plane.clone());

                let plane_graphics = Graphics3D {
                    mesh_id: "grass".to_string(),
                };
                graphics_3d_components.insert(plane.clone(), plane_graphics);

                let plane_position = Point3 {
                    x: f32::from(x),
                    y: 0.0,
                    z: f32::from(z),
                };
                position_components.insert(plane.clone(), plane_position);

                surface_components.insert(plane.clone());
            }
        }
    }

    fn load_swords(
        entities: &mut Vec<Entity>,
        graphics_3d_components: &mut HashMap<String, Graphics3D>,
        graphics_2d_components: &mut HashMap<String, Graphics2D>,
        position_components: &mut HashMap<String, Point3<f32>>,
        size_components: &mut HashMap<String, Size>,
        hitbox_components: &mut HashMap<String, Hitbox>,
        storable_components: &mut HashMap<String, Storable>,
        description_components: &mut HashMap<String, Description>,
    ) {
        for i in 1..71 {
            let sword = "sword".to_string() + &i.to_string();
            entities.push(sword.clone());

            let sword_graphics = Graphics3D {
                mesh_id: "sword".to_string(),
            };
            graphics_3d_components.insert(sword.clone(), sword_graphics);

            let sword_graphics_inventory = Graphics2D {
                material_id: "sword_inventory".to_string(),
            };
            graphics_2d_components.insert(sword.clone(), sword_graphics_inventory);

            let sword_position = Point3 {
                x: i.to_f32().unwrap() + 0.1,
                y: 0.75,
                z: i.to_f32().unwrap() + 0.1,
            };
            position_components.insert(sword.clone(), sword_position);

            let size = Size {
                scale_x: 0.5,
                scale_y: 0.5,
                scale_z: 0.5,
            };
            size_components.insert(sword.clone(), size);

            let sword_hitbox_min = sword_position.sub_element_wise(Point3::new(0.26, 0.26, 0.26));
            let sword_hitbox_max = sword_position.add_element_wise(Point3::new(0.26, 0.26, 0.26));
            let sword_hitbox = Hitbox {
                box_corner_min: sword_hitbox_min,
                box_corner_max: sword_hitbox_max,
            };
            hitbox_components.insert(sword.clone(), sword_hitbox);

            let sword_storable = Storable {
                shape: ItemShape {
                    width: 1,
                    height: 1,
                },
            };
            storable_components.insert(sword.clone(), sword_storable);
            description_components.insert(
                sword.clone(),
                Description {
                    text: "Sword of Tungstenator".to_string(),
                },
            );
        }
    }

    fn load_shield(
        entities: &mut Vec<Entity>,
        graphics_3d_components: &mut HashMap<Entity, Graphics3D>,
        graphics_2d_components: &mut HashMap<Entity, Graphics2D>,
        position_components: &mut HashMap<Entity, Point3<f32>>,
        size_components: &mut HashMap<Entity, Size>,
        hitbox_components: &mut HashMap<Entity, Hitbox>,
        storable_components: &mut HashMap<Entity, Storable>,
        description_components: &mut HashMap<String, Description>,
    ) {
        let shield = "shield".to_string();
        entities.push(shield.clone());
        let shield_graphics = Graphics3D {
            mesh_id: "shield".to_string(),
        };
        graphics_3d_components.insert(shield.clone(), shield_graphics);

        let shield_graphics_inventory = Graphics2D {
            material_id: "shield_inventory".to_string(),
        };
        graphics_2d_components.insert(shield.clone(), shield_graphics_inventory);

        let shield_position = Point3 {
            x: -2.8,
            y: 0.75,
            z: -2.7,
        };
        position_components.insert(shield.clone(), shield_position);

        let size = Size {
            scale_x: 0.5,
            scale_y: 0.5,
            scale_z: 0.5,
        };
        size_components.insert(shield.clone(), size);

        let shield_hitbox_min = shield_position.sub_element_wise(Point3::new(0.26, 0.26, 0.26));
        let shield_hitbox_max = shield_position.add_element_wise(Point3::new(0.26, 0.26, 0.26));
        let shield_hitbox = Hitbox {
            box_corner_min: shield_hitbox_min,
            box_corner_max: shield_hitbox_max,
        };
        hitbox_components.insert(shield.clone(), shield_hitbox);

        let shield_storable = Storable {
            shape: ItemShape {
                width: 1,
                height: 2,
            },
        };
        storable_components.insert(shield.clone(), shield_storable);
        description_components.insert(
            shield.clone(),
            Description {
                text: "Shield of Hydrogax".to_string(),
            },
        );
    }

    fn load_player(
        entities: &mut Vec<Entity>,
        graphics_3d_components: &mut HashMap<Entity, Graphics3D>,
        position_components: &mut HashMap<Entity, Point3<f32>>,
        rotation_components: &mut HashMap<Entity, Rotation>,
        hitbox_components: &mut HashMap<Entity, Hitbox>,
        health_components: &mut HashMap<Entity, Health>,
        camera_target_components: &mut HashMap<Entity, CameraTarget>,
        storage_components: &mut HashMap<Entity, Storage>,
        description_components: &mut HashMap<String, Description>,
    ) {
        let player = "player".to_string();
        entities.push(player.clone());

        let player_graphics = Graphics3D {
            mesh_id: "gozer".to_string(),
        };
        graphics_3d_components.insert(player.clone(), player_graphics);

        let player_position = Point3 {
            x: 0.0,
            y: 0.5,
            z: 0.0,
        };
        position_components.insert(player.clone(), player_position);

        let rotation = Rotation { degrees_y: 50.0 };
        rotation_components.insert(player.clone(), rotation);

        let player_hitbox_min = player_position.sub_element_wise(Point3::new(0.1, 0.0, 0.1));
        let player_hitbox_max = player_position.add_element_wise(Point3::new(0.1, 1.8, 0.1));
        let player_hitbox = Hitbox {
            box_corner_min: player_hitbox_min,
            box_corner_max: player_hitbox_max,
        };
        hitbox_components.insert(player.clone(), player_hitbox);

        health_components.insert(
            player.clone(),
            Health {
                hitpoints: 100,
                max_hitpoints: 100,
            },
        );

        let camera_target = CameraTarget {
            distance: f32::sqrt(ROUGHLY_CAMERA_DISTANCE / 3.0),
            rotation_x_degrees: 225.0,
            rotation_y_degrees: 315.0,
        };
        camera_target_components.insert(player.clone(), camera_target);

        let player_storage = Storage {
            number_of_rows: 8,
            number_of_columns: 8,
        };
        storage_components.insert(player.clone(), player_storage);
        description_components.insert(
            player.clone(),
            Description {
                text: "That's me!".to_string(),
            },
        );
    }

    fn load_npc(
        entities: &mut Vec<Entity>,
        graphics_3d_components: &mut HashMap<Entity, Graphics3D>,
        position_components: &mut HashMap<Entity, Point3<f32>>,
        hitbox_components: &mut HashMap<Entity, Hitbox>,
        description_components: &mut HashMap<String, Description>,
        dialogue_components: &mut HashMap<String, Dialogue>,
    ) {
        let npc = "Dennis".to_string();
        entities.push(npc.clone());

        let player_graphics = Graphics3D {
            mesh_id: "gozer".to_string(),
        };
        graphics_3d_components.insert(npc.clone(), player_graphics);

        let player_position = Point3 {
            x: -3.0,
            y: 0.5,
            z: 2.0,
        };
        position_components.insert(npc.clone(), player_position);

        let npc_hitbox_min = player_position.sub_element_wise(Point3::new(0.1, 0.0, 0.1));
        let npc_hitbox_max = player_position.add_element_wise(Point3::new(0.1, 1.8, 0.1));
        let npc_hitbox = Hitbox {
            box_corner_min: npc_hitbox_min,
            box_corner_max: npc_hitbox_max,
        };
        hitbox_components.insert(npc.clone(), npc_hitbox);

        description_components.insert(
            npc.clone(),
            Description {
                text: "Dennis is a menace.".to_string(),
            },
        );

        dialogue_components.insert(
            npc.clone(),
            Dialogue {
                dialogue_id: "dennis_intro".to_string(),
            },
        );
    }

    fn load_camera(entities: &mut Vec<Entity>, camera_components: &mut HashMap<String, Camera>) {
        let camera = "camera".to_string();
        let camera_component = Camera::new();
        entities.push(camera.clone());
        camera_components.insert(camera.clone(), camera_component);
    }

    fn load_camera_ui(entities: &mut Vec<Entity>, camera_components: &mut HashMap<String, Camera>) {
        let camera = "camera_ui".to_string();
        let mut camera_component = Camera::new();
        camera_component.eye = Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        camera_component.target = Point3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        };

        camera_component.z_near = -1.0;
        camera_component.z_far = 1.0;
        entities.push(camera.clone());
        camera_components.insert(camera.clone(), camera_component);
    }

    pub fn get_graphics(&self, entity: &Entity) -> Option<&Graphics3D> {
        self.graphics_3d_components.get(entity)
    }

    #[allow(dead_code)]
    pub fn get_graphics_inventory(&self, entity: &Entity) -> Option<&Graphics2D> {
        self.graphics_2d_components.get(entity)
    }

    // Note: On a position change, also consider updating the hitbox
    pub fn create_position(&mut self, entity: Entity, position: Point3<f32>) {
        self.position_components.insert(entity, position);
    }

    pub fn get_position(&self, entity: &Entity) -> Option<&Point3<f32>> {
        self.position_components.get(entity)
    }

    #[allow(dead_code)]
    pub fn get_position_mut(&mut self, entity: &Entity) -> Option<&mut Point3<f32>> {
        self.position_components.get_mut(entity)
    }

    // Note: On a position change, also consider updating the hitbox
    pub fn remove_position(&mut self, to_remove: &Entity) {
        self.position_components.remove(to_remove);
    }

    pub fn get_size(&self, entity: &Entity) -> Option<&Size> {
        self.size_components.get(entity)
    }

    pub fn get_rotation(&self, entity: &Entity) -> Option<&Rotation> {
        self.rotation_components.get(entity)
    }

    pub fn create_hitbox(&mut self, entity: Entity, hitbox: Hitbox) {
        self.hitbox_components.insert(entity, hitbox);
    }

    pub fn get_hitbox(&self, entity: &Entity) -> Option<&Hitbox> {
        self.hitbox_components.get(entity)
    }

    pub fn remove_hitbox(&mut self, to_remove: &Entity) {
        self.hitbox_components.remove(to_remove);
    }

    pub fn get_camera_target(&self, entity: &Entity) -> Option<&CameraTarget> {
        self.camera_target_components.get(entity)
    }

    pub fn get_camera_target_mut(&mut self, entity: &Entity) -> Option<&mut CameraTarget> {
        self.camera_target_components.get_mut(entity)
    }

    #[allow(dead_code)]
    pub fn get_camera(&self, entity: &str) -> Option<&Camera> {
        self.camera_components.get(entity)
    }

    pub fn get_camera_mut(&mut self, entity: &str) -> Option<&mut Camera> {
        self.camera_components.get_mut(entity)
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

    #[allow(dead_code)]
    pub fn get_in_storages(&self, storage_entity: &Entity) -> HashMap<&Entity, &InStorage> {
        self.in_storage_components
            .iter()
            .filter(|(_, in_storage)| in_storage.storage_entity == *storage_entity)
            .collect()
    }
}
