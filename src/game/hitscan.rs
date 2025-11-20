use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct BulletHole {
    pub x: f32,
    pub y: f32,
    pub life: u8,
}

#[derive(Clone, Debug)]
pub struct DebugRay {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub life: u8,
    pub hit: bool,
}

impl BulletHole {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            life: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.life += 1;
        self.life < 100
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        let alpha = (1.0 - (self.life as f32 / 100.0)) * 0.6;
        
        draw_circle(
            screen_x,
            screen_y,
            2.0,
            Color::from_rgba(40, 40, 40, (alpha * 255.0) as u8),
        );
    }
}

impl DebugRay {
    pub fn new(start_x: f32, start_y: f32, end_x: f32, end_y: f32, hit: bool) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
            life: 0,
            hit,
        }
    }

    pub fn update(&mut self) -> bool {
        self.life += 1;
        self.life < 60
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_start_x = self.start_x - camera_x;
        let screen_start_y = self.start_y - camera_y;
        let screen_end_x = self.end_x - camera_x;
        let screen_end_y = self.end_y - camera_y;
        
        let alpha = (1.0 - (self.life as f32 / 60.0)) * 0.8;
        
        let color = if self.hit {
            Color::from_rgba(255, 50, 50, (alpha * 255.0) as u8)
        } else {
            Color::from_rgba(255, 255, 50, (alpha * 255.0) as u8)
        };
        
        draw_line(
            screen_start_x,
            screen_start_y,
            screen_end_x,
            screen_end_y,
            3.0,
            color,
        );
    }
}

pub fn fire_hitscan(
    start_x: f32,
    start_y: f32,
    angle: f32,
    _range: f32,
    spread: f32,
    count: u8,
    owner: u16,
    damage_per_pellet: i32,
) -> Vec<(f32, f32, f32, u16, i32)> {
    let mut rays = Vec::new();
    
    for i in 0..count {
        let spread_angle = if count > 1 {
            let spread_range = spread;
            let offset = (i as f32 / (count - 1) as f32) - 0.5;
            angle + offset * spread_range
        } else {
            angle + (rand::gen_range(-spread, spread))
        };
        
        rays.push((start_x, start_y, spread_angle, owner, damage_per_pellet));
    }
    
    rays
}

