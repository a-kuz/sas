use super::map::Map;
use super::particle::Particle;
use super::projectile_model_cache::ProjectileModelCache;
use super::weapon::Weapon;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Projectile {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub owner_id: u16,
    pub weapon_type: Weapon,
    pub life: u32,
    pub life_secs: f32,
    pub damage: i32,
    pub active: bool,
    pub angle: f32,
    pub bounces: u8,
    pub trail_time: u32,
    pub last_trail_x: f32,
    pub last_trail_y: f32,
    pub just_bounced: bool,
    pub is_rolling: bool,
}

impl Projectile {
    pub fn new(
        x: f32,
        y: f32,
        angle: f32,
        owner_id: u16,
        weapon: Weapon,
        player_vel_x: f32,
        player_vel_y: f32,
    ) -> Self {
        let (vel_x, vel_y) = match weapon {
            Weapon::RocketLauncher => {
                let speed = 15.0;
                (angle.cos() * speed, angle.sin() * speed)
            }
            Weapon::GrenadeLauncher => {
                let base_speed = 16.0;
                let vx = angle.cos() * base_speed + player_vel_x * 0.5;
                let mut vy = angle.sin() * base_speed + player_vel_y * 0.5;
                vy -= 1.5;
                (vx, vy)
            }
            Weapon::Plasmagun => {
                let speed = 33.0;
                (angle.cos() * speed, angle.sin() * speed)
            }
            Weapon::BFG => {
                let speed = 33.0;
                (angle.cos() * speed, angle.sin() * speed)
            }
            Weapon::Railgun => {
                let speed = 200.0;
                (angle.cos() * speed, angle.sin() * speed)
            }
            _ => (0.0, 0.0),
        };

        Self {
            id: 0,
            x,
            y,
            vel_x,
            vel_y,
            owner_id,
            weapon_type: weapon,
            life: 0,
            life_secs: 0.0,
            damage: weapon.damage(),
            active: true,
            angle: 0.0,
            bounces: 0,
            trail_time: 0,
            last_trail_x: x,
            last_trail_y: y,
            just_bounced: false,
            is_rolling: false,
        }
    }

    pub fn update(&mut self, dt: f32, map: &Map) -> bool {
        use super::constants::*;

        if !self.active {
            return false;
        }

        self.just_bounced = false;

        let dt_60fps = dt * 60.0;

        if matches!(self.weapon_type, Weapon::GrenadeLauncher) {
            self.vel_y += 0.25 * dt_60fps;

            let check_ground = ((self.y + 5.0) / 16.0) as i32;
            let is_on_ground =
                map.is_solid((self.x / 32.0) as i32, check_ground) && self.vel_y.abs() < 0.5;

            self.is_rolling = is_on_ground && self.vel_x.abs() > 0.1;

            if is_on_ground {
                self.vel_y = 0.0;
                self.vel_x *= 0.98_f32.powf(dt_60fps);
            } else {
                self.vel_y *= 0.998_f32.powf(dt_60fps);
                self.vel_x *= 0.998_f32.powf(dt_60fps);
            }

            if self.vel_y.abs() < 0.1 && self.vel_x.abs() < 0.1 {
                self.vel_x = 0.0;
                self.vel_y = 0.0;
            }

            let old_x = self.x;
            let old_y = self.y;

            self.x += self.vel_x * dt_60fps;

            let check_left = ((self.x - 4.0) / 32.0) as i32;
            let check_right = ((self.x + 4.0) / 32.0) as i32;
            let center_y = (self.y / 16.0) as i32;

            if (map.is_solid(check_left, center_y) && self.vel_x < 0.0)
                || (map.is_solid(check_right, center_y) && self.vel_x > 0.0)
            {
                self.x = old_x;

                if self.vel_x.abs() < 1.0 {
                    self.vel_x = 0.0;
                } else {
                    self.vel_x = -self.vel_x * GRENADE_BOUNCE_WALL;
                    self.vel_x /= GRENADE_SLOWDOWN;
                    self.bounces += 1;
                    self.just_bounced = true;
                }
            }

            self.y += self.vel_y * dt_60fps;

            let check_top = ((self.y - 4.0) / 16.0) as i32;
            let check_bottom = ((self.y + 4.0) / 16.0) as i32;
            let center_x = (self.x / 32.0) as i32;

            if (map.is_solid(center_x, check_top) && self.vel_y < 0.0)
                || (map.is_solid(center_x, check_bottom) && self.vel_y > 0.0)
            {
                self.y = old_y;

                if self.vel_y.abs() < 1.5 {
                    self.vel_y = 0.0;
                    self.is_rolling = true;
                } else {
                    self.vel_y = -self.vel_y * GRENADE_BOUNCE_FLOOR;
                    self.vel_x /= GRENADE_SLOWDOWN;
                    self.bounces += 1;
                    self.just_bounced = true;
                }
            }

            if self.vel_x != 0.0 {
                self.angle += if self.vel_x > 0.0 {
                    -2.0 * dt_60fps
                } else {
                    2.0 * dt_60fps
                };
            }
        } else {
            self.x += self.vel_x * dt_60fps;
            self.y += self.vel_y * dt_60fps;
        }

        if matches!(
            self.weapon_type,
            Weapon::RocketLauncher | Weapon::Plasmagun | Weapon::BFG
        ) {
            self.vel_x *= 0.999_f32.powf(dt_60fps);
        }

        let tile_x = (self.x / 32.0) as i32;
        let tile_y = (self.y / 16.0) as i32;

        if map.is_solid(tile_x, tile_y) {
            if !matches!(self.weapon_type, Weapon::GrenadeLauncher) {
                self.active = false;
                return false;
            }
        }

        self.life += 1;
        self.life_secs += dt;
        self.trail_time += 1;

        let max_life = match self.weapon_type {
            Weapon::RocketLauncher => 600,
            Weapon::GrenadeLauncher => (GRENADE_FUSE_SECS * 60.0) as u32 + 60,
            Weapon::Plasmagun => 60,
            Weapon::BFG => 300,
            Weapon::Railgun => 8,
            Weapon::Gauntlet => 0,
            Weapon::MachineGun => 0,
            Weapon::Shotgun => 0,
            Weapon::Lightning => 0,
        };

        if self.life > max_life {
            self.active = false;
            return false;
        }

        if matches!(self.weapon_type, Weapon::GrenadeLauncher) && self.life_secs > GRENADE_FUSE_SECS
        {
            self.active = false;
            return false;
        }

        true
    }

