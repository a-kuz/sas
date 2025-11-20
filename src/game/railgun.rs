use macroquad::prelude::*;
use crate::game::map::Map;
use crate::game::particle::Particle;
use crate::compat_rand::*;

const RAIL_BEAM_DURATION: f32 = 0.8;
const RAIL_CORE_WIDTH: f32 = 4.0;
const RAIL_MAX_RANGE: f32 = 8192.0;
const RAIL_MAX_PENETRATIONS: u8 = 4;

#[derive(Clone, Debug)]
pub struct RailBeam {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub _player_id: u16,
}

#[derive(Clone, Debug)]
pub struct RailRing {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub _distance_along_beam: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct RailExplosion {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct RailgunEffects {
    pub beams: Vec<RailBeam>,
    pub rings: Vec<RailRing>,
    pub explosions: Vec<RailExplosion>,
}

impl RailBeam {
    pub fn new(start_x: f32, start_y: f32, end_x: f32, end_y: f32, player_color: Color) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
            life: 0.0,
            max_life: RAIL_BEAM_DURATION,
            color: player_color,
            _player_id: 0,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life += dt;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_start_x = self.start_x - camera_x;
        let screen_start_y = self.start_y - camera_y;
        let screen_end_x = self.end_x - camera_x;
        let screen_end_y = self.end_y - camera_y;

        let t = self.life / self.max_life;
        let fade = 1.0 - (t * t);
        
        let rail_width = crate::cvar::get_cvar_float("r_railWidth");
        let width_scale = (rail_width / 16.0).max(0.01).min(15.0);
        
        let core_color = Color::from_rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            (fade * 180.0) as u8,
        );

        let inner_color = Color::from_rgba(255, 255, 255, (fade * 255.0) as u8);
        
        draw_line(
            screen_start_x,
            screen_start_y,
            screen_end_x,
            screen_end_y,
            1.5 * fade * width_scale,
            core_color,
        );

        draw_line(
            screen_start_x,
            screen_start_y,
            screen_end_x,
            screen_end_y,
            0.8 * fade * width_scale,
            inner_color,
        );
    }
}

impl RailRing {
    pub fn new(beam_start_x: f32, beam_start_y: f32, beam_end_x: f32, beam_end_y: f32, 
               distance: f32, rotation: f32, color: Color) -> Self {
        let beam_dx = beam_end_x - beam_start_x;
        let beam_dy = beam_end_y - beam_start_y;
        let beam_length = (beam_dx * beam_dx + beam_dy * beam_dy).sqrt();
        
        let t = if beam_length > 0.0 { distance / beam_length } else { 0.0 };
        let x = beam_start_x + beam_dx * t;
        let y = beam_start_y + beam_dy * t;

        Self {
            x,
            y,
            rotation,
            _distance_along_beam: distance,
            life: 0.0,
            max_life: RAIL_BEAM_DURATION,
            color,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life += dt;
        self.rotation += dt * 360.0;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let fade = 1.0 - (self.life / self.max_life);
        let alpha = (fade * 180.0) as u8;

        let ring_color = Color::from_rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            alpha,
        );

        let radius = RAIL_CORE_WIDTH * fade;
        draw_circle_lines(screen_x, screen_y, radius, 2.0, ring_color);
    }
}

impl RailExplosion {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            x,
            y,
            life: 0.0,
            max_life: 0.6,
            color,
            radius: 24.0,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life += dt;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let t = self.life / self.max_life;
        let fade = 1.0 - (t * t);

        let current_radius = self.radius * (0.5 + t * 0.5);
        let alpha = (fade * 200.0) as u8;

        let explosion_color = Color::from_rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            alpha,
        );

        draw_circle(screen_x, screen_y, current_radius, explosion_color);

        let inner_color = Color::from_rgba(255, 255, 255, (alpha as f32 * 0.6) as u8);
        draw_circle(screen_x, screen_y, current_radius * 0.6, inner_color);
    }

    pub fn create_particles(&self) -> Vec<Particle> {
        let mut particles = Vec::new();
        

        for _ in 0..15 {
            let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
            let speed = gen_range_f32(2.0, 6.0);
            let particle = Particle::new(
                self.x,
                self.y,
                angle.cos() * speed,
                angle.sin() * speed,
                true,
            );
            particles.push(particle);
        }

        particles
    }
}

