use macroquad::prelude::*;
use std::collections::HashSet;

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWarpMouseCursorPosition(point: CGPoint);
    fn CGAssociateMouseAndMouseCursorPosition(connected: u8);
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
pub fn center_mouse_cursor() {
    unsafe {
        let screen_w = screen_width() as f64;
        let screen_h = screen_height() as f64;
        let center = CGPoint {
            x: screen_w / 2.0,
            y: screen_h / 2.0,
        };
        CGWarpMouseCursorPosition(center);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn center_mouse_cursor() {}

#[derive(Clone, Debug)]
pub struct Input {
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub crouch: bool,
    pub shoot: bool,
    pub aim_angle: f32,
    pub aim_x: f32,
    pub aim_y: f32,
    pub weapon_switch: Option<u8>,
    keys_held: HashSet<KeyCode>,
}

#[derive(Clone, Debug)]
pub struct LocalMultiplayerInput {
    pub player1: PlayerInput,
    pub player2: PlayerInput,
}

#[derive(Clone, Debug)]
pub struct PlayerInput {
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub crouch: bool,
    pub shoot: bool,
    pub aim_angle: f32,
    pub aim_x: f32,
    pub aim_y: f32,
    pub weapon_switch: Option<u8>,
    pub flip_x: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            move_left: false,
            move_right: false,
            jump: false,
            crouch: false,
            shoot: false,
            aim_angle: 0.0,
            aim_x: 1.0,
            aim_y: 0.0,
            weapon_switch: None,
            keys_held: HashSet::new(),
        }
    }

    pub fn update(&mut self, ignore_mouse_delta: bool) {
        let pressed = get_keys_pressed();
        let released = get_keys_released();

        for key in pressed {
            self.keys_held.insert(key);
        }

        for key in released {
            self.keys_held.remove(&key);
        }

        self.move_left =
            self.keys_held.contains(&KeyCode::A) || self.keys_held.contains(&KeyCode::Left);
        self.move_right =
            self.keys_held.contains(&KeyCode::D) || self.keys_held.contains(&KeyCode::Right);
        self.jump = self.keys_held.contains(&KeyCode::Space)
            || self.keys_held.contains(&KeyCode::W)
            || self.keys_held.contains(&KeyCode::Up);
        self.crouch = self.keys_held.contains(&KeyCode::LeftControl)
            || self.keys_held.contains(&KeyCode::S)
            || self.keys_held.contains(&KeyCode::Down);
        self.shoot = is_mouse_button_down(MouseButton::Left);

        let sensitivity = crate::cvar::get_cvar_float("sensitivity");
        let m_yaw = crate::cvar::get_cvar_float("m_yaw");
        let m_pitch = crate::cvar::get_cvar_float("m_pitch");

        let mouse_delta = if ignore_mouse_delta {
            vec2(0.0, 0.0)
        } else {
            mouse_delta_position()
        };

        let joystick_sensitivity = 0.01;
        self.aim_x += mouse_delta.x * joystick_sensitivity * sensitivity * m_yaw;
        self.aim_y += mouse_delta.y * joystick_sensitivity * sensitivity * m_pitch;

        let len = (self.aim_x * self.aim_x + self.aim_y * self.aim_y).sqrt();
        if len > 0.0 {
            self.aim_x /= len;
            self.aim_y /= len;
        }

        self.aim_angle = self.aim_y.atan2(self.aim_x);

        while self.aim_angle > std::f32::consts::PI {
            self.aim_angle -= 2.0 * std::f32::consts::PI;
        }
        while self.aim_angle < -std::f32::consts::PI {
            self.aim_angle += 2.0 * std::f32::consts::PI;
        }

        self.weapon_switch = None;
        if is_key_pressed(KeyCode::Key1) {
            self.weapon_switch = Some(0);
        } else if is_key_pressed(KeyCode::Key2) {
            self.weapon_switch = Some(1);
        } else if is_key_pressed(KeyCode::Key3) {
            self.weapon_switch = Some(2);
        } else if is_key_pressed(KeyCode::Key4) {
            self.weapon_switch = Some(3);
        } else if is_key_pressed(KeyCode::Key5) {
            self.weapon_switch = Some(4);
        } else if is_key_pressed(KeyCode::Key6) {
            self.weapon_switch = Some(5);
        } else if is_key_pressed(KeyCode::Key7) {
            self.weapon_switch = Some(6);
        } else if is_key_pressed(KeyCode::Key8) {
            self.weapon_switch = Some(7);
        } else if is_key_pressed(KeyCode::Key9) {
            self.weapon_switch = Some(8);
        }
    }
}

impl LocalMultiplayerInput {
    pub fn new() -> Self {
        Self {
            player1: PlayerInput::new(),
            player2: PlayerInput::new(),
        }
    }