    pub fn render(&self, camera_x: f32, camera_y: f32, cache: &mut ProjectileModelCache) {
        if !self.active {
            return;
        }

        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        match self.weapon_type {
            Weapon::RocketLauncher => {
                let has_model = cache
                    .get_model(super::projectile_model_cache::ProjectileModelType::Rocket)
                    .is_some();
                if !has_model {
                    cache.get_or_load_model(
                        super::projectile_model_cache::ProjectileModelType::Rocket,
                    );
                }

                let has_texture = cache
                    .get_texture("q3-resources/models/ammo/rocket/rocket.png")
                    .is_some();
                if !has_texture {
                    cache.get_or_load_texture("q3-resources/models/ammo/rocket/rocket.png");
                }

                if let Some(model) =
                    cache.get_model(super::projectile_model_cache::ProjectileModelType::Rocket)
                {
                    let texture = cache.get_texture("q3-resources/models/ammo/rocket/rocket.png");
                    let angle = self.vel_y.atan2(self.vel_x);
                    let offset_x = angle.cos() * 8.0;
                    let offset_y = angle.sin() * 8.0;

                    for mesh in &model.meshes {
                        super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex(
                            mesh,
                            0,
                            screen_x + offset_x,
                            screen_y + offset_y,
                            1.0,
                            WHITE,
                            texture,
                            Some("q3-resources/models/ammo/rocket/rocket.png"),
                            false,
                            angle,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                        );
                    }
                } else {
                    let angle = self.vel_y.atan2(self.vel_x);
                    let rocket_length = 12.0;
                    let rocket_width = 3.0;

                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    let front_x = screen_x + cos_a * rocket_length * 0.5;
                    let front_y = screen_y + sin_a * rocket_length * 0.5;
                    let back_x = screen_x - cos_a * rocket_length * 0.5;
                    let back_y = screen_y - sin_a * rocket_length * 0.5;

                    draw_line(
                        back_x,
                        back_y,
                        front_x,
                        front_y,
                        rocket_width,
                        Color::from_rgba(80, 80, 80, 255),
                    );
                    draw_line(
                        back_x,
                        back_y,
                        front_x,
                        front_y,
                        rocket_width - 1.0,
                        Color::from_rgba(120, 120, 120, 255),
                    );

                    draw_circle(
                        front_x,
                        front_y,
                        rocket_width * 0.7,
                        Color::from_rgba(200, 100, 50, 255),
                    );
                    draw_circle(
                        back_x,
                        back_y,
                        rocket_width * 0.5,
                        Color::from_rgba(255, 150, 0, 255),
                    );
                }
            }
            Weapon::GrenadeLauncher => {
                use super::constants::GRENADE_FUSE_SECS;

                let has_model = cache
                    .get_model(super::projectile_model_cache::ProjectileModelType::Grenade)
                    .is_some();
                if !has_model {
                    cache.get_or_load_model(
                        super::projectile_model_cache::ProjectileModelType::Grenade,
                    );
                }

                let has_texture = cache
                    .get_texture("q3-resources/models/ammo/grenade.png")
                    .is_some();
                if !has_texture {
                    cache.get_or_load_texture("q3-resources/models/ammo/grenade.png");
                }

                if let Some(model) =
                    cache.get_model(super::projectile_model_cache::ProjectileModelType::Grenade)
                {
                    let texture = cache.get_texture("q3-resources/models/ammo/grenade.png");
                    let angle = self.vel_y.atan2(self.vel_x);

                    for mesh in &model.meshes {
                        super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex(
                            mesh,
                            0,
                            screen_x,
                            screen_y,
                            1.0,
                            WHITE,
                            texture,
                            Some("q3-resources/models/ammo/grenade.png"),
                            false,
                            angle,
                            self.angle * std::f32::consts::PI / 180.0,
                            0.0,
                            0.0,
                            0.0,
                        );
                    }

                    draw_circle(screen_x, screen_y, 8.0, Color::from_rgba(150, 255, 120, 60));

                    if self.life_secs > GRENADE_FUSE_SECS - 0.5 {
                        let blink = ((self.life / 5) % 2) == 0;
                        if blink {
                            draw_circle(screen_x, screen_y, 6.0, Color::from_rgba(255, 0, 0, 180));
                        }
                    }
                } else {
                    let rotation = self.angle;
                    draw_circle(
                        screen_x,
                        screen_y,
                        6.0,
                        Color::from_rgba(120, 200, 100, 180),
                    );
                    draw_circle(
                        screen_x,
                        screen_y,
                        4.5,
                        Color::from_rgba(170, 255, 130, 220),
                    );
                    draw_circle(
                        screen_x,
                        screen_y,
                        3.0,
                        Color::from_rgba(210, 255, 170, 255),
                    );

                    let marker_x = screen_x + rotation.to_radians().cos() * 3.0;
                    let marker_y = screen_y + rotation.to_radians().sin() * 3.0;
                    draw_circle(
                        marker_x,
                        marker_y,
                        1.8,
                        Color::from_rgba(255, 255, 200, 255),
                    );

                    if self.life_secs > GRENADE_FUSE_SECS - 0.5 {
                        let blink = ((self.life / 5) % 2) == 0;
                        if blink {
                            draw_circle(screen_x, screen_y, 6.0, Color::from_rgba(255, 0, 0, 180));
                        }
                    }
                }
            }
            Weapon::Plasmagun => {
                draw_circle(screen_x, screen_y, 6.0, Color::from_rgba(50, 150, 255, 120));
                draw_circle(screen_x, screen_y, 5.0, Color::from_rgba(80, 180, 255, 200));
                draw_circle(
                    screen_x,
                    screen_y,
                    3.5,
                    Color::from_rgba(150, 220, 255, 255),
                );
            }
            Weapon::BFG => {
                draw_circle(screen_x, screen_y, 10.0, Color::from_rgba(50, 255, 50, 100));
                draw_circle(screen_x, screen_y, 8.0, Color::from_rgba(80, 255, 80, 180));
                draw_circle(
                    screen_x,
                    screen_y,
                    6.0,
                    Color::from_rgba(120, 255, 120, 255),
                );
                draw_circle(
                    screen_x,
                    screen_y,
                    4.0,
                    Color::from_rgba(200, 255, 200, 255),
                );
            }
            Weapon::Railgun => {
                draw_line(
                    screen_x - 15.0,
                    screen_y,
                    screen_x + 15.0,
                    screen_y,
                    3.0,
                    Color::from_rgba(255, 255, 255, 200),
                );
                draw_circle(
                    screen_x,
                    screen_y,
                    3.0,
                    Color::from_rgba(200, 255, 255, 255),
                );
            }
            Weapon::Gauntlet | Weapon::MachineGun | Weapon::Shotgun | Weapon::Lightning => {}
        }
    }

