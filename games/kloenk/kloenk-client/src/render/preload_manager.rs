use crate::render::model::ModelDefinition;
use crate::render::model_loader::ModelLoader;
use cgmath::Vector4;
use std::vec::Drain;

// or like AssetLoadTask / AssetLoadCommand
// Maybe also in charge of unloading? Just making sure the current state of the world is loaded correctly
pub struct PreloadManager {
    models_to_load: Vec<ModelDefinition>,
}

impl PreloadManager {
    pub async fn new() -> PreloadManager {
        PreloadManager {
            models_to_load: Self::preload_models().await,
        }
    }

    pub fn drain_models_to_load(&mut self) -> Drain<'_, ModelDefinition> {
        self.models_to_load.drain(..)
    }

    // TODO maybe ModelDefinition instead? reuse preload with ids+source
    async fn preload_models() -> Vec<ModelDefinition> {
        let mut models_to_load: Vec<ModelDefinition> = Vec::new();
        models_to_load.push(ModelLoader::load_colored_square_model(
            "black".to_string(),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ));

        models_to_load.push(ModelLoader::load_colored_square_model(
            "grey".to_string(),
            Vector4::new(0.2, 0.2, 0.2, 1.0),
        ));

        // #780606
        models_to_load.push(ModelLoader::load_colored_square_model(
            "blood_red".to_string(),
            Vector4::new(0.46875, 0.0234375, 0.0234375, 1.0),
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "shield".to_string(),
            "CUBE",
            "shield.dds",
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "shield_inventory".to_string(),
            "SQUARE",
            "shield.dds",
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "sword".to_string(),
            "CUBE",
            "sword.dds",
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "sword_inventory".to_string(),
            "SQUARE",
            "sword.dds",
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "grass".to_string(),
            "CUBE",
            "grass.dds",
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "tree".to_string(),
            "CUBE",
            "tree.dds",
        ));

        // TODO let's just say we need material gozer.dds and maybe gozer2.dds for this. how would we load this?
        let gozer_models = ModelLoader::preload_gltf("gozer.gltf").await;
        for model in gozer_models {
            models_to_load.push(model);
        }

        models_to_load.push(ModelLoader::make_preload_model(
            "close_button".to_string(),
            "SQUARE",
            "close_button.dds",
        ));

        models_to_load.push(ModelLoader::make_preload_model(
            "close_button_hover".to_string(),
            "SQUARE",
            "close_button_hover.dds",
        ));

        models_to_load
    }
}
