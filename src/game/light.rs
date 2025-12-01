use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct ExplosionFlash {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub life: u8,
    pub max_life: u8,
    pub color: Color,
    pub flash_intensity: f32,
}

impl ExplosionFlash {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Self {
            x,
            y,
            radius: radius * 10.0,
            life: 0,
            max_life: 255,
            color: Color::from_rgba(255, 255, 180, 255),
            flash_intensity: 47.0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.life = self.life.saturating_add(1);
        let life_ratio = self.life as f32 / self.max_life as f32;

        if life_ratio < 0.3 {
            self.flash_intensity = 1.0;
        } else {
            self.flash_intensity = 1.0 - ((life_ratio - 0.3) / 0.7);
        }

        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        if screen_x < -100.0
            || screen_x > screen_width() + 100.0
            || screen_y < -100.0
            || screen_y > screen_height() + 100.0
        {
            return;
        }
    }
}

#[derive(Clone, Debug)]
pub struct LightPulse {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub life: u8,
    pub max_life: u8,
    pub color: Color,
}

impl LightPulse {
    pub fn new(x: f32, y: f32, radius: f32, color: Color, max_life_ms: u32) -> Self {
        let frames = ((max_life_ms as f32 / 1000.0) * 60.0).clamp(2.0, 30.0) as u8;
        Self {
            x,
            y,
            radius,
            life: 0,
            max_life: frames,
            color,
        }
    }

    pub fn new_explosion_flash(x: f32, y: f32, _radius: f32) -> Self {
        Self {
            x,
            y,
            radius: 200.0,
            life: 0,
            max_life: 12,
            color: Color::from_rgba(255, 220, 150, 255),
        }
    }

    pub fn from_weapon(x: f32, y: f32, weapon_id: u8) -> Self {
        match weapon_id {
            4 => Self::new(x, y, 140.0, Color::from_rgba(255, 180, 80, 180), 120),
            6 => Self::new(x, y, 130.0, Color::from_rgba(210, 230, 255, 200), 100),
            7 => Self::new(x, y, 120.0, Color::from_rgba(80, 180, 255, 180), 110),
            8 => Self::new(x, y, 160.0, Color::from_rgba(120, 255, 120, 180), 140),
            _ => Self::new(x, y, 100.0, Color::from_rgba(255, 210, 120, 160), 90),
        }
    }

    pub fn update(&mut self) -> bool {
        self.life = self.life.saturating_add(1);
        self.life < self.max_life
    }
}
