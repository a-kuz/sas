use super::prediction::UserCommand;
use super::PlayerState;
use crate::game::bg_pmove::{pmove, PmoveCmd, PmoveState};
use crate::game::map::Map;

#[derive(Clone, Debug)]
pub struct PredictedPlayerState {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub angle: f32,
    pub was_in_air: bool,
    pub command_time: u32,
    pub hit_jumppad: bool,
    pub landed: bool,
}

#[derive(Clone, Debug)]
pub struct PredictionError {
    pub error_x: f32,
    pub error_y: f32,
    pub error_time: f64,
    pub magnitude: f32,
}

pub struct ClientPrediction {
    predicted_state: Option<PredictedPlayerState>,
    prediction_error: Option<PredictionError>,
    error_decay_rate: f32,
    last_server_state: Option<PlayerState>,
}

impl ClientPrediction {
    pub fn new() -> Self {
        Self {
            predicted_state: None,
            prediction_error: None,
            error_decay_rate: 0.1,
            last_server_state: None,
        }
    }

    pub fn predict_player_movement(
        &mut self,
        base_snapshot: &PlayerState,
        commands: &[UserCommand],
        map: &Map,
        current_time: u32,
    ) -> PredictedPlayerState {
        let mut pmove_state = PmoveState {
            x: base_snapshot.position.0,
            y: base_snapshot.position.1,
            vel_x: base_snapshot.velocity.0,
            vel_y: base_snapshot.velocity.1,
            was_in_air: !base_snapshot.on_ground,
        };

        let mut last_angle = base_snapshot.angle;
        let mut last_cmd_time = base_snapshot.command_time;
        let mut hit_jumppad = false;
        let mut landed = false;

        let mut last_input = PmoveCmd {
            move_right: 0.0,
            jump: false,
            crouch: false,
            haste_active: false,
        };
        for cmd in commands {
            if cmd.server_time <= base_snapshot.command_time {
                continue;
            }

            if cmd.server_time > current_time {
                break;
            }

            let pmove_cmd = PmoveCmd {
                move_right: cmd.move_right,
                jump: (cmd.buttons & 2) != 0,
                crouch: (cmd.buttons & 4) != 0,
                haste_active: false,
            };

            let dt_ms = (cmd.server_time.saturating_sub(last_cmd_time)).clamp(1, 100) as f32;
            let dt = dt_ms * 0.001;
            let result = pmove(&pmove_state, &pmove_cmd, dt, map);

            if result.hit_jumppad {
                hit_jumppad = true;
            }
            if result.landed {
                landed = true;
            }

            pmove_state = PmoveState {
                x: result.new_x,
                y: result.new_y,
                vel_x: result.new_vel_x,
                vel_y: result.new_vel_y,
                was_in_air: result.new_was_in_air,
            };

            last_angle = cmd.angles;
            last_cmd_time = cmd.server_time;
            last_input = pmove_cmd;
        }

        if last_cmd_time < current_time {
            let dt_ms = (current_time - last_cmd_time).clamp(1, 100) as f32;
            let dt = dt_ms * 0.001;
            let result = pmove(&pmove_state, &last_input, dt, map);

            if result.hit_jumppad {
                hit_jumppad = true;
            }
            if result.landed {
                landed = true;
            }

            pmove_state = PmoveState {
                x: result.new_x,
                y: result.new_y,
                vel_x: result.new_vel_x,
                vel_y: result.new_vel_y,
                was_in_air: result.new_was_in_air,
            };
        }

        let predicted = PredictedPlayerState {
            x: pmove_state.x,
            y: pmove_state.y,
            vel_x: pmove_state.vel_x,
            vel_y: pmove_state.vel_y,
            angle: last_angle,
            was_in_air: pmove_state.was_in_air,
            command_time: last_cmd_time,
            hit_jumppad,
            landed,
        };

        self.predicted_state = Some(predicted.clone());
        predicted
    }

