use crate::render::model::Mesh;
use crate::render::model_loader::{load_colored_square_model, load_gltf, load_model};
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
            load_colored_square_model(device, Vector3::new(0.0, 0.0, 0.0)).unwrap();
        mesh_map.insert(
            "black".to_string(),
            black.meshes.into_iter().next().unwrap(),
        );

        let grey =
            load_colored_square_model(device, Vector3::new(0.2, 0.2, 0.2)).unwrap();
        mesh_map.insert("grey".to_string(), grey.meshes.into_iter().next().unwrap());

        // #780606
        let blood_red = load_colored_square_model(
            device,
            Vector3::new(0.46875, 0.0234375, 0.0234375),
        )
            .unwrap();
        mesh_map.insert(
            "blood_red".to_string(),
            blood_red.meshes.into_iter().next().unwrap(),
        );

        let shield = load_model(device, "CUBE", "shield.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "shield".to_string(),
            shield.meshes.into_iter().next().unwrap(),
        );

        let shield_inventory = load_model(device, "SQUARE", "shield.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "shield_inventory".to_string(),
            shield_inventory.meshes.into_iter().next().unwrap(),
        );

        let sword = load_model(device, "CUBE", "sword.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "sword".to_string(),
            sword.meshes.into_iter().next().unwrap(),
        );

        let sword_inventory = load_model(device, "SQUARE", "sword.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "sword_inventory".to_string(),
            sword_inventory.meshes.into_iter().next().unwrap(),
        );

        let grass = load_model(device, "CUBE", "grass.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "grass".to_string(),
            grass.meshes.into_iter().next().unwrap(),
        );

        let tree = load_model(device, "CUBE", "tree.webp").await.unwrap();
        mesh_map.insert("tree".to_string(), tree.meshes.into_iter().next().unwrap());

        let gozer = load_gltf(device, "gozer.gltf").await;
        mesh_map.insert(
            "gozer".to_string(),
            gozer.meshes.into_iter().next().unwrap(),
        );

        let close_button = load_model(device, "SQUARE", "close_button.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "close_button".to_string(),
            close_button.meshes.into_iter().next().unwrap(),
        );

        let close_button_hover = load_model(device, "SQUARE", "close_button_hover.webp")
            .await
            .unwrap();
        mesh_map.insert(
            "close_button_hover".to_string(),
            close_button_hover.meshes.into_iter().next().unwrap(),
        );

        mesh_map
    }
}
