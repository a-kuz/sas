use crate::game::map::ItemType;
use crate::game::md3::MD3Model;
use macroquad::prelude::*;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemModelType {
    HealthSmall,
    HealthMedium,
    HealthLarge,
    HealthMega,

    ArmorShard,
    ArmorYellow,
    ArmorRed,

    WeaponGauntlet,
    WeaponShotgun,
    WeaponMachinegun,
    WeaponGrenadeLauncher,
    WeaponRocketLauncher,
    WeaponLightning,
    WeaponRailgun,
    WeaponPlasmagun,
    WeaponBFG,

    AmmoShells,
    AmmoBullets,
    AmmoGrenades,
    AmmoRockets,
    AmmoCells,
    AmmoLightning,
    AmmoSlugs,
    AmmoBfg,

    PowerupQuad,
    PowerupBattleSuit,
    PowerupHaste,
    PowerupInvis,
    PowerupRegen,
    PowerupFlight,

    HoldableTeleporter,
    HoldableMedkit,
}

impl ItemModelType {
    pub fn item_color(&self) -> Color {
        match self {
            ItemModelType::HealthSmall | ItemModelType::HealthMedium => {
                Color::from_rgba(0, 255, 0, 255)
            }
            ItemModelType::HealthLarge => Color::from_rgba(255, 255, 0, 255),
            ItemModelType::HealthMega => Color::from_rgba(0, 100, 255, 255),

            ItemModelType::ArmorShard => Color::from_rgba(200, 200, 200, 255),
            ItemModelType::ArmorYellow => Color::from_rgba(255, 255, 0, 255),
            ItemModelType::ArmorRed => Color::from_rgba(255, 0, 0, 255),

            ItemModelType::PowerupQuad => Color::from_rgba(0, 100, 255, 255),
            ItemModelType::PowerupBattleSuit => Color::from_rgba(0, 255, 100, 255),
            ItemModelType::PowerupHaste => Color::from_rgba(255, 255, 0, 255),
            ItemModelType::PowerupInvis => Color::from_rgba(200, 200, 255, 255),
            ItemModelType::PowerupRegen => Color::from_rgba(255, 100, 100, 255),
            ItemModelType::PowerupFlight => Color::from_rgba(255, 200, 0, 255),

            _ => WHITE,
        }
    }

    pub fn from_item_type(item_type: ItemType) -> Option<Self> {
        match item_type {
            ItemType::Health25 => Some(ItemModelType::HealthMedium),
            ItemType::Health50 => Some(ItemModelType::HealthLarge),
            ItemType::Health100 => Some(ItemModelType::HealthMega),
            ItemType::Armor50 => Some(ItemModelType::ArmorYellow),
            ItemType::Armor100 => Some(ItemModelType::ArmorRed),
            ItemType::Shotgun => Some(ItemModelType::WeaponShotgun),
            ItemType::GrenadeLauncher => Some(ItemModelType::WeaponGrenadeLauncher),
            ItemType::RocketLauncher => Some(ItemModelType::WeaponRocketLauncher),
            ItemType::LightningGun => Some(ItemModelType::WeaponLightning),
            ItemType::Railgun => Some(ItemModelType::WeaponRailgun),
            ItemType::Plasmagun => Some(ItemModelType::WeaponPlasmagun),
            ItemType::BFG => Some(ItemModelType::WeaponBFG),
            ItemType::Quad => Some(ItemModelType::PowerupQuad),
            ItemType::Regen => Some(ItemModelType::PowerupRegen),
            ItemType::Battle => Some(ItemModelType::PowerupBattleSuit),
            ItemType::Flight => Some(ItemModelType::PowerupFlight),
            ItemType::Haste => Some(ItemModelType::PowerupHaste),
            ItemType::Invis => Some(ItemModelType::PowerupInvis),
        }
    }

