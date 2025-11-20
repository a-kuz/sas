use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Teleport {
    pub x: f32,
    pub y: f32,
    pub dest_x: f32,
    pub dest_y: f32,
    pub cooldown: u8,
}

impl Teleport {
    pub fn update(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
    }

    pub fn can_teleport(&self) -> bool {
        self.cooldown == 0
    }

    pub fn activate(&mut self) {
        self.cooldown = 20;
    }

    pub fn check_collision(&self, px: f32, py: f32) -> bool {
        let dx = self.x - px;
        let dy = self.y - py;
        (dx * dx + dy * dy).sqrt() < 20.0
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let time = get_time() as f32;

        let portal_width = 48.0;
        let portal_height = 80.0;
        let base_y = screen_y - portal_height / 2.0;
        let left = screen_x - portal_width / 2.0;
        let right = screen_x + portal_width / 2.0;

        // Side emitters (yellow/orange frames)
        let side_w = 8.0;
        draw_rectangle(left - 6.0, base_y, side_w, portal_height, Color::from_rgba(210, 170, 60, 220));
        draw_rectangle(right - side_w + 6.0, base_y, side_w, portal_height, Color::from_rgba(210, 170, 60, 220));
        for i in 0..5 {
            let a = 200 - i * 28;
            draw_rectangle(left - 6.0 + i as f32, base_y, 1.0, portal_height, Color::from_rgba(255, 220, 100, a as u8));
            draw_rectangle(right + 6.0 - i as f32, base_y, 1.0, portal_height, Color::from_rgba(255, 220, 100, a as u8));
        }

        // Inner portal body (dark blue)
        draw_rectangle(
            left + 6.0,
            base_y,
            portal_width - 12.0,
            portal_height,
            Color::from_rgba(24, 48, 64, 230),
        );

        // Central horizontal bars moving upward
        let bar_gap = 10.0;
        let offset = (time * 40.0) % bar_gap; // pixels per second
        let inner_left = left + 12.0;
        let inner_right = right - 12.0;
        let bar_width = inner_right - inner_left;
        let mut y = base_y + portal_height - offset;
        while y >= base_y {
            draw_rectangle(inner_left, y - 2.0, bar_width, 4.0, Color::from_rgba(220, 230, 240, 210));
            draw_rectangle(inner_left, y - 1.0, bar_width, 2.0, Color::from_rgba(255, 255, 255, 230));
            y -= bar_gap;
        }

        // Core glow
        let glow = (time * 2.2).sin() * 0.25 + 0.75;
        draw_rectangle(
            inner_left,
            base_y,
            bar_width,
            portal_height,
            Color::from_rgba(100, 150, 220, (25.0 * glow) as u8),
        );
    }
}

