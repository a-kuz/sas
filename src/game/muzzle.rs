use crate::game::md3::MD3Model;
use crate::game::weapon::Weapon;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct MuzzleFlashCache {
    models: HashMap<Weapon, Option<MD3Model>>,
    textures: HashMap<Weapon, Option<Texture2D>>,
    texture_paths: HashMap<Weapon, String>,
}

impl MuzzleFlashCache {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            textures: HashMap::new(),
            texture_paths: HashMap::new(),
        }
    }

    pub async fn load_all(&mut self) {
        self.load_flash_model(
            Weapon::MachineGun,
            "q3-resources/models/weapons2/machinegun/machinegun_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::MachineGun,
            "q3-resources/models/weapons2/machinegun/f_machinegun.png",
        )
        .await;

        self.load_flash_model(
            Weapon::Shotgun,
            "q3-resources/models/weapons2/shotgun/shotgun_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::Shotgun,
            "q3-resources/models/weapons2/shotgun/f_shotgun.png",
        )
        .await;

        self.load_flash_model(
            Weapon::GrenadeLauncher,
            "q3-resources/models/weapons2/grenadel/grenadel_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::GrenadeLauncher,
            "q3-resources/models/weapons2/grenadel/f_grenadel.png",
        )
        .await;

        self.load_flash_model(
            Weapon::RocketLauncher,
            "q3-resources/models/weapons2/rocketl/rocketl_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::RocketLauncher,
            "q3-resources/models/weapons2/rocketl/f_rocketl.png",
        )
        .await;

        self.load_flash_model(
            Weapon::Lightning,
            "q3-resources/models/weapons2/lightning/lightning_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::Lightning,
            "q3-resources/models/weapons2/lightning/f_lightning.png",
        )
        .await;

        self.load_flash_model(
            Weapon::Railgun,
            "q3-resources/models/weapons2/railgun/railgun_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::Railgun,
            "q3-resources/models/weapons2/railgun/f_railgun.png",
        )
        .await;

        self.load_flash_model(
            Weapon::Plasmagun,
            "q3-resources/models/weapons2/plasma/plasma_flash.md3",
        )
        .await;
        self.load_flash_texture(
            Weapon::Plasmagun,
            "q3-resources/models/weapons2/plasma/f_plasma.png",
        )
        .await;

        self.load_flash_model(
            Weapon::BFG,
            "q3-resources/models/weapons2/bfg/bfg_flash.md3",
        )
        .await;
        self.load_flash_texture(Weapon::BFG, "q3-resources/models/weapons2/bfg/f_bfg.png")
            .await;
    }

    async fn load_flash_texture(&mut self, weapon: Weapon, path: &str) {
        self.texture_paths.insert(weapon, path.to_string());
        if let Ok(tex) = load_texture(path).await {
            tex.set_filter(FilterMode::Linear);
            self.textures.insert(weapon, Some(tex));
        } else {
            self.textures.insert(weapon, None);
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

    pub fn get_texture(&self, weapon: Weapon) -> Option<&Texture2D> {
        self.textures.get(&weapon).and_then(|opt| opt.as_ref())
    }

    pub fn get_texture_path(&self, weapon: Weapon) -> Option<&str> {
        self.texture_paths.get(&weapon).map(|s| s.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct MuzzleFlash {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub roll: f32,
    pub life: u8,
    pub weapon: Weapon,
    pub birth_time: f64,
}

impl MuzzleFlash {
    pub fn new(x: f32, y: f32, angle: f32, weapon: Weapon) -> Self {
        let roll = crate::compat_rand::gen_range_f32(-10.0, 10.0);
        Self {
            x,
            y,
            angle,
            roll,
            life: 0,
            weapon,
            birth_time: crate::time::get_time(),
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
            let texture = cache.get_texture(self.weapon);
            let texture_path = cache.get_texture_path(self.weapon);
            for mesh in &model.meshes {
                super::md3_render::render_md3_mesh_rotated_with_additive(
                    mesh,
                    0,
                    screen_x + offset_x,
                    screen_y + offset_y,
                    1.2,
                    Color::from_rgba(255, 255, 255, (fade * 255.0) as u8),
                    texture,
                    texture_path,
                    self.roll.to_radians(),
                    true,
                );
            }
        }
    }
}