    pub fn update(&mut self, ignore_mouse_delta: bool) {
        self.player1.move_left = is_key_down(KeyCode::A);
        self.player1.move_right = is_key_down(KeyCode::D);
        self.player1.jump = is_key_down(KeyCode::W);
        self.player1.crouch = is_key_down(KeyCode::S) || is_key_down(KeyCode::LeftShift);
        self.player1.shoot = is_key_down(KeyCode::Space);

        let old_flip = self.player1.flip_x;
        if self.player1.move_left && !self.player1.move_right {
            self.player1.flip_x = true;
        } else if self.player1.move_right && !self.player1.move_left {
            self.player1.flip_x = false;
        }

        if old_flip != self.player1.flip_x {
            self.player1.aim_angle = std::f32::consts::PI - self.player1.aim_angle;
        }

        self.player2.move_left = is_key_down(KeyCode::Left);
        self.player2.move_right = is_key_down(KeyCode::Right);
        self.player2.jump = is_mouse_button_down(MouseButton::Right);
        self.player2.crouch = is_key_down(KeyCode::Down);
        self.player2.shoot = is_mouse_button_down(MouseButton::Left);

        self.player1.weapon_switch = None;
        if is_key_pressed(KeyCode::Key1) {
            self.player1.weapon_switch = Some(0);
        } else if is_key_pressed(KeyCode::Key2) {
            self.player1.weapon_switch = Some(1);
        } else if is_key_pressed(KeyCode::Key3) {
            self.player1.weapon_switch = Some(2);
        } else if is_key_pressed(KeyCode::Key4) {
            self.player1.weapon_switch = Some(3);
        } else if is_key_pressed(KeyCode::Key5) {
            self.player1.weapon_switch = Some(4);
        } else if is_key_pressed(KeyCode::Key6) {
            self.player1.weapon_switch = Some(5);
        } else if is_key_pressed(KeyCode::Key7) {
            self.player1.weapon_switch = Some(6);
        } else if is_key_pressed(KeyCode::Key8) {
            self.player1.weapon_switch = Some(7);
        } else if is_key_pressed(KeyCode::Key9) {
            self.player1.weapon_switch = Some(8);
        } else if is_key_pressed(KeyCode::F) {
            self.player1.weapon_switch = Some(4);
        } else if is_key_pressed(KeyCode::R) {
            self.player1.weapon_switch = Some(6);
        } else if is_key_pressed(KeyCode::Q) {
            self.player1.weapon_switch = Some(7);
        } else if is_key_pressed(KeyCode::G) {
            self.player1.weapon_switch = Some(3);
        }

        self.player2.weapon_switch = None;
        let mouse_wheel = mouse_wheel().1;
        if mouse_wheel > 0.0 {
            self.player2.weapon_switch = Some(255);
        } else if mouse_wheel < 0.0 {
            self.player2.weapon_switch = Some(254);
        }

        let keyboard_aim_speed = 0.02;

        if is_key_down(KeyCode::I) {
            self.player1.aim_angle += keyboard_aim_speed;
        }
        if is_key_down(KeyCode::K) {
            self.player1.aim_angle -= keyboard_aim_speed;
        }

        while self.player1.aim_angle > std::f32::consts::PI {
            self.player1.aim_angle -= 2.0 * std::f32::consts::PI;
        }
        while self.player1.aim_angle < -std::f32::consts::PI {
            self.player1.aim_angle += 2.0 * std::f32::consts::PI;
        }

        self.player1.aim_x = self.player1.aim_angle.cos();
        self.player1.aim_y = self.player1.aim_angle.sin();

        let sensitivity = crate::cvar::get_cvar_float("sensitivity");
        let m_yaw = crate::cvar::get_cvar_float("m_yaw");
        let m_pitch = crate::cvar::get_cvar_float("m_pitch");

        let mouse_delta = if ignore_mouse_delta {
            vec2(0.0, 0.0)
        } else {
            mouse_delta_position()
        };

        let joystick_sensitivity = 0.01;
        self.player2.aim_x += mouse_delta.x * joystick_sensitivity * sensitivity * m_yaw;
        self.player2.aim_y += mouse_delta.y * joystick_sensitivity * sensitivity * m_pitch;

        let len = (self.player2.aim_x * self.player2.aim_x
            + self.player2.aim_y * self.player2.aim_y)
            .sqrt();
        if len > 0.0 {
            self.player2.aim_x /= len;
            self.player2.aim_y /= len;
        }

        self.player2.aim_angle = self.player2.aim_y.atan2(self.player2.aim_x);

        if self.player2.aim_x >= 0.0 {
            self.player2.flip_x = false;
        } else {
            self.player2.flip_x = true;
        }
    }
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            move_left: false,
            move_right: false,
            jump: false,
            crouch: false,
            shoot: false,
            aim_angle: 0.0,
            aim_x: 1.0,
            aim_y: 0.0,
            weapon_switch: None,
            flip_x: false,
        }
    }
}
