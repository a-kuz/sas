use macroquad::prelude::*;

pub struct DamageNumber {
    pub player_id: u32,
    pub target_id: u16,
    pub x: f32,
    pub y: f32,
    pub damage: i32,
    pub lifetime: f32,
    pub alpha: f32,
    pub scale: f32,
    pub target_health: i32,
    pub target_armor: i32,
}

impl DamageNumber {
    pub fn new(
        player_id: u32,
        target_id: u16,
        x: f32,
        y: f32,
        damage: i32,
        target_health: i32,
        target_armor: i32,
    ) -> Self {
        Self {
            player_id,
            target_id,
            x,
            y: y - 50.0,
            damage,
            lifetime: 0.0,
            alpha: 1.0,
            scale: 1.4,
            target_health,
            target_armor,
        }
    }

    pub fn add_damage(
        &mut self,
        additional_damage: i32,
        target_health: i32,
        target_armor: i32,
        new_x: f32,
        new_y: f32,
    ) {
        self.damage += additional_damage;
        self.lifetime = 0.0;
        self.scale = 1.5;
        self.target_health = target_health;
        self.target_armor = target_armor;
        self.x = new_x;
        self.y = new_y - 50.0;
    }

    pub fn update(&mut self) -> bool {
        const MAX_LIFETIME: f32 = 3.0;
        const FADE_START: f32 = 1.8;
        const FLOAT_SPEED: f32 = 30.0;

        self.lifetime += get_frame_time();

        self.y -= FLOAT_SPEED * get_frame_time();

        if self.scale > 1.0 {
            self.scale -= get_frame_time() * 3.0;
            if self.scale < 1.0 {
                self.scale = 1.0;
            }
        }

        if self.lifetime < FADE_START {
            self.alpha = 1.0;
        } else {
            let fade_progress = (self.lifetime - FADE_START) / (MAX_LIFETIME - FADE_START);
            self.alpha = 1.0 - fade_progress.min(1.0);
        }

        self.lifetime < MAX_LIFETIME
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let color = if self.target_health < 20 {
            Color::from_rgba(255, 50, 50, (self.alpha * 255.0) as u8)
        } else if self.target_armor > 20 {
            Color::from_rgba(100, 149, 237, (self.alpha * 255.0) as u8)
        } else if self.target_health > 100 {
            Color::from_rgba(100, 149, 237, (self.alpha * 255.0) as u8)
        } else {
            Color::from_rgba(255, 255, 255, (self.alpha * 255.0) as u8)
        };

        let text = format!("{}", self.damage);
        let base_font_size = if self.damage >= 100 {
            40.0
        } else if self.damage >= 50 {
            36.0
        } else {
            32.0
        };
        let font_size = base_font_size * self.scale;

        let text_dims = measure_text(&text, None, font_size as u16, 1.0);
        let text_x = screen_x - text_dims.width / 2.0;
        let text_y = screen_y;

        let outline_color = Color::from_rgba(0, 0, 0, (self.alpha * 220.0) as u8);
        for dx in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            for dy in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                if dx != 0.0 || dy != 0.0 {
                    draw_text(&text, text_x + dx, text_y + dy, font_size, outline_color);
                }
            }
        }

        draw_text(&text, text_x, text_y, font_size, color);
    }
}
