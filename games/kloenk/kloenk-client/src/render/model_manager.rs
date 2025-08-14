use crate::render::model::ModelDefinition;
use std::collections::HashMap;

pub struct ModelManager {
    loaded_models: HashMap<String, ModelDefinition>,
}

impl ModelManager {
    pub fn get_model(&self, model_id: String) -> &ModelDefinition {
        self.loaded_models.get(&model_id).unwrap()
    }

    pub fn add_model(&mut self, model: ModelDefinition) {
        self.loaded_models.insert(model.id.clone(), model);
    }
}

impl ModelManager {
    pub async fn new() -> ModelManager {
        ModelManager {
            loaded_models: HashMap::new(),
        }
    }
}
