use crate::count_shader;
use crate::game::weapon::Weapon;
use macroquad::miniquad;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

static WEAPON_HIT_MATERIAL_ADDITIVE: OnceLock<Material> = OnceLock::new();
static WEAPON_HIT_MATERIAL_ALPHA: OnceLock<Material> = OnceLock::new();

fn get_weapon_hit_material_additive() -> &'static Material {
    WEAPON_HIT_MATERIAL_ADDITIVE.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        
        varying lowp vec2 uv;
        varying lowp vec4 color;
        
        uniform mat4 Model;
        uniform mat4 Projection;
        
        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision mediump float;
        
        varying lowp vec4 color;
        varying lowp vec2 uv;
        
        uniform sampler2D Texture;
        
        void main() {
            vec4 texColor = texture2D(Texture, uv);
            vec3 finalColor = texColor.rgb * color.rgb;
            gl_FragColor = vec4(finalColor, 1.0);
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::One,
                        miniquad::BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}

fn get_weapon_hit_material_alpha() -> &'static Material {
    WEAPON_HIT_MATERIAL_ALPHA.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        
        varying lowp vec2 uv;
        varying lowp vec4 color;
        
        uniform mat4 Model;
        uniform mat4 Projection;
        
        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision mediump float;
        
        varying lowp vec4 color;
        varying lowp vec2 uv;
        
        uniform sampler2D Texture;
        
        void main() {
            vec4 texColor = texture2D(Texture, uv);
            gl_FragColor = vec4(texColor.rgb * color.rgb, texColor.a * color.a);
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::Value(miniquad::BlendValue::SourceAlpha),
                        miniquad::BlendFactor::OneMinusValue(miniquad::BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HitEffectType {
    Bullet,
    Plasma,
    Rail,
    Rocket,
    Grenade,
    BFG,
    Lightning,
    Blood,
}

pub struct WeaponHitEffect {
    pub x: f32,
    pub y: f32,
    pub effect_type: HitEffectType,
    pub frame: u32,
    pub max_frames: u32,
    pub frame_duration: u32,
    pub life: u32,
    pub scale: f32,
    pub rotation: f32,
    pub frame_time: f32,
}

impl WeaponHitEffect {
    pub fn new(x: f32, y: f32, weapon: Weapon) -> Self {
        let (effect_type, max_frames, frame_duration, scale) = match weapon {
            Weapon::MachineGun | Weapon::Shotgun => (HitEffectType::Bullet, 3, 4, 1.0),
            Weapon::Plasmagun => (HitEffectType::Plasma, 1, 6, 1.2),
            Weapon::Railgun => (HitEffectType::Rail, 4, 4, 1.5),
            Weapon::RocketLauncher => (HitEffectType::Rocket, 8, 4, 1.8),
            Weapon::GrenadeLauncher => (HitEffectType::Grenade, 3, 5, 1.5),
            Weapon::BFG => (HitEffectType::BFG, 3, 5, 2.0),
            Weapon::Lightning => (HitEffectType::Lightning, 3, 4, 1.3),
            _ => (HitEffectType::Bullet, 3, 4, 1.0),
        };

        let rotation = rand::gen_range(0.0, 360.0);

        Self {
            x,
            y,
            effect_type,
            frame: 0,
            max_frames,
            frame_duration,
            life: 0,
            scale,
            rotation,
            frame_time: 0.0,
        }
    }

    pub fn new_blood(x: f32, y: f32) -> Self {
        let rotation = rand::gen_range(0.0, 360.0);
        Self {
            x,
            y,
            effect_type: HitEffectType::Blood,
            frame: 0,
            max_frames: 5,
            frame_duration: 3,
            life: 0,
            scale: 1.0,
            rotation,
            frame_time: 0.0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.life += 1;
        self.frame_time += 1.0;

        if self.frame_time >= self.frame_duration as f32 {
            self.frame += 1;
            self.frame_time = 0.0;
        }

        self.frame < self.max_frames
    }

    pub fn render(&self, camera_x: f32, camera_y: f32, cache: &WeaponHitTextureCache) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        if screen_x < -100.0
            || screen_x > screen_width() + 100.0
            || screen_y < -100.0
            || screen_y > screen_height() + 100.0
        {
            return;
        }

        let size = 64.0 * self.scale;
        let make_params = || DrawTextureParams {
            dest_size: Some(Vec2::new(size, size)),
            rotation: self.rotation.to_radians(),
            ..Default::default()
        };

        let blend_factor = self.frame_time / self.frame_duration as f32;
        let next_frame = (self.frame + 1).min(self.max_frames - 1);

        let current_texture = cache.get_texture(&self.effect_type, self.frame);
        let next_texture = if self.frame < self.max_frames - 1 {
            cache.get_texture(&self.effect_type, next_frame)
        } else {
            None
        };

        let base_alpha = if self.frame == self.max_frames - 1 {
            1.0 - blend_factor
        } else {
            1.0
        };

        if self.effect_type == HitEffectType::Blood {
            gl_use_material(get_weapon_hit_material_alpha());

            if let Some(texture) = current_texture {
                let alpha = base_alpha * (1.0 - blend_factor);
                let color = Color::from_rgba(255, 255, 255, (alpha * 255.0) as u8);
                draw_texture_ex(
                    &texture,
                    screen_x - size / 2.0,
                    screen_y - size / 2.0,
                    color,
                    make_params(),
                );
            }

            if let Some(texture) = next_texture {
                let alpha = base_alpha * blend_factor;
                let color = Color::from_rgba(255, 255, 255, (alpha * 255.0) as u8);
                draw_texture_ex(
                    &texture,
                    screen_x - size / 2.0,
                    screen_y - size / 2.0,
                    color,
                    make_params(),
                );
            }

            count_shader!("weapon_hit_alpha");
            gl_use_default_material();
        } else {
            gl_use_material(get_weapon_hit_material_additive());

            if let Some(texture) = current_texture {
                let alpha = base_alpha * (1.0 - blend_factor);
                let color = Color::from_rgba(
                    (255.0 * alpha) as u8,
                    (255.0 * alpha) as u8,
                    (255.0 * alpha) as u8,
                    255,
                );
                draw_texture_ex(
                    &texture,
                    screen_x - size / 2.0,
                    screen_y - size / 2.0,
                    color,
                    make_params(),
                );
            }

            if let Some(texture) = next_texture {
                let alpha = base_alpha * blend_factor;
                let color = Color::from_rgba(
                    (255.0 * alpha) as u8,
                    (255.0 * alpha) as u8,
                    (255.0 * alpha) as u8,
                    255,
                );
                draw_texture_ex(
                    &texture,
                    screen_x - size / 2.0,
                    screen_y - size / 2.0,
                    color,
                    make_params(),
                );
            }

            count_shader!("weapon_hit_additive");
            gl_use_default_material();
        }
    }
}

pub struct WeaponHitTextureCache {
    textures: HashMap<(HitEffectType, u32), Texture2D>,
}

impl WeaponHitTextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub async fn load_all(&mut self) {
        self.load_bullet_textures().await;
        self.load_plasma_textures().await;
        self.load_rail_textures().await;
        self.load_rocket_textures().await;
        self.load_grenade_textures().await;
        self.load_bfg_textures().await;
        self.load_lightning_textures().await;
        self.load_blood_textures().await;
    }

    async fn load_bullet_textures(&mut self) {
        for i in 1..=3 {
            let path = format!("q3-resources/models/weaphits/bullet{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures
                    .insert((HitEffectType::Bullet, i - 1), texture);
            }
        }
    }

    async fn load_plasma_textures(&mut self) {
        let path = "q3-resources/models/weaphits/plasmaboom.png";
        if let Ok(texture) = load_texture(path).await {
            texture.set_filter(FilterMode::Linear);
            self.textures.insert((HitEffectType::Plasma, 0), texture);
        }
    }

    async fn load_rail_textures(&mut self) {
        for i in 1..=4 {
            let path = format!("q3-resources/models/weaphits/ring02_{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures.insert((HitEffectType::Rail, i - 1), texture);
            }
        }
    }

    async fn load_rocket_textures(&mut self) {
        for i in 1..=8 {
            let path = format!("q3-resources/models/weaphits/rlboom/rlboom_{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures
                    .insert((HitEffectType::Rocket, i - 1), texture);
            }
        }
    }

    async fn load_grenade_textures(&mut self) {
        for i in 1..=3 {
            let path = format!("q3-resources/models/weaphits/glboom/glboom_{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures
                    .insert((HitEffectType::Grenade, i - 1), texture);
            }
        }
    }

    async fn load_bfg_textures(&mut self) {
        for i in 1..=3 {
            let path = format!("q3-resources/models/weaphits/bfgboom/bfgboom_{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures.insert((HitEffectType::BFG, i - 1), texture);
            }
        }
    }

    async fn load_lightning_textures(&mut self) {
        for i in 1..=3 {
            let path = format!("q3-resources/models/weaphits/ring02_{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures
                    .insert((HitEffectType::Lightning, i - 1), texture);
            }
        }
    }

    async fn load_blood_textures(&mut self) {
        for i in 1..=5 {
            let path = format!("q3-resources/models/weaphits/blood20{}.png", i);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                self.textures.insert((HitEffectType::Blood, i - 1), texture);
            }
        }
    }

    pub fn get_texture(&self, effect_type: &HitEffectType, frame: u32) -> Option<&Texture2D> {
        self.textures.get(&(effect_type.clone(), frame))
    }
}
