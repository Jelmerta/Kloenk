use crate::render::model::{ColorDefinition, ModelDefinition, PrimitiveDefinition};
use cgmath::Vector4;
use std::collections::{HashMap, HashSet};
use std::string::ToString;

// TODO what if we just put vertices/textures/color in here such that model manager keeps track if model is ready or not?
// TODO hmm but what about callbacks on event loop
pub struct ModelManager {
    model_definitions: HashMap<String, ModelDefinition>,
    loaded_models: HashSet<String>,
    loaded_vertices: HashSet<String>,
    loaded_textures: HashSet<String>,
    loaded_colors: HashSet<String>,
}

impl ModelManager {
    pub fn get_model_3d(&self, model_id: String) -> &ModelDefinition {
        if self.loaded_models.contains(&model_id) {
            self.model_definitions.get(&model_id).unwrap()
        } else {
            self.model_definitions
                .get(&("default_model_3d".to_string()))
                .unwrap()
        }
    }

    pub fn get_model_2d(&self, model_id: String) -> &ModelDefinition {
        if self.is_ready(&model_id) {
            self.model_definitions.get(&model_id).unwrap()
        } else {
            self.model_definitions
                .get(&("default_model_2d".to_string()))
                .unwrap()
        }
    }

    pub fn add_model(&mut self, model: ModelDefinition) {
        self.model_definitions.insert(model.id.clone(), model);
    }

    pub fn is_ready(&self, model_id: &String) -> bool {
        self.loaded_models.contains(model_id)
    }

    pub fn added_vertices(&mut self, vertices_id: &String) {
        log::error!("added_vertices {}", vertices_id);
        self.loaded_vertices.insert(vertices_id.clone());
        self.update_ready();
    }

    pub fn added_texture(&mut self, texture_id: &String) {
        log::error!("added_texture {}", texture_id);
        self.loaded_textures.insert(texture_id.clone());
        self.update_ready();
    }

    pub fn added_color(&mut self, color_id: &String) {
        log::error!("added_color {}", color_id);
        self.loaded_colors.insert(color_id.clone());
        self.update_ready();
    }

    // TODO maybe maximise usage/Don't call this method too often. gonna explode if 3000 models are being loaded and each is gonna check all models
    // could also make a reverse hashmap: look up all model ids for a texture and only update those
    fn update_ready(&mut self) {
        for (model_id, _) in &self.model_definitions {
            if self.loaded_models.contains(model_id) {
                continue;
            }

            let model = self.model_definitions.get(model_id).unwrap();

            if self.primitives_loaded(model) {
                log::error!("Model loaded {}", model_id);
                self.loaded_models.insert(model.id.clone());
            }
        }
    }

    fn primitives_loaded(&self, model_definition: &ModelDefinition) -> bool {
        for primitive in model_definition.primitives.iter() {
            if !self.loaded_vertices.contains(&primitive.vertices_id) {
                return false; // TODO go to next model
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
        loaded_vertices.insert("SQUARE".to_string());
        loaded_vertices.insert("CUBE".to_string());

        // let loaded_textures = HashSet::new();
        // loaded_textures.insert("white")
        // TODO white is just none right? get will return it anyway?

        let mut model_manager = ModelManager {
            model_definitions: HashMap::new(),
            loaded_models: HashSet::new(),
            loaded_vertices,
            loaded_textures: HashSet::new(),
            loaded_colors: HashSet::new(),
        };

        // TODO hm should we return different model based on ui model / 3d?
        let default_model_2d = ModelDefinition {
            id: "default_model_2d".to_string(),
            primitives: vec![PrimitiveDefinition {
                vertices_id: "SQUARE".to_string(),
                color_definition: ColorDefinition {
                    id: "white".to_string(),
                    value: Vector4::new(1.0, 1.0, 1.0, 1.0),
                },
                texture_definition: None,
            }],
        };
        model_manager.add_model(default_model_2d);
        let default_model_3d = ModelDefinition {
            id: "default_model_3d".to_string(),
            primitives: vec![PrimitiveDefinition {
                vertices_id: "CUBE".to_string(),
                color_definition: ColorDefinition {
                    id: "white".to_string(),
                    value: Vector4::new(1.0, 1.0, 1.0, 1.0),
                },
                texture_definition: None,
            }],
        };
        model_manager.add_model(default_model_3d);
        model_manager
    }
}
