use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Trail {
    pub x: f32,
    pub y: f32,
    pub life: u8,
    pub max_life: u8,
    pub color: Color,
    pub size: f32,
}

impl Trail {
    pub fn update(&mut self) -> bool {
        self.life += 1;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let alpha = (1.0 - (self.life as f32 / self.max_life as f32)) * 0.8;
        let size = self.size * (1.0 - (self.life as f32 / self.max_life as f32) * 0.5);

        let mut color = self.color;
        color.a = alpha;

        draw_circle(screen_x, screen_y, size, color);
    }
}
