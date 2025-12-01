use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct BloodDroplet {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub amount: f32,
    pub life: f32,
    pub max_life: f32,
}

impl BloodDroplet {
    pub fn new(x: f32, y: f32, amount: f32, vel_x: f32, vel_y: f32) -> Self {
        Self {
            x,
            y,
            vel_x,
            vel_y,
            amount,
            life: 0.0,
            max_life: 10.0, // Blood droplets live for 10 seconds
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life += dt;
        if self.life >= self.max_life {
            return false; // Remove droplet
        }

        // Apply gravity
        self.vel_y += 0.1 * dt;

        // Update position
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;

        // Apply friction/damping
        self.vel_x *= 0.98;
        self.vel_y *= 0.98;

        true
    }
}

pub struct LiquidBloodManager {
    droplets: Vec<BloodDroplet>,
    blood_texture: Option<Texture2D>,
    blood_render_target: Option<RenderTarget>,
    pub texture_size: u32,
}

impl LiquidBloodManager {
    pub fn new() -> Self {
        Self {
            droplets: Vec::new(),
            blood_texture: None,
            blood_render_target: None,
            texture_size: 512, // Blood buffer resolution
        }
    }

    pub fn add_blood(&mut self, x: f32, y: f32, amount: f32, vel_x: f32, vel_y: f32) {
        self.droplets
            .push(BloodDroplet::new(x, y, amount, vel_x, vel_y));
    }

    pub fn update(&mut self, dt: f32, map_width: f32, map_height: f32) {
        // Update droplets
        self.droplets.retain_mut(|droplet| droplet.update(dt));

        // Initialize render target if needed
        if self.blood_render_target.is_none() {
            self.blood_render_target = Some(render_target(
                self.texture_size as u32,
                self.texture_size as u32,
            ));
            self.blood_texture = self
                .blood_render_target
                .as_ref()
                .map(|rt| rt.texture.clone());
        }

        // Render blood buffer
        if let Some(render_target) = &self.blood_render_target {
            set_camera(&Camera2D {
                render_target: Some(render_target.clone()),
                zoom: vec2(2.0 / map_width, 2.0 / map_height),
                target: vec2(map_width / 2.0, map_height / 2.0),
                ..Default::default()
            });

            clear_background(Color::new(0.0, 0.0, 0.0, 0.0)); // Clear with transparent

            // Render droplets as points
            for droplet in &self.droplets {
                let alpha = 1.0 - (droplet.life / droplet.max_life);
                let color = Color::new(droplet.amount * alpha, 0.0, 0.0, alpha);
                draw_circle(droplet.x, droplet.y, droplet.amount * 2.0, color);
            }

            set_default_camera();
        }
    }

    pub fn get_blood_texture(&self) -> Option<Texture2D> {
        self.blood_texture.clone()
    }

    pub fn clear(&mut self) {
        self.droplets.clear();
    }
}
