use crate::render::model::Mesh;
use crate::resources;
use cgmath::Vector3;
use std::collections::HashMap;
use wgpu::Device;

pub struct ModelManager {
    meshes: HashMap<String, Mesh>,
}

impl ModelManager {
    pub fn get_mesh(&self, mesh_id: String) -> &Mesh {
        self.meshes.get(&mesh_id).unwrap()
    }
}

impl ModelManager {
    pub async fn new(device: &Device) -> ModelManager {
        ModelManager {
            meshes: Self::load_models(device).await,
        }
    }

    async fn load_models(device: &Device) -> HashMap<String, Mesh> {
        let mut mesh_map: HashMap<String, Mesh> = HashMap::new();
        let black =
            resources::load_colored_square_model(device, Vector3::new(0.0, 0.0, 0.0)).unwrap();
        mesh_map.insert(
            "black".to_string(),
            black.meshes.into_iter().next().unwrap(),
        );

        let grey =
            resources::load_colored_square_model(device, Vector3::new(0.2, 0.2, 0.2)).unwrap();
        mesh_map.insert("grey".to_string(), grey.meshes.into_iter().next().unwrap());

        let shield = resources::load_model(device, "CUBE", "shield")
            .await
            .unwrap();
        mesh_map.insert(
            "shield".to_string(),
            shield.meshes.into_iter().next().unwrap(),
        );

        let shield_inventory = resources::load_model(device, "SQUARE", "shield")
            .await
            .unwrap();
        mesh_map.insert(
            "shield_inventory".to_string(),
            shield_inventory.meshes.into_iter().next().unwrap(),
        );

        let sword = resources::load_model(device, "CUBE", "sword")
            .await
            .unwrap();
        mesh_map.insert(
            "sword".to_string(),
            sword.meshes.into_iter().next().unwrap(),
        );

        let sword_inventory = resources::load_model(device, "SQUARE", "sword")
            .await
            .unwrap();
        mesh_map.insert(
            "sword_inventory".to_string(),
            sword_inventory.meshes.into_iter().next().unwrap(),
        );

        let grass = resources::load_model(device, "CUBE", "grass")
            .await
            .unwrap();
        mesh_map.insert(
            "grass".to_string(),
            grass.meshes.into_iter().next().unwrap(),
        );

        let tree = resources::load_model(device, "CUBE", "tree").await.unwrap();
        mesh_map.insert("tree".to_string(), tree.meshes.into_iter().next().unwrap());

        let gozer = resources::load_gltf(device, "gozer.gltf").await;
        mesh_map.insert(
            "gozer".to_string(),
            gozer.meshes.into_iter().next().unwrap(),
        );

        mesh_map
    }
}