    pub fn check_collision(
        &self,
        target_x: f32,
        target_y: f32,
        target_w: f32,
        target_h: f32,
    ) -> bool {
        if !self.active {
            return false;
        }

        let projectile_size = match self.weapon_type {
            Weapon::RocketLauncher => 8.0,
            Weapon::GrenadeLauncher => 10.0,
            Weapon::Plasmagun => 12.0,
            Weapon::BFG => 16.0,
            Weapon::Railgun => 6.0,
            _ => 8.0,
        };

        let rect1_x = self.x - projectile_size / 2.0;
        let rect1_y = self.y - projectile_size / 2.0;
        let rect1_w = projectile_size;
        let rect1_h = projectile_size;

        let rect2_x = target_x - target_w / 2.0;
        let rect2_y = target_y - target_h / 2.0;
        let rect2_w = target_w;
        let rect2_h = target_h;

        rect1_x < rect2_x + rect2_w
            && rect1_x + rect1_w > rect2_x
            && rect1_y < rect2_y + rect2_h
            && rect1_y + rect1_h > rect2_y
    }

    pub fn create_explosion_particles(&self) -> Vec<Particle> {
        let mut particles = Vec::new();

        use crate::compat_rand::*;

        match self.weapon_type {
            Weapon::RocketLauncher => {
                for _ in 0..8 {
                    let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
                    let speed = gen_range_f32(3.0, 8.0);
                    let explosion_particle = Particle::new_explosion(
                        self.x + gen_range_f32(-5.0, 5.0),
                        self.y + gen_range_f32(-5.0, 5.0),
                        angle.cos() * speed,
                        angle.sin() * speed,
                        gen_range_f32(15.0, 25.0),
                    );
                    particles.push(explosion_particle);
                }

                for _ in 0..8 {
                    let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
                    let speed = gen_range_f32(2.0, 6.0);
                    let smoke_particle = Particle::new_smoke(
                        self.x + gen_range_f32(-8.0, 8.0),
                        self.y + gen_range_f32(-8.0, 8.0),
                        angle.cos() * speed,
                        angle.sin() * speed,
                        gen_range_f32(8.0, 16.0),
                        gen_range_f32(1.0, 1.667),
                    );
                    particles.push(smoke_particle);
                }

                for _ in 0..15 {
                    let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
                    let speed = gen_range_f32(4.0, 12.0);
                    let spark = Particle::new(
                        self.x,
                        self.y,
                        angle.cos() * speed,
                        angle.sin() * speed - 1.5,
                        false,
                    );
                    particles.push(spark);
                }
            }
            Weapon::GrenadeLauncher => {
                for _ in 0..6 {
                    let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
                    let speed = gen_range_f32(2.0, 6.0);
                    let explosion_particle = Particle::new_explosion(
                        self.x + gen_range_f32(-3.0, 3.0),
                        self.y + gen_range_f32(-3.0, 3.0),
                        angle.cos() * speed,
                        angle.sin() * speed,
                        gen_range_f32(12.0, 20.0),
                    );
                    particles.push(explosion_particle);
                }

                for _ in 0..30 {
                    let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
                    let speed = gen_range_f32(2.0, 8.0);
                    let spark = Particle::new(
                        self.x,
                        self.y,
                        angle.cos() * speed,
                        angle.sin() * speed - 1.5,
                        true,
                    );
                    particles.push(spark);
                }
            }
            _ => {
                let count = match self.weapon_type {
                    Weapon::BFG => 60,
                    Weapon::Plasmagun => 25,
                    Weapon::Railgun => 15,
                    _ => 0,
                };

                for i in 0..count {
                    let angle = (i as f32 / count as f32) * std::f32::consts::PI * 2.0;
                    let speed = gen_range_f32(2.0, 6.0);
                    let particle = Particle::new(
                        self.x,
                        self.y,
                        angle.cos() * speed,
                        angle.sin() * speed - 1.5,
                        true,
                    );
                    particles.push(particle);
                }
            }
        }

        particles
    }

