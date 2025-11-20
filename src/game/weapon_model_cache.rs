use crate::game::md3::MD3Model;
use crate::game::weapon::Weapon;
use crate::resource_path::get_resource_path;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct WeaponModel {
    pub models: Vec<MD3Model>,
    pub textures: HashMap<String, Texture2D>,
    pub weapon_type: Weapon,
}

pub struct WeaponModelCache {
    models: HashMap<Weapon, WeaponModel>,
}

impl WeaponModelCache {
    pub fn new() -> Self {
        Self { models: HashMap::new() }
    }

    fn weapon_path(weapon: Weapon) -> String {
        let relative = match weapon {
            Weapon::Gauntlet => "models/weapons2/gauntlet/gauntlet.md3",
            Weapon::MachineGun => "models/weapons2/machinegun/machinegun.md3",
            Weapon::Shotgun => "models/weapons2/shotgun/shotgun.md3",
            Weapon::GrenadeLauncher => "models/weapons2/grenadel/grenadel.md3",
            Weapon::RocketLauncher => "models/weapons2/rocketl/rocketl.md3",
            Weapon::Lightning => "models/weapons2/lightning/lightning.md3",
            Weapon::Railgun => "models/weapons2/railgun/railgun.md3",
            Weapon::Plasmagun => "models/weapons2/plasma/plasma.md3",
            Weapon::BFG => "models/weapons2/bfg/bfg.md3",
        };
        get_resource_path(relative)
    }
    
    fn extra_paths(weapon: Weapon) -> Vec<String> {
        let relatives: Vec<&str> = match weapon {
            Weapon::MachineGun => vec![
                "models/weapons2/machinegun/machinegun_barrel.md3",
            ],
            _ => Vec::new(),
        };
        relatives.iter().map(|p| get_resource_path(p)).collect()
    }

    fn weapon_textures(weapon: Weapon) -> Vec<&'static str> {
        match weapon {
            Weapon::Gauntlet => vec![
                "models/weapons2/gauntlet/gauntlet1.png",
                "models/weapons2/gauntlet/gauntlet3.png",
                "models/weapons2/gauntlet/gauntlet4.png",
            ],
            Weapon::MachineGun => vec![
                "models/weapons2/machinegun/machinegun.png",
            ],
            Weapon::Shotgun => vec![
                "models/weapons2/shotgun/shotgun.png",
            ],
            Weapon::GrenadeLauncher => vec![
                "models/weapons2/grenadel/grenadel.png",
            ],
            Weapon::RocketLauncher => vec![
                "models/weapons2/rocketl/rocketl.png",
                "models/weapons2/rocketl/rocketl2.png",
            ],
            Weapon::Lightning => vec![
                "models/weapons2/lightning/lightning2.png",
                "models/weapons2/lightning/button.png",
                "models/weapons2/lightning/glass.png",
            ],
            Weapon::Railgun => vec![
                "models/weapons2/railgun/railgun1.png",
                "models/weapons2/railgun/railgun3.png",
                "models/weapons2/railgun/railgun4.png",
            ],
            Weapon::Plasmagun => vec![
                "models/weapons2/plasma/plasma.png",
            ],
            Weapon::BFG => vec![
                "models/weapons2/bfg/f_bfg.png",
                "models/weapons2/bfg/f_bfg2.png",
            ],
        }
    }

    pub async fn preload(&mut self, weapon: Weapon) {
        if !self.models.contains_key(&weapon) {
            let main_model_result;
            
            #[cfg(target_arch = "wasm32")]
            {
                main_model_result = MD3Model::load_async(Self::weapon_path(weapon)).await;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                main_model_result = MD3Model::load(Self::weapon_path(weapon));
            }
            
            if let Ok(main_model) = main_model_result {
                let mut models = vec![main_model];
                
                for path in Self::extra_paths(weapon) {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Ok(m) = MD3Model::load_async(path).await {
                            models.push(m);
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Ok(m) = MD3Model::load(path) {
                            models.push(m);
                        }
                    }
                }

                let mut weapon_model = WeaponModel {
                    models,
                    textures: HashMap::new(),
                    weapon_type: weapon,
                };

                for texture_path in Self::weapon_textures(weapon) {
                    let full_path = get_resource_path(texture_path);
                    if let Some(texture) = super::skin_loader::load_texture_file(&full_path).await {
                        println!("[Weapon] ✓ Loaded texture {}", texture_path);

                        for model in &weapon_model.models {
                            for mesh in &model.meshes {
                                let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                                    .trim_end_matches('\0')
                                    .to_string();
                                if !weapon_model.textures.contains_key(&mesh_name) {
                                    weapon_model.textures.insert(mesh_name, texture.clone());
                                }
                            }
                        }
                    }
                }

                self.models.insert(weapon, weapon_model);
            }
        }
    }
    
    pub fn get(&self, weapon: Weapon) -> Option<&WeaponModel> {
        self.models.get(&weapon)
    }
}



