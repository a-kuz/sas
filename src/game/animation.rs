use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimState {
    Idle,
    Walk,
    Run,
    Jump,
    Attack,
}

#[derive(Clone, Debug)]
pub struct PlayerAnimation {
    pub state: AnimState,
    pub frame: u8,
    pub torso_frame: u8,
    pub legs_frame: u8,
}

impl PlayerAnimation {
    pub fn new() -> Self {
        Self {
            state: AnimState::Idle,
            frame: 0,
            torso_frame: 0,
            legs_frame: 0,
        }
    }

    pub fn update(&mut self, on_ground: bool, moving: bool, shooting: bool, speed: f32) {
        self.frame = (self.frame + 1) % 60;

        let new_state = if !on_ground {
            AnimState::Jump
        } else if shooting {
            AnimState::Attack
        } else if moving && speed > 2.5 {
            AnimState::Run
        } else if moving {
            AnimState::Walk
        } else {
            AnimState::Idle
        };

        if new_state != self.state {
            self.state = new_state;
            match new_state {
                AnimState::Attack => self.torso_frame = 0,
                AnimState::Jump => self.legs_frame = 0,
                AnimState::Walk => self.legs_frame = 0,
                AnimState::Run => self.legs_frame = 0,
                AnimState::Idle => {}
            }
        }

        match self.state {
            AnimState::Walk => {
                if self.frame % 6 == 0 {
                    self.legs_frame = (self.legs_frame + 1) % 8;
                }
            }
            AnimState::Run => {
                if self.frame % 3 == 0 {
                    self.legs_frame = (self.legs_frame + 1) % 10;
                }
            }
            AnimState::Attack => {
                if self.frame % 2 == 0 && self.torso_frame < 4 {
                    self.torso_frame += 1;
                }
            }
            AnimState::Jump => {
                if self.legs_frame < 3 {
                    self.legs_frame += 1;
                }
            }
            AnimState::Idle => {}
        }
    }
}
