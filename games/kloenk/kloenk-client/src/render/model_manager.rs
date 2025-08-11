use crate::render::model::Model;
use std::collections::HashMap;

pub struct ModelManager {
    loaded_models: HashMap<String, Model>,
}

impl ModelManager {
    pub fn get_model(&self, model_id: String) -> &Model {
        self.loaded_models.get(&model_id).unwrap()
    }

    pub fn add_model(&mut self, model_id: String, model: Model) {
        self.loaded_models.insert(model_id, model);
    }
}

impl ModelManager {
    pub async fn new() -> ModelManager {
        ModelManager {
            loaded_models: HashMap::new(),
        }
    }
}