    pub fn check_prediction_error(
        &mut self,
        predicted: &PredictedPlayerState,
        server_state: &PlayerState,
    ) -> Option<PredictionError> {
        let error_x = server_state.position.0 - predicted.x;
        let error_y = server_state.position.1 - predicted.y;
        let magnitude = (error_x * error_x + error_y * error_y).sqrt();

        if magnitude > 2.0 {
            let error = PredictionError {
                error_x,
                error_y,
                error_time: super::get_network_time(),
                magnitude,
            };

            self.prediction_error = Some(error.clone());
            self.last_server_state = Some(server_state.clone());

            Some(error)
        } else {
            if magnitude > 0.1 {
                self.prediction_error = Some(PredictionError {
                    error_x,
                    error_y,
                    error_time: super::get_network_time(),
                    magnitude,
                });
            }
            None
        }
    }

    pub fn apply_error_correction(&mut self, dt: f32) -> Option<(f32, f32)> {
        if let Some(ref error) = self.prediction_error {
            let decay_factor = (self.error_decay_rate * dt * 60.0).min(1.0);

            let correction_x = error.error_x * decay_factor;
            let correction_y = error.error_y * decay_factor;

            let remaining_error = PredictionError {
                error_x: error.error_x - correction_x,
                error_y: error.error_y - correction_y,
                error_time: error.error_time,
                magnitude: error.magnitude * (1.0 - decay_factor),
            };

            if remaining_error.magnitude < 0.1 {
                self.prediction_error = None;
            } else {
                self.prediction_error = Some(remaining_error);
            }

            Some((correction_x, correction_y))
        } else {
            None
        }
    }

    pub fn get_predicted_state(&self) -> Option<&PredictedPlayerState> {
        self.predicted_state.as_ref()
    }

    pub fn get_prediction_error(&self) -> Option<&PredictionError> {
        self.prediction_error.as_ref()
    }

    pub fn clear_prediction_error(&mut self) {
        self.prediction_error = None;
    }
}

impl Default for ClientPrediction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::Map;

    #[test]
    fn test_prediction_no_movement() {
        let mut prediction = ClientPrediction::new();
        let map = Map::new("test");

        let base_state = PlayerState {
            player_id: 1,
            position: (100.0, 100.0),
            velocity: (0.0, 0.0),
            on_ground: true,
            ..Default::default()
        };

        let predicted = prediction.predict_player_movement(&base_state, &[], &map, 1000);

        assert!((predicted.x - 100.0).abs() < 0.1);
        assert!((predicted.y - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_prediction_with_movement() {
        let mut prediction = ClientPrediction::new();
        let map = Map::new("test");

        let base_state = PlayerState {
            player_id: 1,
            position: (100.0, 100.0),
            velocity: (0.0, 0.0),
            on_ground: true,
            ..Default::default()
        };

        let mut commands = Vec::new();
        for i in 0..10 {
            commands.push(UserCommand {
                server_time: i * 16,
                sequence: i as u32,
                move_forward: 0.0,
                move_right: 1.0,
                buttons: 0,
                angles: 0.0,
            });
        }

        let predicted = prediction.predict_player_movement(&base_state, &commands, &map, 160);

        assert!(predicted.x > 100.0, "Player should have moved right");
    }

    #[test]
    fn test_prediction_error_detection() {
        let mut prediction = ClientPrediction::new();

        let predicted = PredictedPlayerState {
            x: 100.0,
            y: 100.0,
            vel_x: 5.0,
            vel_y: 0.0,
            angle: 0.0,
            was_in_air: false,
            command_time: 1000,
            hit_jumppad: false,
            landed: false,
        };

        let server_state = PlayerState {
            player_id: 1,
            position: (110.0, 100.0),
            velocity: (5.0, 0.0),
            ..Default::default()
        };

        let error = prediction.check_prediction_error(&predicted, &server_state);

        assert!(error.is_some());
        let err = error.unwrap();
        assert!((err.magnitude - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_error_correction_decay() {
        let mut prediction = ClientPrediction::new();

        prediction.prediction_error = Some(PredictionError {
            error_x: 10.0,
            error_y: 0.0,
            error_time: super::super::get_network_time(),
            magnitude: 10.0,
        });

        let correction = prediction.apply_error_correction(0.016);
        assert!(correction.is_some());

        let (corr_x, _corr_y) = correction.unwrap();
        assert!(corr_x > 0.0 && corr_x < 10.0);

        assert!(prediction.prediction_error.is_some());
        let remaining = prediction.prediction_error.as_ref().unwrap();
        assert!(remaining.magnitude < 10.0);
    }
}
