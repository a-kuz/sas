use crate::game::player_model::PlayerModel;
use std::collections::HashMap;

pub struct ModelCache {
    models: HashMap<String, PlayerModel>,
}

impl ModelCache {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn get_or_load(&mut self, model_name: &str) -> Option<&PlayerModel> {
        if !self.models.contains_key(model_name) {
            if let Ok(model) = PlayerModel::load(model_name) {
                self.models.insert(model_name.to_string(), model);
            } else {
                return None;
            }
        }

        self.models.get(model_name)
    }

    pub async fn get_or_load_async(&mut self, model_name: &str) -> Option<&PlayerModel> {
        if !self.models.contains_key(model_name) {
            if let Ok(model) = PlayerModel::load_async(model_name).await {
                self.models.insert(model_name.to_string(), model);
            } else {
                return None;
            }
        }

        self.models.get(model_name)
    }

    pub fn get(&self, model_name: &str) -> Option<&PlayerModel> {
        self.models.get(model_name)
    }

    pub fn get_mut(&mut self, model_name: &str) -> Option<&mut PlayerModel> {
        self.models.get_mut(model_name)
    }
}