    pub fn model_path(&self) -> &'static str {
        match self {
            ItemModelType::HealthSmall => "q3-resources/models/powerups/health/small_cross.md3",
            ItemModelType::HealthMedium => "q3-resources/models/powerups/health/medium_cross.md3",
            ItemModelType::HealthLarge => "q3-resources/models/powerups/health/large_cross.md3",
            ItemModelType::HealthMega => "q3-resources/models/powerups/health/mega_cross.md3",

            ItemModelType::ArmorShard => "q3-resources/models/powerups/armor/shard.md3",
            ItemModelType::ArmorYellow => "q3-resources/models/powerups/armor/armor_yel.md3",
            ItemModelType::ArmorRed => "q3-resources/models/powerups/armor/armor_red.md3",

            ItemModelType::WeaponGauntlet => "q3-resources/models/weapons2/gauntlet/gauntlet.md3",
            ItemModelType::WeaponShotgun => "q3-resources/models/weapons2/shotgun/shotgun.md3",
            ItemModelType::WeaponMachinegun => {
                "q3-resources/models/weapons2/machinegun/machinegun.md3"
            }
            ItemModelType::WeaponGrenadeLauncher => {
                "q3-resources/models/weapons2/grenadel/grenadel.md3"
            }
            ItemModelType::WeaponRocketLauncher => {
                "q3-resources/models/weapons2/rocketl/rocketl.md3"
            }
            ItemModelType::WeaponLightning => {
                "q3-resources/models/weapons2/lightning/lightning.md3"
            }
            ItemModelType::WeaponRailgun => "q3-resources/models/weapons2/railgun/railgun.md3",
            ItemModelType::WeaponPlasmagun => "q3-resources/models/weapons2/plasma/plasma.md3",
            ItemModelType::WeaponBFG => "q3-resources/models/weapons2/bfg/bfg.md3",

            ItemModelType::AmmoShells => "q3-resources/models/powerups/ammo/shotgunam.md3",
            ItemModelType::AmmoBullets => "q3-resources/models/powerups/ammo/machinegunam.md3",
            ItemModelType::AmmoGrenades => "q3-resources/models/powerups/ammo/grenadeam.md3",
            ItemModelType::AmmoRockets => "q3-resources/models/powerups/ammo/rocketam.md3",
            ItemModelType::AmmoCells => "q3-resources/models/powerups/ammo/plasmaam.md3",
            ItemModelType::AmmoLightning => "q3-resources/models/powerups/ammo/lightningam.md3",
            ItemModelType::AmmoSlugs => "q3-resources/models/powerups/ammo/railgunam.md3",
            ItemModelType::AmmoBfg => "q3-resources/models/powerups/ammo/bfgam.md3",

            ItemModelType::PowerupQuad => "q3-resources/models/powerups/instant/quad.md3",
            ItemModelType::PowerupBattleSuit => "q3-resources/models/powerups/instant/enviro.md3",
            ItemModelType::PowerupHaste => "q3-resources/models/powerups/instant/haste.md3",
            ItemModelType::PowerupInvis => "q3-resources/models/powerups/instant/invis.md3",
            ItemModelType::PowerupRegen => "q3-resources/models/powerups/instant/regen.md3",
            ItemModelType::PowerupFlight => "q3-resources/models/powerups/instant/flight.md3",

            ItemModelType::HoldableTeleporter => {
                "q3-resources/models/powerups/holdable/teleporter.md3"
            }
            ItemModelType::HoldableMedkit => "q3-resources/models/powerups/holdable/medkit.md3",
        }
    }

    pub fn effect_model_path(&self) -> Option<&'static str> {
        match self {
            ItemModelType::HealthSmall => {
                Some("q3-resources/models/powerups/health/small_sphere.md3")
            }
            ItemModelType::HealthMedium => {
                Some("q3-resources/models/powerups/health/medium_sphere.md3")
            }
            ItemModelType::HealthLarge => {
                Some("q3-resources/models/powerups/health/large_sphere.md3")
            }
            ItemModelType::HealthMega => {
                Some("q3-resources/models/powerups/health/mega_sphere.md3")
            }
            ItemModelType::ArmorShard => {
                Some("q3-resources/models/powerups/armor/shard_sphere.md3")
            }
            _ => None,
        }
    }

    pub fn texture_paths(&self) -> Option<Vec<&'static str>> {
        match self {
            ItemModelType::HealthSmall
            | ItemModelType::HealthMedium
            | ItemModelType::HealthLarge
            | ItemModelType::HealthMega => Some(vec!["models/mapobjects/cross/cross.png"]),

            ItemModelType::ArmorShard => Some(vec!["models/powerups/armor/shard2.png"]),
            ItemModelType::ArmorYellow => Some(vec![
                "models/powerups/armor/newyellow.png",
                "models/powerups/armor/energy_yel3.png",
            ]),
            ItemModelType::ArmorRed => Some(vec![
                "models/powerups/armor/newred.png",
                "models/powerups/armor/energy_red1.png",
            ]),

            ItemModelType::WeaponGauntlet => Some(vec![
                "models/weapons2/gauntlet/gauntlet1.png",
                "models/weapons2/gauntlet/gauntlet3.png",
                "models/weapons2/gauntlet/gauntlet4.png",
            ]),
            ItemModelType::WeaponMachinegun => {
                Some(vec!["models/weapons2/machinegun/machinegun.png"])
            }
            ItemModelType::WeaponShotgun => Some(vec!["models/weapons2/shotgun/shotgun.png"]),
            ItemModelType::WeaponGrenadeLauncher => {
                Some(vec!["models/weapons2/grenadel/grenadel.png"])
            }
            ItemModelType::WeaponRocketLauncher => Some(vec![
                "models/weapons2/rocketl/rocketl.png",
                "models/weapons2/rocketl/rocketl2.png",
            ]),
            ItemModelType::WeaponLightning => Some(vec![
                "models/weapons2/lightning/lightning2.png",
                "models/weapons2/lightning/button.png",
                "models/weapons2/lightning/glass.png",
            ]),
            ItemModelType::WeaponRailgun => Some(vec![
                "models/weapons2/railgun/railgun1.png",
                "models/weapons2/railgun/railgun3.png",
                "models/weapons2/railgun/railgun4.png",
            ]),
            ItemModelType::WeaponPlasmagun => Some(vec!["models/weapons2/plasma/plasma.png"]),
            ItemModelType::WeaponBFG => Some(vec![
                "models/weapons2/bfg/f_bfg.png",
                "models/weapons2/bfg/f_bfg2.png",
            ]),
            _ => None,
        }
    }
}

