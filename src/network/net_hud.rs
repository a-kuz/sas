use super::client_prediction::PredictionError;
use super::net_stats::NetStats;
use macroquad::prelude::*;

pub struct NetHud {
    pub show_stats: bool,
    pub show_graph: bool,
    pub show_prediction_errors: bool,
}

impl NetHud {
    pub fn new() -> Self {
        Self {
            show_stats: false,
            show_graph: false,
            show_prediction_errors: false,
        }
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn toggle_graph(&mut self) {
        self.show_graph = !self.show_graph;
    }

    pub fn toggle_prediction_errors(&mut self) {
        self.show_prediction_errors = !self.show_prediction_errors;
    }

    pub fn render(&self, stats: &NetStats, prediction_error: Option<&PredictionError>) {
        if self.show_stats {
            self.render_stats(stats);
        }

        if self.show_graph {
            self.render_graph(stats);
        }

        if self.show_prediction_errors {
            self.render_prediction_errors(prediction_error);
        }
    }

    fn render_stats(&self, stats: &NetStats) {
        let x = screen_width() - 320.0;
        let y = 10.0;

        draw_rectangle(
            x - 5.0,
            y - 5.0,
            310.0,
            150.0,
            Color::from_rgba(0, 0, 0, 180),
        );

        let text_color = WHITE;
        let mut line_y = y;

        draw_text(
            &format!("Ping: {} ms", stats.ping),
            x,
            line_y,
            20.0,
            text_color,
        );
        line_y += 20.0;

        draw_text(
            &format!("Packet Loss: {:.1}%", stats.packet_loss),
            x,
            line_y,
            20.0,
            text_color,
        );
        line_y += 20.0;

        draw_text(
            &format!("Incoming: {} B/s", stats.incoming_rate),
            x,
            line_y,
            20.0,
            text_color,
        );
        line_y += 20.0;

        draw_text(
            &format!("Outgoing: {} B/s", stats.outgoing_rate),
            x,
            line_y,
            20.0,
            text_color,
        );
        line_y += 20.0;

        draw_text(
            &format!("Snapshots: {}/s", stats.snapshot_rate),
            x,
            line_y,
            20.0,
            text_color,
        );
        line_y += 20.0;

        draw_text(
            &format!("Pred Errors: {}", stats.prediction_errors),
            x,
            line_y,
            20.0,
            text_color,
        );
        line_y += 20.0;

        draw_text(
            &format!("Extraps: {}", stats.extrapolations),
            x,
            line_y,
            20.0,
            text_color,
        );
    }

    fn render_graph(&self, stats: &NetStats) {
        let x = screen_width() - 320.0;
        let y = 170.0;
        let width = 300.0;
        let height = 100.0;

        draw_rectangle(
            x - 5.0,
            y - 5.0,
            width + 10.0,
            height + 10.0,
            Color::from_rgba(0, 0, 0, 180),
        );

        draw_line(x, y + height, x + width, y + height, 1.0, WHITE);
        draw_line(x, y, x, y + height, 1.0, WHITE);

        let ping_data = stats.get_ping_graph_data();

        if ping_data.len() > 1 {
            let max_ping = ping_data.iter().max().unwrap_or(&100);
            let scale = height / (*max_ping as f32).max(100.0);

            for i in 0..ping_data.len().saturating_sub(1) {
                let x1 = x + (i as f32 / ping_data.len() as f32) * width;
                let y1 = y + height - (ping_data[i] as f32 * scale);

                let x2 = x + ((i + 1) as f32 / ping_data.len() as f32) * width;
                let y2 = y + height - (ping_data[i + 1] as f32 * scale);

                let color = if ping_data[i] < 50 {
                    GREEN
                } else if ping_data[i] < 100 {
                    YELLOW
                } else {
                    RED
                };

                draw_line(x1, y1, x2, y2, 2.0, color);
            }
        }

        draw_text(
            &format!(
                "Ping Graph ({}ms max)",
                ping_data.iter().max().unwrap_or(&0)
            ),
            x,
            y - 10.0,
            16.0,
            WHITE,
        );
    }

    fn render_prediction_errors(&self, error: Option<&PredictionError>) {
        if let Some(err) = error {
            let x = screen_width() / 2.0 - 150.0;
            let y = screen_height() - 50.0;

            draw_rectangle(
                x - 5.0,
                y - 25.0,
                310.0,
                40.0,
                Color::from_rgba(0, 0, 0, 180),
            );

            let color = if err.magnitude > 10.0 {
                RED
            } else if err.magnitude > 5.0 {
                YELLOW
            } else {
                GREEN
            };

            draw_text(
                &format!(
                    "Prediction Error: {:.2}px (x:{:.2}, y:{:.2})",
                    err.magnitude, err.error_x, err.error_y
                ),
                x,
                y,
                20.0,
                color,
            );
        }
    }
}

impl Default for NetHud {
    fn default() -> Self {
        Self::new()
    }
}