impl RailgunEffects {
    pub fn new() -> Self {
        Self {
            beams: Vec::new(),
            rings: Vec::new(),
            explosions: Vec::new(),
        }
    }

    pub fn fire_railgun(&mut self, start_x: f32, start_y: f32, end_x: f32, end_y: f32, 
                       player_color: Color) {
        let beam = RailBeam::new(start_x, start_y, end_x, end_y, player_color);
        self.beams.push(beam);


        let explosion = RailExplosion::new(end_x, end_y, player_color);
        self.explosions.push(explosion);
    }

    pub fn get_linear_lights(&self) -> Vec<super::map::LinearLight> {
        let mut lights = Vec::new();
        
        for beam in &self.beams {
            let t = beam.life / beam.max_life;
            let fade = 1.0 - (t * t);
            if fade > 0.1 {
                lights.push(super::map::LinearLight {
                    start_x: beam.start_x,
                    start_y: beam.start_y,
                    end_x: beam.end_x,
                    end_y: beam.end_y,
                    width: RAIL_CORE_WIDTH * 25.0 * fade,
                    r: (beam.color.r * 255.0) as u8,
                    g: (beam.color.g * 255.0) as u8,
                    b: (beam.color.b * 255.0) as u8,
                    intensity: fade * 12.0,
                });
            }
        }
        
        lights
    }

    pub fn get_explosion_lights(&self) -> Vec<super::map::LightSource> {
        let mut lights = Vec::new();
        
        for explosion in &self.explosions {
            let t = explosion.life / explosion.max_life;
            let fade = 1.0 - (t * t);
            if fade > 0.1 {
                lights.push(super::map::LightSource {
                    x: explosion.x,
                    y: explosion.y,
                    radius: explosion.radius * 2.0 * fade,
                    r: (explosion.color.r * 255.0) as u8,
                    g: (explosion.color.g * 255.0) as u8,
                    b: (explosion.color.b * 255.0) as u8,
                    intensity: fade * 8.0,
                    flicker: false,
                });
            }
        }
        
        lights
    }

    pub fn update(&mut self, dt: f32) {
        self.beams.retain_mut(|beam| beam.update(dt));
        self.explosions.retain_mut(|explosion| explosion.update(dt));
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        for beam in &self.beams {
            beam.render(camera_x, camera_y);
        }

        for explosion in &self.explosions {
            explosion.render(camera_x, camera_y);
        }
    }

    pub fn create_explosion_particles(&self) -> Vec<Particle> {
        let mut all_particles = Vec::new();
        
        for explosion in &self.explosions {
            if explosion.life < 0.1 {
                let particles = explosion.create_particles();
                all_particles.extend(particles);
            }
        }

        all_particles
    }
}

pub fn fire_railgun_hitscan(
    start_x: f32,
    start_y: f32,
    angle: f32,
    _owner_id: u16,
    map: &Map,
) -> (f32, f32, Vec<(usize, i32, f32, f32)>) {
    let mut current_x = start_x;
    let mut current_y = start_y;
    let hits = Vec::new();
    let penetrations = 0;

    let dx = angle.cos();
    let dy = angle.sin();

    let step_size = 8.0;
    let max_steps = (RAIL_MAX_RANGE / step_size) as i32;

    for _ in 0..max_steps {
        current_x += dx * step_size;
        current_y += dy * step_size;

        let tile_x = (current_x / 32.0) as i32;
        let tile_y = (current_y / 16.0) as i32;

        if map.is_solid(tile_x, tile_y) {
            break;
        }

        if penetrations >= RAIL_MAX_PENETRATIONS {
            break;
        }
    }

    (current_x, current_y, hits)
}

pub fn get_player_railgun_color(player_id: u16) -> Color {
    match player_id {
        1 => Color::from_rgba(100, 200, 255, 255),
        2 => Color::from_rgba(255, 100, 100, 255),
        3 => Color::from_rgba(100, 255, 100, 255),
        4 => Color::from_rgba(255, 255, 100, 255),
        _ => Color::from_rgba(200, 150, 255, 255),
    }
}