pub struct ItemModel {
    pub model: Option<MD3Model>,
    pub effect_model: Option<MD3Model>,
    pub textures: HashMap<String, Texture2D>,
    pub rotation: f32,
    pub bob_offset: f32,
    pub prelit_color: Option<Color>,
    pub item_type: ItemModelType,
    pub respawn_time: f32,
    pub scale_factor: f32,
}

impl ItemModel {
    pub async fn load(item_type: ItemModelType) -> Result<Self, String> {
        let model_path = item_type.model_path();

        eprintln!("[MD3] Loading item model: {}", model_path);

        let model_result;
        #[cfg(target_arch = "wasm32")]
        {
            model_result = MD3Model::load_async(model_path).await;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            model_result = MD3Model::load(model_path);
        }

        let model = match model_result {
            Ok(model) => {
                eprintln!(
                    "[MD3] ✓ Loaded {}: {} frames, {} meshes",
                    model_path, model.header.num_bone_frames, model.header.num_meshes
                );
                Some(model)
            }
            Err(e) => {
                eprintln!("[MD3] ✗ Failed to load {}: {}", model_path, e);
                None
            }
        };

        let effect_model = if let Some(effect_path) = item_type.effect_model_path() {
            let effect_result;
            #[cfg(target_arch = "wasm32")]
            {
                effect_result = MD3Model::load_async(effect_path).await;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                effect_result = MD3Model::load(effect_path);
            }

            match effect_result {
                Ok(model) => {
                    eprintln!(
                        "[MD3] ✓ Loaded effect model {}: {} frames, {} meshes",
                        effect_path, model.header.num_bone_frames, model.header.num_meshes
                    );
                    Some(model)
                }
                Err(e) => {
                    eprintln!("[MD3] ✗ Failed to load effect model {}: {}", effect_path, e);
                    None
                }
            }
        } else {
            None
        };

        if model.is_none() {
            return Err(format!("Failed to load model for {:?}", item_type));
        }

        let mut textures = HashMap::new();

        if let Some(texture_paths) = item_type.texture_paths() {
            for texture_path in texture_paths {
                let full_path = format!("q3-resources/{}", texture_path);
                if let Some(texture) = super::skin_loader::load_texture_file(&full_path).await {
                    println!("[ItemModel] ✓ Loaded texture {}", texture_path);

                    if let Some(ref m) = model {
                        for mesh in &m.meshes {
                            let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                                .trim_end_matches('\0')
                                .to_string();
                            if !textures.contains_key(&mesh_name) {
                                textures.insert(mesh_name, texture.clone());
                            }
                        }
                    }
                }
            }
        }

        eprintln!("[MD3] Item model loaded successfully!");

        Ok(Self {
            model,
            effect_model,
            textures,
            rotation: 0.0,
            bob_offset: 0.0,
            prelit_color: None,
            item_type,
            respawn_time: -1000.0,
            scale_factor: 1.0,
        })
    }

