use macroquad::prelude::*;
use crate::game::map::Map;
use crate::game::md3::MD3Model;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GibType {
    Abdomen,
    Arm,
    Brain,
    Chest,
    Fist,
    Foot,
    Forearm,
    Intestine,
    Leg,
    Skull,
}

pub struct GibModelCache {
    pub models: HashMap<GibType, MD3Model>,
    pub texture: Option<Texture2D>,
}

impl GibModelCache {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            texture: None,
        }
    }

    pub async fn load(&mut self) {
        let base_path = "q3-resources/models/gibs";
        
        // Load texture
        if let Some(tex) = crate::game::skin_loader::load_texture_file(&format!("{}/gibs.png", base_path)).await {
            self.texture = Some(tex);
        } else if let Some(tex) = crate::game::skin_loader::load_texture_file(&format!("{}/gibs.jpg", base_path)).await {
            self.texture = Some(tex);
        }

        // Load models
        let types = [
            (GibType::Abdomen, "abdomen.md3"),
            (GibType::Arm, "arm.md3"),
            (GibType::Brain, "brain.md3"),
            (GibType::Chest, "chest.md3"),
            (GibType::Fist, "fist.md3"),
            (GibType::Foot, "foot.md3"),
            (GibType::Forearm, "forearm.md3"),
            (GibType::Intestine, "intestine.md3"),
            (GibType::Leg, "leg.md3"),
            (GibType::Skull, "skull.md3"),
        ];

        for (gib_type, filename) in types.iter() {
            let path = format!("{}/{}", base_path, filename);
            #[cfg(target_arch = "wasm32")]
            {
                if let Ok(model) = MD3Model::load_async(&path).await {
                    self.models.insert(*gib_type, model);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok(model) = MD3Model::load(path) {
                    self.models.insert(*gib_type, model);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Gib {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub angle: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub spin_pitch: f32,
    pub spin_yaw: f32,
    pub spin_roll: f32,
    pub life: u32,
    pub gib_type: GibType,
    pub size: f32,
}

impl Gib {
    pub fn new(x: f32, y: f32, vel_x: f32, vel_y: f32, gib_type: GibType) -> Self {
        use crate::compat_rand::*;
        
        Self {
            x,
            y,
            vel_x,
            vel_y,
            angle: gen_f32() * std::f32::consts::PI * 2.0,
            pitch: gen_f32() * std::f32::consts::PI * 2.0,
            yaw: gen_f32() * std::f32::consts::PI * 2.0,
            roll: gen_f32() * std::f32::consts::PI * 2.0,
            spin_pitch: (gen_f32() - 0.5) * 0.5,
            spin_yaw: (gen_f32() - 0.5) * 0.5,
            spin_roll: (gen_f32() - 0.5) * 0.5,
            life: 0,
            gib_type,
            size: match gib_type {
                GibType::Skull => 8.0,
                GibType::Brain => 6.0,
                GibType::Abdomen => 8.0,
                GibType::Chest => 10.0,
                _ => 5.0,
            },
        }
    }

    pub fn update(&mut self, dt: f32, map: &Map) -> bool {
        self.life += 1;
        
        if self.life > 600 {
            return false;
        }

        let dt_norm = dt * 60.0;
        
        self.vel_y += 0.5 * dt_norm;
        
        if self.vel_y > 15.0 {
            self.vel_y = 15.0;
        }
        
        self.pitch += self.spin_pitch * dt_norm;
        self.yaw += self.spin_yaw * dt_norm;
        self.roll += self.spin_roll * dt_norm;
        
        self.x += self.vel_x * dt_norm;
        self.y += self.vel_y * dt_norm;
        
        let tile_x = (self.x / 32.0) as i32;
        let tile_y = ((self.y + 4.0) / 16.0) as i32;
        
        if map.is_solid(tile_x, tile_y) {
            self.vel_y = -self.vel_y * 0.4;
            self.vel_x *= 0.7;
            self.spin_pitch *= 0.8;
            self.spin_yaw *= 0.8;
            self.spin_roll *= 0.8;
            
            let max_corrections = 16;
            let mut correction_count = 0;
            while correction_count < max_corrections && map.is_solid(tile_x, ((self.y + 4.0) / 16.0) as i32) {
                self.y -= 1.0;
                correction_count += 1;
            }
            
            if self.vel_y.abs() < 0.5 && self.vel_x.abs() < 0.5 {
                self.vel_y = 0.0;
                self.vel_x *= 0.95;
            }
        }
        
        let wall_tile_left = ((self.x - 4.0) / 32.0) as i32;
        let wall_tile_right = ((self.x + 4.0) / 32.0) as i32;
        let wall_y = (self.y / 16.0) as i32;
        
        if map.is_solid(wall_tile_left, wall_y) || map.is_solid(wall_tile_right, wall_y) {
            self.vel_x = -self.vel_x * 0.4;
        }
        
        true
    }

    pub fn render(&self, camera_x: f32, camera_y: f32, cache: &GibModelCache) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        let screen_w = macroquad::window::screen_width();
        let screen_h = macroquad::window::screen_height();
        let margin = 100.0;
        
        if screen_x < -margin || screen_x > screen_w + margin || 
           screen_y < -margin || screen_y > screen_h + margin {
            return;
        }
        
        let alpha = if self.life > 300 {
            ((600 - self.life) as f32 / 300.0).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if let Some(model) = cache.models.get(&self.gib_type) {
            let color = Color::from_rgba(255, 255, 255, (255.0 * alpha) as u8);
            
            // Render MD3 model
            for mesh in &model.meshes {
                let safe_frame = 0; // Gibs usually have 1 frame or we just use the first
                crate::game::md3_render::render_md3_mesh_with_yaw_and_roll(
                    mesh,
                    safe_frame,
                    screen_x,
                    screen_y,
                    1.0, // Scale
                    color,
                    cache.texture.as_ref(),
                    None,
                    false, // flip_x
                    self.pitch,
                    self.yaw,
                    self.roll,
                    None, // lighting context (optional, could add if needed)
                );
            }
        } else {
            // Fallback rendering if model not found
            draw_circle(screen_x, screen_y, self.size, Color::from_rgba(200, 50, 50, (255.0 * alpha) as u8));
        }
        
        if self.life < 40 {
            let blood_trail = (self.life as f32 * 0.7) as i32;
            let vel_mag = (self.vel_x * self.vel_x + self.vel_y * self.vel_y).sqrt();
            
            if vel_mag.is_finite() && vel_mag < 50.0 {
                for i in 0..blood_trail {
                    let trail_x = screen_x - self.vel_x * (i as f32 * 0.4);
                    let trail_y = screen_y - self.vel_y * (i as f32 * 0.4);
                    
                    if trail_x.is_finite() && trail_y.is_finite() &&
                       trail_x >= -margin && trail_x <= screen_w + margin &&
                       trail_y >= -margin && trail_y <= screen_h + margin {
                        let trail_alpha = ((blood_trail - i) as f32 / blood_trail as f32) * 0.5;
                        let size = 2.0 - (i as f32 / blood_trail as f32) * 1.0;
                        draw_circle(trail_x, trail_y, size, Color::from_rgba(200, 10, 10, (255.0 * trail_alpha) as u8));
                    }
                }
            }
        }
    }
}

pub fn spawn_gibs(x: f32, y: f32) -> Vec<Gib> {
    use crate::compat_rand::*;
    
    let mut gibs = Vec::new();
    
    let gib_velocity = 10.0;
    let gib_jump = -10.0;
    
    let mega_explosion = gen_bool(0.1);
    
    // Always spawn essential parts
    gibs.push(Gib::new(x, y, (gen_f32() - 0.5) * gib_velocity, gib_jump + (gen_f32() - 0.5) * gib_velocity, GibType::Skull));
    gibs.push(Gib::new(x, y, (gen_f32() - 0.5) * gib_velocity, gib_jump + (gen_f32() - 0.5) * gib_velocity, GibType::Chest));
    gibs.push(Gib::new(x, y, (gen_f32() - 0.5) * gib_velocity, gib_jump + (gen_f32() - 0.5) * gib_velocity, GibType::Abdomen));
    
    // Random limbs
    let limb_count = if mega_explosion { gen_range_usize(150, 500) } else { gen_range_usize(3, 6) };
    for _ in 0..limb_count {
        let limb_type = match gen_range_usize(0, 4) {
            0 => GibType::Arm,
            1 => GibType::Leg,
            2 => GibType::Forearm,
            _ => GibType::Foot,
        };
        gibs.push(Gib::new(x, y, (gen_f32() - 0.5) * gib_velocity, gib_jump + (gen_f32() - 0.5) * gib_velocity, limb_type));
    }
    
    // Organs
    let organ_count = if mega_explosion { gen_range_usize(100, 750) } else { gen_range_usize(2, 5) };
    for _ in 0..organ_count {
        let organ_type = match gen_range_usize(0, 3) {
            0 => GibType::Brain,
            1 => GibType::Intestine,
            _ => GibType::Fist, // Why fist? Quake 3 has fist gibs.
        };
        gibs.push(Gib::new(x, y, (gen_f32() - 0.5) * gib_velocity, gib_jump + (gen_f32() - 0.5) * gib_velocity, organ_type));
    }
    
    gibs
}
