use macroquad::prelude::*;
use crate::game::map::Map;

#[derive(Clone, Copy, Debug)]
pub enum GibType {
    Skull,
    Brain,
    Meat1,
    Meat2,
    Meat3,
    Bone,
}

#[derive(Clone, Debug)]
pub struct Gib {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub angle: f32,
    pub spin: f32,
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
            spin: (gen_f32() - 0.5) * 0.3,
            life: 0,
            gib_type,
            size: match gib_type {
                GibType::Skull => 8.0,
                GibType::Brain => 6.0,
                GibType::Bone => 5.0,
                GibType::Meat1 => 5.0,
                GibType::Meat2 => 4.5,
                GibType::Meat3 => 4.0,
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
        
        self.angle += self.spin * dt_norm;
        
        self.x += self.vel_x * dt_norm;
        self.y += self.vel_y * dt_norm;
        
        let tile_x = (self.x / 32.0) as i32;
        let tile_y = ((self.y + 4.0) / 16.0) as i32;
        
        if map.is_solid(tile_x, tile_y) {
            self.vel_y = -self.vel_y * 0.4;
            self.vel_x *= 0.7;
            self.spin *= 0.8;
            
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

    pub fn render(&self, camera_x: f32, camera_y: f32) {
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

        match self.gib_type {
            GibType::Skull => {
                draw_circle(screen_x, screen_y, self.size, Color::from_rgba(220, 210, 200, (255.0 * alpha) as u8));
                draw_circle(screen_x - 2.0, screen_y - 1.0, 2.0, Color::from_rgba(50, 50, 50, (200.0 * alpha) as u8));
                draw_circle(screen_x + 2.0, screen_y - 1.0, 2.0, Color::from_rgba(50, 50, 50, (200.0 * alpha) as u8));
                draw_circle(screen_x, screen_y + 2.0, 1.5, Color::from_rgba(50, 50, 50, (200.0 * alpha) as u8));
            }
            GibType::Brain => {
                draw_circle(screen_x, screen_y, self.size, Color::from_rgba(220, 100, 120, (255.0 * alpha) as u8));
                draw_circle(screen_x - 2.0, screen_y, 3.0, Color::from_rgba(200, 80, 100, (220.0 * alpha) as u8));
                draw_circle(screen_x + 2.0, screen_y, 3.0, Color::from_rgba(200, 80, 100, (220.0 * alpha) as u8));
                draw_circle(screen_x, screen_y + 1.5, 2.0, Color::from_rgba(180, 50, 70, (200.0 * alpha) as u8));
                draw_circle(screen_x - 1.0, screen_y - 1.0, 1.5, Color::from_rgba(160, 30, 50, (180.0 * alpha) as u8));
            }
            GibType::Bone => {
                let cos_a = self.angle.cos();
                let sin_a = self.angle.sin();
                let len = self.size;
                
                let x1 = screen_x + cos_a * len;
                let y1 = screen_y + sin_a * len;
                let x2 = screen_x - cos_a * len;
                let y2 = screen_y - sin_a * len;
                
                draw_line(x1, y1, x2, y2, 3.0, Color::from_rgba(220, 210, 200, (255.0 * alpha) as u8));
                draw_circle(x1, y1, 2.0, Color::from_rgba(240, 230, 220, (255.0 * alpha) as u8));
                draw_circle(x2, y2, 2.0, Color::from_rgba(240, 230, 220, (255.0 * alpha) as u8));
            }
            GibType::Meat1 => {
                draw_circle(screen_x, screen_y, self.size, Color::from_rgba(200, 20, 20, (255.0 * alpha) as u8));
                draw_circle(screen_x - 1.0, screen_y - 1.0, 2.5, Color::from_rgba(255, 40, 40, (220.0 * alpha) as u8));
                draw_circle(screen_x + 0.5, screen_y + 0.5, 1.5, Color::from_rgba(140, 10, 10, (200.0 * alpha) as u8));
            }
            GibType::Meat2 => {
                draw_circle(screen_x, screen_y, self.size, Color::from_rgba(180, 15, 15, (255.0 * alpha) as u8));
                draw_circle(screen_x + 1.0, screen_y, 2.5, Color::from_rgba(220, 30, 30, (220.0 * alpha) as u8));
                draw_circle(screen_x - 0.5, screen_y - 0.5, 1.8, Color::from_rgba(120, 10, 10, (200.0 * alpha) as u8));
            }
            GibType::Meat3 => {
                draw_circle(screen_x, screen_y, self.size, Color::from_rgba(160, 10, 10, (255.0 * alpha) as u8));
                draw_circle(screen_x, screen_y + 1.0, 3.0, Color::from_rgba(200, 25, 25, (220.0 * alpha) as u8));
                draw_circle(screen_x + 1.0, screen_y - 1.0, 1.5, Color::from_rgba(100, 5, 5, (200.0 * alpha) as u8));
            }
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
    
    let skull_count = if mega_explosion {
        gen_range_usize(20, 80)
    } else {
        if gen_bool(0.5) { 1 } else { 0 }
    };
    
    let brain_count = if mega_explosion {
        gen_range_usize(15, 60)
    } else {
        if skull_count == 0 { 1 } else { 0 }
    };
    
    let meat_count = if mega_explosion {
        gen_range_usize(100, 500)
    } else {
        gen_range_usize(5, 12)
    };
    
    let bone_count = if mega_explosion {
        gen_range_usize(50, 200)
    } else {
        gen_range_usize(2, 6)
    };
    
    for _ in 0..skull_count {
        gibs.push(Gib::new(
            x,
            y,
            (gen_f32() - 0.5) * gib_velocity,
            gib_jump + (gen_f32() - 0.5) * gib_velocity,
            GibType::Skull,
        ));
    }
    
    for _ in 0..brain_count {
        gibs.push(Gib::new(
            x,
            y,
            (gen_f32() - 0.5) * gib_velocity,
            gib_jump + (gen_f32() - 0.5) * gib_velocity,
            GibType::Brain,
        ));
    }
    
    for _ in 0..meat_count {
        let gib_type = match gen_range_usize(0, 3) {
            0 => GibType::Meat1,
            1 => GibType::Meat2,
            _ => GibType::Meat3,
        };
        
        gibs.push(Gib::new(
            x,
            y,
            (gen_f32() - 0.5) * gib_velocity * 1.2,
            gib_jump + (gen_f32() - 0.5) * gib_velocity,
            gib_type,
        ));
    }
    
    for _ in 0..bone_count {
        gibs.push(Gib::new(
            x,
            y,
            (gen_f32() - 0.5) * gib_velocity,
            gib_jump + (gen_f32() - 0.5) * gib_velocity,
            GibType::Bone,
        ));
    }
    
    gibs
}