    pub fn trigger_respawn(&mut self) {
        self.respawn_time = 0.0;
        self.scale_factor = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        let rotation_speed = match self.item_type {
            ItemModelType::HealthSmall
            | ItemModelType::HealthMedium
            | ItemModelType::HealthLarge
            | ItemModelType::HealthMega => dt * 1.0,
            _ => dt * 1.0,
        };

        self.rotation += rotation_speed;
        self.bob_offset = (self.rotation * 2.0).sin() * 5.0;

        if self.respawn_time >= 0.0 && self.respawn_time < 1.0 {
            self.respawn_time += dt;
            self.scale_factor = self.respawn_time.min(1.0);
        } else if self.respawn_time >= 1.0 {
            self.scale_factor = 1.0;
            self.respawn_time = -1000.0;
        }
    }

    pub fn precompute_lighting(
        &mut self,
        world_x: f32,
        world_y: f32,
        lights: &[super::map::LightSource],
        ambient: f32,
    ) {
        let mut total_r = ambient;
        let mut total_g = ambient;
        let mut total_b = ambient;

        for light in lights.iter().take(4) {
            let dx = light.x - world_x;
            let dy = light.y - world_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < light.radius {
                let attenuation = (1.0 - dist / light.radius).powf(1.6);
                let intensity = attenuation * light.intensity;
                total_r += light.r as f32 / 255.0 * intensity;
                total_g += light.g as f32 / 255.0 * intensity;
                total_b += light.b as f32 / 255.0 * intensity;
            }
        }

        self.prelit_color = Some(Color::from_rgba(
            (total_r.min(1.0) * 255.0) as u8,
            (total_g.min(1.0) * 255.0) as u8,
            (total_b.min(1.0) * 255.0) as u8,
            255,
        ));
    }

