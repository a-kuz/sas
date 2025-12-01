use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct GameMessage {
    pub text: String,
    pub life: u32,
    pub color: Color,
}

impl GameMessage {
    pub fn new(text: String, color: Color) -> Self {
        Self {
            text,
            life: 0,
            color,
        }
    }

    pub fn kill_message(killer: &str, victim: &str, weapon: u8) -> Self {
        let weapon_text = match weapon {
            0 => "pummeled",
            1 => "machinegunned",
            2 => "gunned down",
            3 => "shredded by shrapnel from",
            4 => "was blasted by",
            5 => "electrocuted by",
            6 => "railed by",
            7 => "melted by plasmagun of",
            8 => "blasted by BFG of",
            _ => "killed by",
        };

        let text = format!("{} was {} {}", victim, weapon_text, killer);
        Self::new(text, Color::from_rgba(255, 255, 100, 255))
    }

    pub fn update(&mut self) -> bool {
        self.life += 1;
        self.life < 250
    }

    pub fn render(&self, y_offset: f32) {
        let alpha = if self.life < 50 {
            1.0
        } else {
            1.0 - ((self.life - 50) as f32 / 200.0)
        };

        let mut color = self.color;
        color.a = alpha;

        draw_text(&self.text, 20.0, 100.0 + y_offset, 18.0, color);
    }
}
