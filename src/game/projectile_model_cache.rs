use crate::game::md3::MD3Model;
use std::collections::HashMap;

pub enum ProjectileModelType {
    Rocket,
    Grenade,
}

pub struct ProjectileModelCache {
    models: HashMap<String, MD3Model>,
    textures: HashMap<String, macroquad::prelude::Texture2D>,
}

impl ProjectileModelCache {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn get_or_load_model(&mut self, model_type: ProjectileModelType) -> Option<&MD3Model> {
        let model_path = match model_type {
            ProjectileModelType::Rocket => "q3-resources/models/ammo/rocket/rocket.md3",
            ProjectileModelType::Grenade => "q3-resources/models/ammo/grenade1.md3",
        };

        if !self.models.contains_key(model_path) {
            if let Ok(model) = MD3Model::load(model_path) {
                self.models.insert(model_path.to_string(), model);
            } else {
                return None;
            }
        }

        self.models.get(model_path)
    }

    pub fn get_model(&self, model_type: ProjectileModelType) -> Option<&MD3Model> {
        let model_path = match model_type {
            ProjectileModelType::Rocket => "q3-resources/models/ammo/rocket/rocket.md3",
            ProjectileModelType::Grenade => "q3-resources/models/ammo/grenade1.md3",
        };
        self.models.get(model_path)
    }

    pub fn get_or_load_texture(&mut self, path: &str) -> Option<&macroquad::prelude::Texture2D> {
        if !self.textures.contains_key(path) {
            if std::path::Path::new(path).exists() {
                if let Ok(bytes) = std::fs::read(path) {
                    let texture =
                        macroquad::prelude::Texture2D::from_file_with_format(&bytes, None);
                    self.textures.insert(path.to_string(), texture);
                }
            }
        }

        self.textures.get(path)
    }

    pub fn get_texture(&self, path: &str) -> Option<&macroquad::prelude::Texture2D> {
        self.textures.get(path)
    }
}
