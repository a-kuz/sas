use macroquad::prelude::*;
use std::collections::VecDeque;

pub struct PredictionDebugRenderer {
    client_trail: VecDeque<(f32, f32)>,
    server_trail: VecDeque<(f32, f32)>,
    prediction_errors: VecDeque<PredictionError>,
    max_trail_length: usize,
}

#[derive(Clone, Debug)]
pub struct PredictionError {
    pub position: (f32, f32),
    pub error_magnitude: f32,
    pub timestamp: f64,
}

impl PredictionDebugRenderer {
    pub fn new() -> Self {
        Self {
            client_trail: VecDeque::new(),
            server_trail: VecDeque::new(),
            prediction_errors: VecDeque::new(),
            max_trail_length: 60,
        }
    }

    pub fn add_client_position(&mut self, x: f32, y: f32) {
        self.client_trail.push_back((x, y));
        if self.client_trail.len() > self.max_trail_length {
            self.client_trail.pop_front();
        }
    }

    pub fn add_server_position(&mut self, x: f32, y: f32) {
        self.server_trail.push_back((x, y));
        if self.server_trail.len() > self.max_trail_length {
            self.server_trail.pop_front();
        }
    }

    pub fn add_prediction_error(&mut self, client_pos: (f32, f32), server_pos: (f32, f32)) {
        let dx = client_pos.0 - server_pos.0;
        let dy = client_pos.1 - server_pos.1;
        let error_magnitude = (dx * dx + dy * dy).sqrt();

        if error_magnitude > 1.0 {
            self.prediction_errors.push_back(PredictionError {
                position: server_pos,
                error_magnitude,
                timestamp: get_time(),
            });

            if self.prediction_errors.len() > 20 {
                self.prediction_errors.pop_front();
            }
        }
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let draw_enabled = crate::cvar::get_cvar_bool("net_drawPrediction");
        if !draw_enabled {
            return;
        }

        self.render_trails(camera_x, camera_y);
        self.render_prediction_errors(camera_x, camera_y);
    }

    fn render_trails(&self, camera_x: f32, camera_y: f32) {
        for (i, (x, y)) in self.client_trail.iter().enumerate() {
            let screen_x = x - camera_x;
            let screen_y = y - camera_y;
            let alpha = (i as f32 / self.client_trail.len() as f32 * 255.0) as u8;
            draw_circle(screen_x, screen_y, 2.0, Color::from_rgba(0, 255, 0, alpha));
        }

        for (i, (x, y)) in self.server_trail.iter().enumerate() {
            let screen_x = x - camera_x;
            let screen_y = y - camera_y;
            let alpha = (i as f32 / self.server_trail.len() as f32 * 255.0) as u8;
            draw_circle(screen_x, screen_y, 3.0, Color::from_rgba(255, 0, 0, alpha));
        }

        if let (Some(client), Some(server)) = (self.client_trail.back(), self.server_trail.back()) {
            let c_screen_x = client.0 - camera_x;
            let c_screen_y = client.1 - camera_y;
            let s_screen_x = server.0 - camera_x;
            let s_screen_y = server.1 - camera_y;

            draw_line(
                c_screen_x,
                c_screen_y,
                s_screen_x,
                s_screen_y,
                2.0,
                Color::from_rgba(255, 255, 0, 150),
            );
        }
    }

    fn render_prediction_errors(&self, camera_x: f32, camera_y: f32) {
        let current_time = get_time();

        for error in &self.prediction_errors {
            let age = (current_time - error.timestamp) as f32;
            if age < 2.0 {
                let screen_x = error.position.0 - camera_x;
                let screen_y = error.position.1 - camera_y;

                let alpha = ((1.0 - age / 2.0) * 255.0) as u8;
                let radius = error.error_magnitude.min(20.0);

                draw_circle_lines(
                    screen_x,
                    screen_y,
                    radius,
                    2.0,
                    Color::from_rgba(255, 0, 255, alpha),
                );

                draw_text(
                    &format!("{:.1}", error.error_magnitude),
                    screen_x + radius,
                    screen_y,
                    12.0,
                    Color::from_rgba(255, 255, 255, alpha),
                );
            }
        }
    }

    pub fn clear(&mut self) {
        self.client_trail.clear();
        self.server_trail.clear();
        self.prediction_errors.clear();
    }
}

impl Default for PredictionDebugRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_collision_debug(x: f32, y: f32, collision_type: &str, camera_x: f32, camera_y: f32) {
    let show_collision = crate::cvar::get_cvar_bool("net_showCollision");
    if !show_collision {
        return;
    }

    let screen_x = x - camera_x;
    let screen_y = y - camera_y;

    let color = match collision_type {
        "wall_x" => Color::from_rgba(255, 0, 0, 200),
        "wall_y" => Color::from_rgba(0, 255, 0, 200),
        "ground" => Color::from_rgba(0, 0, 255, 200),
        "ceiling" => Color::from_rgba(255, 255, 0, 200),
        _ => Color::from_rgba(255, 255, 255, 200),
    };

    draw_circle(screen_x, screen_y, 4.0, color);
    draw_text(collision_type, screen_x + 6.0, screen_y, 10.0, color);
}

pub fn render_physics_state_hud(
    position: (f32, f32),
    velocity: (f32, f32),
    on_ground: bool,
    frame_count: u32,
) {
    let show_physics = crate::cvar::get_cvar_bool("net_showPhysics");
    if !show_physics {
        return;
    }

    let y_offset = 100.0;
    let line_height = 18.0;

    draw_text(
        "=== PHYSICS DEBUG ===",
        10.0,
        y_offset,
        16.0,
        Color::from_rgba(255, 255, 0, 255),
    );

    draw_text(
        &format!("Pos: ({:.1}, {:.1})", position.0, position.1),
        10.0,
        y_offset + line_height,
        14.0,
        WHITE,
    );

    draw_text(
        &format!("Vel: ({:.2}, {:.2})", velocity.0, velocity.1),
        10.0,
        y_offset + line_height * 2.0,
        14.0,
        WHITE,
    );

    let vel_magnitude = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
    draw_text(
        &format!("Speed: {:.2}", vel_magnitude),
        10.0,
        y_offset + line_height * 3.0,
        14.0,
        WHITE,
    );

    let ground_text = if on_ground { "YES" } else { "NO" };
    let ground_color = if on_ground {
        Color::from_rgba(0, 255, 0, 255)
    } else {
        Color::from_rgba(255, 0, 0, 255)
    };

    draw_text(
        &format!("On Ground: {}", ground_text),
        10.0,
        y_offset + line_height * 4.0,
        14.0,
        ground_color,
    );

    draw_text(
        &format!("Frame: {}", frame_count),
        10.0,
        y_offset + line_height * 5.0,
        14.0,
        Color::from_rgba(200, 200, 200, 255),
    );
}