    pub fn explosion_radius(&self) -> f32 {
        match self.weapon_type {
            Weapon::RocketLauncher => 120.0,
            Weapon::GrenadeLauncher => 150.0,
            Weapon::BFG => 150.0,
            Weapon::Plasmagun => 70.0,
            Weapon::Railgun => 0.0,
            Weapon::Gauntlet => 0.0,
            Weapon::MachineGun => 0.0,
            Weapon::Shotgun => 0.0,
            Weapon::Lightning => 0.0,
        }
    }

    pub fn should_create_trail(&self) -> bool {
        matches!(self.weapon_type, Weapon::RocketLauncher)
    }

    pub fn create_trail_particles(&mut self) -> Vec<Particle> {
        let mut particles = Vec::new();

        if !self.should_create_trail() || !self.active {
            return particles;
        }

        const TRAIL_STEP: u32 = 3;

        if self.trail_time % TRAIL_STEP == 0 {
            use crate::compat_rand::*;

            let smoke_particle = Particle::new_smoke(
                self.x + gen_range_f32(-2.0, 2.0),
                self.y + gen_range_f32(-2.0, 2.0),
                gen_range_f32(-0.5, 0.5),
                gen_range_f32(-0.5, 0.5),
                8.0,
                2.0,
            );
            particles.push(smoke_particle);

            self.last_trail_x = self.x;
            self.last_trail_y = self.y;
        }

        particles
    }

    pub fn check_hit(&self, player_x: f32, player_y: f32) -> bool {
        if !self.active {
            return false;
        }

        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        dist < 20.0
    }
}
