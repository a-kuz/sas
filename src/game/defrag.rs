use serde::{Deserialize, Serialize};
use macroquad::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefragCheckpoint {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub index: usize,
    pub triggered: bool,
}

impl DefragCheckpoint {
    pub fn new(x: f32, y: f32, width: f32, height: f32, index: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            index,
            triggered: false,
        }
    }

    pub fn check_collision(&self, px: f32, py: f32) -> bool {
        px >= self.x && 
        px <= self.x + self.width &&
        py >= self.y && 
        py <= self.y + self.height
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        let time = get_time() as f32;
        let pulse = (time * 2.0 + self.index as f32).sin() * 0.3 + 0.7;
        
        if self.triggered {
            let fade_alpha = (100.0 * pulse) as u8;
            draw_rectangle(
                screen_x,
                screen_y,
                self.width,
                self.height,
                Color::from_rgba(50, 255, 50, fade_alpha),
            );
            draw_rectangle_lines(
                screen_x,
                screen_y,
                self.width,
                self.height,
                2.0,
                Color::from_rgba(100, 255, 100, 180),
            );
        } else {
            let alpha = (200.0 * pulse) as u8;
            draw_rectangle(
                screen_x,
                screen_y,
                self.width,
                self.height,
                Color::from_rgba(100, 200, 255, alpha),
            );
            draw_rectangle_lines(
                screen_x,
                screen_y,
                self.width,
                self.height,
                3.0,
                Color::from_rgba(150, 220, 255, 255),
            );
            
            let text = format!("{}", self.index + 1);
            let text_x = screen_x + self.width / 2.0 - 6.0;
            let text_y = screen_y + self.height / 2.0 + 4.0;
            draw_text(&text, text_x + 1.0, text_y + 1.0, 20.0, BLACK);
            draw_text(&text, text_x, text_y, 20.0, WHITE);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefragFinish {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DefragFinish {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn check_collision(&self, px: f32, py: f32) -> bool {
        px >= self.x && 
        px <= self.x + self.width &&
        py >= self.y && 
        py <= self.y + self.height
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        let time = get_time() as f32;
        let pulse = (time * 3.0).sin() * 0.5 + 0.5;
        
        for i in 0..3 {
            let offset = i as f32 * 6.0;
            let alpha = ((1.0 - i as f32 / 3.0) * 150.0 * pulse) as u8;
            draw_rectangle(
                screen_x - offset,
                screen_y - offset,
                self.width + offset * 2.0,
                self.height + offset * 2.0,
                Color::from_rgba(255, 215, 0, alpha),
            );
        }
        
        draw_rectangle(
            screen_x,
            screen_y,
            self.width,
            self.height,
            Color::from_rgba(255, 215, 0, 200),
        );
        
        draw_rectangle_lines(
            screen_x,
            screen_y,
            self.width,
            self.height,
            4.0,
            Color::from_rgba(255, 255, 100, 255),
        );
        
        let text = "FINISH";
        let text_x = screen_x + self.width / 2.0 - 30.0;
        let text_y = screen_y + self.height / 2.0 + 5.0;
        draw_text(text, text_x + 1.0, text_y + 1.0, 20.0, BLACK);
        draw_text(text, text_x, text_y, 20.0, WHITE);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefragMode {
    pub active: bool,
    pub checkpoints: Vec<DefragCheckpoint>,
    pub finish: DefragFinish,
    #[serde(default)]
    pub current_time: f32,
    #[serde(default)]
    pub best_time: Option<f32>,
    #[serde(default)]
    pub current_checkpoint: usize,
    #[serde(default)]
    pub run_started: bool,
    #[serde(default)]
    pub run_finished: bool,
    #[serde(default)]
    pub finish_time: Option<f32>,
    pub last_checkpoint_pos: (f32, f32),
    pub start_pos: (f32, f32),
}

impl DefragMode {
    pub fn new(checkpoints: Vec<DefragCheckpoint>, finish: DefragFinish, start_pos: (f32, f32)) -> Self {
        Self {
            active: true,
            checkpoints,
            finish,
            current_time: 0.0,
            best_time: None,
            current_checkpoint: 0,
            run_started: false,
            run_finished: false,
            finish_time: None,
            last_checkpoint_pos: start_pos,
            start_pos,
        }
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32) -> Vec<DefragEvent> {
        if !self.active || self.run_finished {
            return Vec::new();
        }

        let mut events = Vec::new();

        if !self.run_started {
            self.run_started = true;
            events.push(DefragEvent::RunStarted);
        }

        if self.run_started {
            self.current_time += dt;
        }

        if self.current_checkpoint < self.checkpoints.len() {
            let checkpoint = &mut self.checkpoints[self.current_checkpoint];
            if !checkpoint.triggered && checkpoint.check_collision(player_x, player_y) {
                checkpoint.triggered = true;
                self.current_checkpoint += 1;
                self.last_checkpoint_pos = (checkpoint.x + checkpoint.width / 2.0, checkpoint.y);
                events.push(DefragEvent::CheckpointReached(checkpoint.index));
            }
        }

        if self.current_checkpoint >= self.checkpoints.len() && 
           self.finish.check_collision(player_x, player_y) {
            self.run_finished = true;
            self.finish_time = Some(self.current_time);
            
            if self.best_time.is_none() || self.current_time < self.best_time.unwrap() {
                self.best_time = Some(self.current_time);
                events.push(DefragEvent::NewRecord(self.current_time));
            } else {
                events.push(DefragEvent::RunFinished(self.current_time));
            }
        }

        events
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.current_checkpoint = 0;
        self.run_started = false;
        self.run_finished = false;
        self.finish_time = None;
        self.last_checkpoint_pos = self.start_pos;
        
        for checkpoint in &mut self.checkpoints {
            checkpoint.triggered = false;
        }
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        for checkpoint in &self.checkpoints {
            checkpoint.render(camera_x, camera_y);
        }
        
        self.finish.render(camera_x, camera_y);
    }

    pub fn render_hud(&self) {
        let screen_w = screen_width();
        
        let time_text = format!("Time: {:.3}s", self.current_time);
        let checkpoint_text = format!("CP: {}/{}", 
            self.current_checkpoint, 
            self.checkpoints.len()
        );
        
        let hud_x = screen_w / 2.0 - 100.0;
        let hud_y = 30.0;
        
        draw_rectangle(hud_x - 10.0, hud_y - 25.0, 220.0, 70.0, Color::from_rgba(0, 0, 0, 150));
        
        draw_text(&time_text, hud_x + 1.0, hud_y + 1.0, 30.0, BLACK);
        draw_text(&time_text, hud_x, hud_y, 30.0, Color::from_rgba(100, 200, 255, 255));
        
        draw_text(&checkpoint_text, hud_x + 1.0, hud_y + 31.0, 20.0, BLACK);
        draw_text(&checkpoint_text, hud_x, hud_y + 30.0, 20.0, WHITE);
        
        if let Some(best) = self.best_time {
            let best_text = format!("Best: {:.3}s", best);
            draw_text(&best_text, hud_x + 1.0, hud_y + 56.0, 16.0, BLACK);
            draw_text(&best_text, hud_x, hud_y + 55.0, 16.0, Color::from_rgba(255, 215, 0, 255));
        }
        
        if self.run_finished {
            let finish_x = screen_w / 2.0 - 150.0;
            let finish_y = screen_height() / 2.0 - 50.0;
            
            draw_rectangle(finish_x - 20.0, finish_y - 40.0, 340.0, 120.0, Color::from_rgba(0, 0, 0, 200));
            
            let finish_text = "RUN COMPLETE!";
            draw_text(finish_text, finish_x + 1.0, finish_y + 1.0, 40.0, BLACK);
            draw_text(finish_text, finish_x, finish_y, 40.0, Color::from_rgba(255, 215, 0, 255));
            
            if let Some(time) = self.finish_time {
                let time_str = format!("Time: {:.3}s", time);
                draw_text(&time_str, finish_x + 40.0, finish_y + 41.0, 30.0, BLACK);
                draw_text(&time_str, finish_x + 40.0, finish_y + 40.0, 30.0, WHITE);
                
                if self.best_time == Some(time) {
                    let record_text = "NEW RECORD!";
                    draw_text(record_text, finish_x + 30.0, finish_y + 71.0, 25.0, BLACK);
                    draw_text(record_text, finish_x + 30.0, finish_y + 70.0, 25.0, Color::from_rgba(255, 100, 100, 255));
                }
            }
        }
        
        let hint_y = screen_height() - 30.0;
        let hint_text = "R - Restart | Backspace - Respawn at checkpoint";
        draw_text(hint_text, 11.0, hint_y + 1.0, 16.0, BLACK);
        draw_text(hint_text, 10.0, hint_y, 16.0, Color::from_rgba(200, 200, 200, 255));
    }

    pub fn get_respawn_position(&self) -> (f32, f32) {
        self.last_checkpoint_pos
    }
}

#[derive(Clone, Debug)]
pub enum DefragEvent {
    RunStarted,
    CheckpointReached(usize),
    RunFinished(f32),
    NewRecord(f32),
}

