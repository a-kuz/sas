use macroquad::prelude::*;
use crate::game::weapon::Weapon;
use crate::game::md3::MD3Model;
use std::collections::HashMap;

pub struct MuzzleFlashCache {
    models: HashMap<Weapon, Option<MD3Model>>,
    textures: HashMap<Weapon, Option<Texture2D>>,
}

impl MuzzleFlashCache {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub async fn load_all(&mut self) {
        self.load_flash_model(Weapon::MachineGun, "q3-resources/models/weapons2/machinegun/machinegun_flash.md3").await;
        self.load_flash_model(Weapon::Shotgun, "q3-resources/models/weapons2/shotgun/shotgun_flash.md3").await;
        self.load_flash_model(Weapon::GrenadeLauncher, "q3-resources/models/weapons2/grenadel/grenadel_flash.md3").await;
        self.load_flash_model(Weapon::RocketLauncher, "q3-resources/models/weapons2/rocketl/rocketl_flash.md3").await;
        self.load_flash_model(Weapon::Lightning, "q3-resources/models/weapons2/lightning/lightning_flash.md3").await;
        self.load_flash_model(Weapon::Railgun, "q3-resources/models/weapons2/railgun/railgun_flash.md3").await;
        self.load_flash_model(Weapon::Plasmagun, "q3-resources/models/weapons2/plasma/plasma_flash.md3").await;
        self.load_flash_model(Weapon::BFG, "q3-resources/models/weapons2/bfg/bfg_flash.md3").await;
        
        if let Ok(tex) = load_texture("q3-resources/gfx/misc/flare.png").await {
            tex.set_filter(FilterMode::Linear);
            self.textures.insert(Weapon::MachineGun, Some(tex.clone()));
        }
    }

    async fn load_flash_model(&mut self, weapon: Weapon, path: &str) {
        if let Ok(model) = MD3Model::load(path) {
            self.models.insert(weapon, Some(model));
        } else {
            self.models.insert(weapon, None);
        }
    }

    pub fn get_model(&self, weapon: Weapon) -> Option<&MD3Model> {
        self.models.get(&weapon).and_then(|opt| opt.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct MuzzleFlash {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub life: u8,
    pub weapon: Weapon,
    pub birth_time: f64,
}

impl MuzzleFlash {
    pub fn new(x: f32, y: f32, angle: f32, weapon: Weapon) -> Self {
        Self {
            x,
            y,
            angle,
            life: 0,
            weapon,
            birth_time: get_time(),
        }
    }

    pub fn update(&mut self) -> bool {
        self.life += 1;
        self.life < 5
    }

    pub fn render(&self, camera_x: f32, camera_y: f32, cache: &MuzzleFlashCache) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let offset_x = self.angle.cos() * 20.0;
        let offset_y = self.angle.sin() * 20.0;
        
        let fade = 1.0 - (self.life as f32 / 5.0);
        
        if let Some(model) = cache.get_model(self.weapon) {
            for mesh in &model.meshes {
                super::md3_render::render_md3_mesh_rotated(
                    mesh,
                    0,
                    screen_x + offset_x,
                    screen_y + offset_y,
                    1.2,
                    Color::from_rgba(255, 255, 255, (fade * 255.0) as u8),
                    None,
                    self.angle + std::f32::consts::PI / 2.0,
                );
            }
        } else {
            let (color, size) = match self.weapon {
                Weapon::RocketLauncher => (Color::from_rgba(255, 140, 20, 255), 32.0),
                Weapon::Plasmagun => (Color::from_rgba(80, 180, 255, 255), 24.0),
                Weapon::BFG => (Color::from_rgba(80, 255, 80, 255), 40.0),
                Weapon::Railgun => (Color::from_rgba(180, 220, 255, 255), 28.0),
                Weapon::GrenadeLauncher => (Color::from_rgba(255, 180, 60, 255), 28.0),
                Weapon::Shotgun => (Color::from_rgba(255, 200, 100, 255), 24.0),
                _ => (Color::from_rgba(255, 220, 100, 255), 20.0),
            };
            
            let current_size = size * fade;
            draw_circle(
                screen_x + offset_x,
                screen_y + offset_y,
                current_size,
                color,
            );
        }
    }
}

