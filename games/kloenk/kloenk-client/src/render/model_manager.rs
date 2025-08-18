use crate::render::model::{ColorDefinition, ModelDefinition, PrimitiveDefinition};
use crate::render::model_loader::ModelLoader;
use cgmath::Vector4;
use std::collections::{HashMap, HashSet};

// TODO what if we just put vertices/textures/color in here such that model manager keeps track if model is ready or not?
// TODO hmm but what about callbacks on event loop
pub struct ModelManager {
    active_models: HashMap<String, ModelDefinition>, // These are the models that should be loaded at this point in time. The loaded models are the actual usable models
    loaded_models: HashMap<String, ModelDefinition>,
    loaded_vertices: HashSet<String>,
    loaded_textures: HashSet<String>,
    loaded_colors: HashSet<String>,
}

impl ModelManager {
    pub fn get_active_models(&self) -> &HashMap<String, ModelDefinition> {
        &self.active_models
    }

    pub fn get_model_3d(&self, model_id: &str) -> &ModelDefinition {
        self.loaded_models.get(model_id).or(self.loaded_models
            .get("default_model_3d")).expect("Default models should exist")
    }

    pub fn get_model_2d(&self, model_id: &str) -> &ModelDefinition {
        self.loaded_models.get(model_id).or(self.loaded_models
            .get("default_model_2d")).expect("Default models should exist")
    }

    pub fn add_active_model(&mut self, model: ModelDefinition) {
        self.active_models.insert(model.id.clone(), model);
    }

    pub fn added_vertices(&mut self, vertices_id: &str) {
        self.loaded_vertices.insert(vertices_id.to_owned());
        self.update_ready();
    }

    pub fn added_texture(&mut self, texture_id: &str) {
        self.loaded_textures.insert(texture_id.to_owned());
        self.update_ready();
    }

    pub fn added_color(&mut self, color_id: &str) {
        self.loaded_colors.insert(color_id.to_owned());
        self.update_ready();
    }

    // TODO maybe maximise usage/Don't call this method too often. gonna explode if 3000 models are being loaded and each is gonna check all models
    // could also make a reverse hashmap: look up all model ids for a texture and only update those
    fn update_ready(&mut self) {
        for model_id in self.active_models.keys() {
            if self.loaded_models.contains_key(model_id) {
                continue;
            }

            let model = self.active_models.get(model_id).unwrap();

            if self.primitives_loaded(model) {
                self.loaded_models.insert(model.id.clone(), model.clone());
            }
        }
    }

    fn primitives_loaded(&self, model_definition: &ModelDefinition) -> bool {
        for primitive in &model_definition.primitives {
            if !self.loaded_vertices.contains(&primitive.vertices_id) {
                return false;
            }

            if primitive.texture_definition.is_some()
                && !self
                .loaded_textures
                .contains(&primitive.texture_definition.clone().unwrap().id)
            {
                return false;
            }

            if !self.loaded_colors.contains(&primitive.color_definition.id) {
                return false;
            }
        }
        true
    }

    // TODO put cube/square/white1x1/white in?
    pub async fn new() -> ModelManager {
        let mut loaded_vertices = HashSet::new();
        loaded_vertices.insert("SQUARE".to_owned());
        loaded_vertices.insert("CUBE".to_owned());

        // let loaded_textures = HashSet::new();
        // loaded_textures.insert("white")
        // TODO white is just none right? get will return it anyway?

        let mut model_manager = ModelManager {
            active_models: HashMap::new(),
            loaded_models: HashMap::new(),
            loaded_vertices,
            loaded_textures: HashSet::new(),
            loaded_colors: HashSet::new(),
        };

        // TODO hm should we return different model based on ui model / 3d?
        let default_model_2d = ModelDefinition {
            id: "default_model_2d".to_owned(),
            primitives: vec![PrimitiveDefinition {
                vertices_id: "SQUARE".to_owned(),
                color_definition: ColorDefinition {
                    id: "white".to_owned(),
                    value: Vector4::new(1.0, 1.0, 1.0, 1.0),
                },
                texture_definition: None,
            }],
        };
        model_manager.add_active_model(default_model_2d);
        let default_model_3d = ModelDefinition {
            id: "default_model_3d".to_owned(),
            primitives: vec![PrimitiveDefinition {
                vertices_id: "CUBE".to_owned(),
                color_definition: ColorDefinition {
                    id: "white".to_owned(),
                    value: Vector4::new(1.0, 1.0, 1.0, 1.0),
                },
                texture_definition: None,
            }],
        };
        model_manager.add_active_model(default_model_3d);

        model_manager.add_active_model(ModelLoader::load_colored_square_model(
            "black",
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ));

        model_manager.add_active_model(ModelLoader::load_colored_square_model(
            "grey",
            Vector4::new(0.2, 0.2, 0.2, 1.0),
        ));

        // #780606
        model_manager.add_active_model(ModelLoader::load_colored_square_model(
            "blood_red",
            Vector4::new(0.46875, 0.0234375, 0.0234375, 1.0),
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "shield",
            "CUBE",
            "shield.dds",
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "shield_inventory",
            "SQUARE",
            "shield.dds",
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "sword",
            "CUBE",
            "sword.dds",
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "sword_inventory",
            "SQUARE",
            "sword.dds",
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "grass",
            "CUBE",
            "grass.dds",
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "tree",
            "CUBE",
            "tree.dds",
        ));

        // TODO let's just say we need material gozer.dds and maybe gozer2.dds for this. how would we load this?

        let gozer_models = ModelLoader::preload_gltf("gozer.gltf").await;
        for model in gozer_models {
            model_manager.add_active_model(model);
        }

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "close_button",
            "SQUARE",
            "close_button.dds",
        ));

        model_manager.add_active_model(ModelLoader::make_preload_model(
            "close_button_hover",
            "SQUARE",
            "close_button_hover.dds",
        ));

        model_manager
    }
}
