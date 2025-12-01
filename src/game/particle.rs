use super::map::Map;
use crate::compat_rand::*;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub enum ParticleType {
    Blood,
    Smoke,
    Explosion,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub life: f32,
    pub max_life: f32,
    pub alpha: f32,
    pub radius: f32,
    pub start_radius: f32,
    pub end_radius: f32,
    pub particle_type: ParticleType,
    pub _angle: f32,
    pub _visual: u8,
    pub _spotted: bool,
}

impl Particle {
    pub fn new(x: f32, y: f32, vel_x: f32, vel_y: f32, long: bool) -> Self {
        Self {
            x,
            y,
            vel_x,
            vel_y,
            life: 0.0,
            max_life: if long { 2.5 } else { 0.833 },
            alpha: 1.0,
            radius: 3.0,
            start_radius: 3.0,
            end_radius: 3.0,
            particle_type: ParticleType::Blood,
            _angle: gen_f32() * 360.0,
            _visual: gen_u8() % 4,
            _spotted: false,
        }
    }

    pub fn new_smoke(x: f32, y: f32, vel_x: f32, vel_y: f32, radius: f32, duration: f32) -> Self {
        Self {
            x,
            y,
            vel_x,
            vel_y,
            life: 0.0,
            max_life: duration,
            alpha: 0.33,
            radius,
            start_radius: radius * 0.5,
            end_radius: radius * 2.0,
            particle_type: ParticleType::Smoke,
            _angle: gen_f32() * 360.0,
            _visual: gen_u8() % 4,
            _spotted: false,
        }
    }

    pub fn new_explosion(x: f32, y: f32, vel_x: f32, vel_y: f32, size: f32) -> Self {
        Self {
            x,
            y,
            vel_x,
            vel_y,
            life: 0.0,
            max_life: 1.0,
            alpha: 1.0,
            radius: size,
            start_radius: size * 0.2,
            end_radius: size * 1.5,
            particle_type: ParticleType::Explosion,
            _angle: gen_f32() * 360.0,
            _visual: gen_u8() % 4,
            _spotted: false,
        }
    }

    pub fn update(&mut self, dt: f32, map: &Map) -> bool {
        let dt_60fps = dt * 60.0;

        self.x += self.vel_x * dt_60fps;
        self.y += self.vel_y * dt_60fps;

        match self.particle_type {
            ParticleType::Blood => {
                self.vel_y += 0.035 * dt_60fps;
                self.vel_x *= 0.99_f32.powf(dt_60fps);

                if self.life > self.max_life / 2.0 {
                    self.alpha = 1.0 - ((self.life - self.max_life / 2.0) / (self.max_life / 2.0));
                } else {
                    self.alpha = 0.94;
                }

                let tile_x = (self.x / 32.0) as i32;
                let tile_y = ((self.y + 3.0) / 16.0) as i32;

                if map.is_solid(tile_x, tile_y) {
                    self.y = (self.y / 16.0).round() * 16.0;
                    self.vel_x = 0.0;
                    self.vel_y = 0.0;

                    if gen_range_i32(0, 2) == 0 {
                        self.life = (self.life - dt).max(0.0);
                    }
                }
            }
            ParticleType::Smoke => {
                self.vel_x *= 0.98_f32.powf(dt_60fps);
                self.vel_y *= 0.98_f32.powf(dt_60fps);

                let life_ratio = self.life / self.max_life;
                self.alpha = 0.33 * (1.0 - life_ratio);
                self.radius =
                    self.start_radius + (self.end_radius - self.start_radius) * life_ratio;
            }
            ParticleType::Explosion => {
                self.vel_x *= 0.95_f32.powf(dt_60fps);
                self.vel_y *= 0.95_f32.powf(dt_60fps);

                let life_ratio = self.life / self.max_life;

                if life_ratio < 0.3 {
                    self.alpha = 1.0;
                    self.radius = self.start_radius
                        + (self.end_radius - self.start_radius) * (life_ratio / 0.3);
                } else {
                    self.alpha = 1.0 - ((life_ratio - 0.3) / 0.7);
                    self.radius = self.end_radius;
                }
            }
        }

        self.life += dt;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        if screen_x < -32.0
            || screen_x > screen_width() + 32.0
            || screen_y < -32.0
            || screen_y > screen_height() + 32.0
        {
            return;
        }

        match self.particle_type {
            ParticleType::Blood => {}
            ParticleType::Smoke => {}
            ParticleType::Explosion => {}
        }
    }
}
