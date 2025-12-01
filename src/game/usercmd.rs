#[derive(Clone, Copy, Debug)]
pub struct UserCmd {
    pub right: f32,
    pub buttons: u8,
    pub angles: (f32, f32),
    pub server_time: u32,
}

pub const BUTTON_ATTACK: u8 = 1;
pub const BUTTON_JUMP: u8 = 2;
pub const BUTTON_CROUCH: u8 = 4;

impl UserCmd {
    pub fn new() -> Self {
        Self {
            right: 0.0,
            buttons: 0,
            angles: (0.0, 0.0),
            server_time: 0,
        }
    }

    pub fn from_input(
        input: &crate::input::Input,
        _screen_width: f32,
        _screen_height: f32,
    ) -> Self {
        let mut cmd = Self::new();

        if input.move_left {
            cmd.right -= 1.0;
        }
        if input.move_right {
            cmd.right += 1.0;
        }

        if input.jump {
            cmd.buttons |= BUTTON_JUMP;
        }
        if input.crouch {
            cmd.buttons |= BUTTON_CROUCH;
        }
        if input.shoot {
            cmd.buttons |= BUTTON_ATTACK;
        }

        let angle = input.aim_angle;
        cmd.angles = (angle, 0.0);

        cmd
    }

    pub fn from_player_input(
        input: &crate::input::PlayerInput,
        _player_x: f32,
        _player_y: f32,
    ) -> Self {
        let mut cmd = Self::new();

        if input.move_left {
            cmd.right -= 1.0;
        }
        if input.move_right {
            cmd.right += 1.0;
        }

        if input.jump {
            cmd.buttons |= BUTTON_JUMP;
        }
        if input.crouch {
            cmd.buttons |= BUTTON_CROUCH;
        }
        if input.shoot {
            cmd.buttons |= BUTTON_ATTACK;
        }

        let angle = input.aim_angle;
        cmd.angles = (angle, 0.0);

        cmd
    }
}