    fn get_mesh_name(mesh_header: &super::md3::MeshHeader) -> String {
        String::from_utf8_lossy(&mesh_header.name)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn render(&self, screen_x: f32, screen_y: f32, scale: f32, base_color: Color) {
        self.render_with_yaw(screen_x, screen_y, scale, base_color, self.rotation);
    }

    pub fn render_with_yaw(
        &self,
        screen_x: f32,
        screen_y: f32,
        scale: f32,
        base_color: Color,
        yaw: f32,
    ) {
        let render_y = screen_y + self.bob_offset;

        draw_ellipse(
            screen_x,
            screen_y + 15.0,
            8.0 * scale,
            3.0 * scale,
            0.0,
            Color::from_rgba(0, 0, 0, 100),
        );

        let is_weapon = matches!(
            self.item_type,
            ItemModelType::WeaponGauntlet
                | ItemModelType::WeaponShotgun
                | ItemModelType::WeaponMachinegun
                | ItemModelType::WeaponGrenadeLauncher
                | ItemModelType::WeaponRocketLauncher
                | ItemModelType::WeaponLightning
                | ItemModelType::WeaponRailgun
                | ItemModelType::WeaponPlasmagun
                | ItemModelType::WeaponBFG
        );

        let mut color = base_color;
        if is_weapon {
            let min_brightness = 0.3;
            color.r = color.r.max(min_brightness);
            color.g = color.g.max(min_brightness);
            color.b = color.b.max(min_brightness);
        }

        let weapon_scale_multiplier = if is_weapon { 1.5 } else { 1.0 };
        let final_scale = scale * self.scale_factor * weapon_scale_multiplier;
        let effect_scale = scale * self.scale_factor;

        if let Some(ref effect_model) = self.effect_model {
            let effect_color = Color::from_rgba(
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                255,
            );

            for mesh in &effect_model.meshes {
                let frame = 0.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_yaw(
                    mesh,
                    frame,
                    screen_x,
                    render_y,
                    effect_scale,
                    effect_color,
                    None,
                    false,
                    0.0,
                    yaw,
                    None,
                );
            }
        }

        if let Some(ref model) = self.model {
            for mesh in &model.meshes {
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let texture = self.textures.get(&mesh_name);
                let frame = 0.min(mesh.vertices.len().saturating_sub(1));

                super::md3_render::render_md3_mesh_with_yaw(
                    mesh,
                    frame,
                    screen_x,
                    render_y,
                    final_scale,
                    color,
                    texture,
                    false,
                    0.0,
                    yaw,
                    None,
                );
            }
        }
    }

    pub fn render_with_full_rotation(
        &self,
        screen_x: f32,
        screen_y: f32,
        scale: f32,
        base_color: Color,
        pitch: f32,
        yaw: f32,
        roll: f32,
    ) {
        let render_y = screen_y;

        draw_ellipse(
            screen_x,
            screen_y + 15.0,
            8.0 * scale,
            3.0 * scale,
            0.0,
            Color::from_rgba(0, 0, 0, 100),
        );

        let is_weapon = matches!(
            self.item_type,
            ItemModelType::WeaponGauntlet
                | ItemModelType::WeaponShotgun
                | ItemModelType::WeaponMachinegun
                | ItemModelType::WeaponGrenadeLauncher
                | ItemModelType::WeaponRocketLauncher
                | ItemModelType::WeaponLightning
                | ItemModelType::WeaponRailgun
                | ItemModelType::WeaponPlasmagun
                | ItemModelType::WeaponBFG
        );

        let mut color = base_color;
        if is_weapon {
            let min_brightness = 0.3;
            color.r = color.r.max(min_brightness);
            color.g = color.g.max(min_brightness);
            color.b = color.b.max(min_brightness);
        }

        let weapon_scale_multiplier = if is_weapon { 1.5 } else { 1.0 };
        let final_scale = scale * self.scale_factor * weapon_scale_multiplier;
        let effect_scale = scale * self.scale_factor;

        if let Some(ref effect_model) = self.effect_model {
            let effect_color = Color::from_rgba(
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                255,
            );

            for mesh in &effect_model.meshes {
                let frame = 0.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_yaw_and_roll(
                    mesh,
                    frame,
                    screen_x,
                    render_y,
                    effect_scale,
                    effect_color,
                    None,
                    None,
                    false,
                    pitch,
                    yaw,
                    roll,
                    None,
                );
            }
        }

        if let Some(ref model) = self.model {
            for mesh in &model.meshes {
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let texture = self.textures.get(&mesh_name);
                let frame = 0.min(mesh.vertices.len().saturating_sub(1));

                super::md3_render::render_md3_mesh_with_yaw_and_roll(
                    mesh,
                    frame,
                    screen_x,
                    render_y,
                    final_scale,
                    color,
                    texture,
                    None,
                    false,
                    pitch,
                    yaw,
                    roll,
                    None,
                );
            }
        }
    }
}

pub struct ItemModelCache {
    models: HashMap<ItemModelType, ItemModel>,
}

impl ItemModelCache {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub async fn load(&mut self, item_type: ItemModelType) -> Result<(), String> {
        if self.models.contains_key(&item_type) {
            return Ok(());
        }

        let model = ItemModel::load(item_type).await?;
        self.models.insert(item_type, model);
        Ok(())
    }

    pub fn get(&self, item_type: ItemModelType) -> Option<&ItemModel> {
        self.models.get(&item_type)
    }

    pub fn get_mut(&mut self, item_type: ItemModelType) -> Option<&mut ItemModel> {
        self.models.get_mut(&item_type)
    }

    pub fn update_all(&mut self, dt: f32) {
        for model in self.models.values_mut() {
            model.update(dt);
        }
    }
}
