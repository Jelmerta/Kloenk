use crate::render::model::{ColorDefinition, ModelDefinition, PrimitiveDefinition};
use cgmath::Vector4;
use std::collections::HashMap;
use std::string::ToString;

pub struct ModelManager {
    loaded_models: HashMap<String, ModelDefinition>,
}

impl ModelManager {
    pub fn get_model_3d(&self, model_id: String) -> &ModelDefinition {
        self.loaded_models.get(&model_id).or_else(|| self.loaded_models.get(&("default_model_3d".to_string()))).unwrap()
    }

    pub fn get_model_2d(&self, model_id: String) -> &ModelDefinition {
        self.loaded_models.get(&model_id).or_else(|| self.loaded_models.get(&("default_model_2d".to_string()))).unwrap()
    }

    pub fn add_model(&mut self, model: ModelDefinition) {
        self.loaded_models.insert(model.id.clone(), model);
    }
}

impl ModelManager {
    pub async fn new() -> ModelManager {
        let mut model_manager = ModelManager {
            loaded_models: HashMap::new(),
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
