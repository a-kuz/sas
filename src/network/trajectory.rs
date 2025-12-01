use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrajectoryType {
    Stationary,
    Linear,
    Gravity,
    Interpolate,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trajectory {
    pub tr_type: TrajectoryType,
    pub tr_time: u32,
    pub tr_base_x: f32,
    pub tr_base_y: f32,
    pub tr_delta_x: f32,
    pub tr_delta_y: f32,
}

impl Trajectory {
    pub fn new_stationary(x: f32, y: f32, time: u32) -> Self {
        Self {
            tr_type: TrajectoryType::Stationary,
            tr_time: time,
            tr_base_x: x,
            tr_base_y: y,
            tr_delta_x: 0.0,
            tr_delta_y: 0.0,
        }
    }

    pub fn new_linear(x: f32, y: f32, vel_x: f32, vel_y: f32, time: u32) -> Self {
        Self {
            tr_type: TrajectoryType::Linear,
            tr_time: time,
            tr_base_x: x,
            tr_base_y: y,
            tr_delta_x: vel_x,
            tr_delta_y: vel_y,
        }
    }

    pub fn new_gravity(x: f32, y: f32, vel_x: f32, vel_y: f32, time: u32) -> Self {
        Self {
            tr_type: TrajectoryType::Gravity,
            tr_time: time,
            tr_base_x: x,
            tr_base_y: y,
            tr_delta_x: vel_x,
            tr_delta_y: vel_y,
        }
    }

    pub fn evaluate(&self, at_time: u32) -> (f32, f32) {
        let delta_time = at_time.saturating_sub(self.tr_time) as f32 / 1000.0;

        match self.tr_type {
            TrajectoryType::Stationary | TrajectoryType::Interpolate => {
                (self.tr_base_x, self.tr_base_y)
            }
            TrajectoryType::Linear => (
                self.tr_base_x + self.tr_delta_x * delta_time,
                self.tr_base_y + self.tr_delta_y * delta_time,
            ),
            TrajectoryType::Gravity => {
                const GRAVITY: f32 = 0.25 * 60.0;
                (
                    self.tr_base_x + self.tr_delta_x * delta_time,
                    self.tr_base_y
                        + self.tr_delta_y * delta_time
                        + 0.5 * GRAVITY * delta_time * delta_time,
                )
            }
        }
    }

    pub fn evaluate_velocity(&self, at_time: u32) -> (f32, f32) {
        let delta_time = at_time.saturating_sub(self.tr_time) as f32 / 1000.0;

        match self.tr_type {
            TrajectoryType::Stationary | TrajectoryType::Interpolate => (0.0, 0.0),
            TrajectoryType::Linear => (self.tr_delta_x, self.tr_delta_y),
            TrajectoryType::Gravity => {
                const GRAVITY: f32 = 0.25 * 60.0;
                (self.tr_delta_x, self.tr_delta_y + GRAVITY * delta_time)
            }
        }
    }
}

impl Default for Trajectory {
    fn default() -> Self {
        Self::new_stationary(0.0, 0.0, 0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectileTrajectory {
    pub id: u32,
    pub trajectory: Trajectory,
    pub weapon_type: u8,
    pub owner_id: u16,
    pub spawn_time: u32,
}

impl ProjectileTrajectory {
    pub fn new(
        id: u32,
        x: f32,
        y: f32,
        vel_x: f32,
        vel_y: f32,
        weapon_type: u8,
        owner_id: u16,
        spawn_time: u32,
    ) -> Self {
        let trajectory = match weapon_type {
            4 => Trajectory::new_linear(x, y, vel_x, vel_y, spawn_time),
            3 => Trajectory::new_gravity(x, y, vel_x, vel_y, spawn_time),
            7 => Trajectory::new_linear(x, y, vel_x, vel_y, spawn_time),
            8 => Trajectory::new_linear(x, y, vel_x, vel_y, spawn_time),
            _ => Trajectory::new_linear(x, y, vel_x, vel_y, spawn_time),
        };

        Self {
            id,
            trajectory,
            weapon_type,
            owner_id,
            spawn_time,
        }
    }

    pub fn get_position(&self, current_time: u32) -> (f32, f32) {
        self.trajectory.evaluate(current_time)
    }

    pub fn get_velocity(&self, current_time: u32) -> (f32, f32) {
        self.trajectory.evaluate_velocity(current_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stationary_trajectory() {
        let traj = Trajectory::new_stationary(100.0, 200.0, 1000);
        let pos = traj.evaluate(2000);
        assert_eq!(pos, (100.0, 200.0));
    }

    #[test]
    fn test_linear_trajectory() {
        let traj = Trajectory::new_linear(100.0, 200.0, 10.0, 20.0, 1000);
        let pos = traj.evaluate(2000);
        assert_eq!(pos, (110.0, 220.0));
    }

    #[test]
    fn test_gravity_trajectory() {
        let traj = Trajectory::new_gravity(100.0, 200.0, 10.0, -5.0, 1000);
        let pos = traj.evaluate(2000);

        assert_eq!(pos.0, 110.0);
        assert!(pos.1 > 200.0);
    }

    #[test]
    fn test_projectile_trajectory_rocket() {
        let proj = ProjectileTrajectory::new(1, 100.0, 200.0, 15.0, 0.0, 4, 1, 1000);

        assert_eq!(proj.trajectory.tr_type, TrajectoryType::Linear);

        let pos = proj.get_position(2000);
        assert_eq!(pos.0, 115.0);
    }

    #[test]
    fn test_projectile_trajectory_grenade() {
        let proj = ProjectileTrajectory::new(1, 100.0, 200.0, 10.0, -5.0, 3, 1, 1000);

        assert_eq!(proj.trajectory.tr_type, TrajectoryType::Gravity);

        let pos = proj.get_position(1000);
        assert_eq!(pos, (100.0, 200.0));

        let pos_later = proj.get_position(2000);
        assert!(pos_later.1 > 200.0);
    }
}
