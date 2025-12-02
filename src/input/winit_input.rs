use winit::event::{ElementState, WindowEvent};
pub use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};
use std::collections::HashSet;
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    Space, Enter, Escape, Tab, Backspace, Delete,
    Left, Right, Up, Down,
    LeftShift, RightShift, LeftControl, RightControl,
    LeftAlt, RightAlt,
    GraveAccent, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Home, End, PageUp, PageDown,
    Kp0, Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7, Kp8, Kp9, KpEnter,
    Minus, Equal, LeftBracket, RightBracket, Backslash, Semicolon, Apostrophe, Comma, Period, Slash,
}

pub fn key_to_keycode(key_event: &winit::event::KeyEvent) -> Option<KeyCode> {
        match &key_event.logical_key {
            Key::Named(named) => match named {
                NamedKey::Space => Some(KeyCode::Space),
                NamedKey::Enter => Some(KeyCode::Enter),
                NamedKey::Escape => Some(KeyCode::Escape),
                NamedKey::Tab => Some(KeyCode::Tab),
                NamedKey::Backspace => Some(KeyCode::Backspace),
                NamedKey::Delete => Some(KeyCode::Delete),
                NamedKey::ArrowLeft => Some(KeyCode::Left),
                NamedKey::ArrowRight => Some(KeyCode::Right),
                NamedKey::ArrowUp => Some(KeyCode::Up),
                NamedKey::ArrowDown => Some(KeyCode::Down),
                NamedKey::Shift => {
                    match &key_event.physical_key {
                        winit::keyboard::PhysicalKey::Code(code) => {
                            match code {
                                winit::keyboard::KeyCode::ShiftLeft => Some(KeyCode::LeftShift),
                                winit::keyboard::KeyCode::ShiftRight => Some(KeyCode::RightShift),
                                _ => Some(KeyCode::LeftShift),
                            }
                        }
                        _ => Some(KeyCode::LeftShift),
                    }
                }
                NamedKey::Control => {
                    match &key_event.physical_key {
                        winit::keyboard::PhysicalKey::Code(code) => {
                            match code {
                                winit::keyboard::KeyCode::ControlLeft => Some(KeyCode::LeftControl),
                                winit::keyboard::KeyCode::ControlRight => Some(KeyCode::RightControl),
                                _ => Some(KeyCode::LeftControl),
                            }
                        }
                        _ => Some(KeyCode::LeftControl),
                    }
                }
                NamedKey::Alt => {
                    match &key_event.physical_key {
                        winit::keyboard::PhysicalKey::Code(code) => {
                            match code {
                                winit::keyboard::KeyCode::AltLeft => Some(KeyCode::LeftAlt),
                                winit::keyboard::KeyCode::AltRight => Some(KeyCode::RightAlt),
                                _ => Some(KeyCode::LeftAlt),
                            }
                        }
                        _ => Some(KeyCode::LeftAlt),
                    }
                }
                NamedKey::Home => Some(KeyCode::Home),
                NamedKey::End => Some(KeyCode::End),
                NamedKey::PageUp => Some(KeyCode::PageUp),
                NamedKey::PageDown => Some(KeyCode::PageDown),
                NamedKey::F1 => Some(KeyCode::F1),
                NamedKey::F2 => Some(KeyCode::F2),
                NamedKey::F3 => Some(KeyCode::F3),
                NamedKey::F4 => Some(KeyCode::F4),
                NamedKey::F5 => Some(KeyCode::F5),
                NamedKey::F6 => Some(KeyCode::F6),
                NamedKey::F7 => Some(KeyCode::F7),
                NamedKey::F8 => Some(KeyCode::F8),
                NamedKey::F9 => Some(KeyCode::F9),
                NamedKey::F10 => Some(KeyCode::F10),
                NamedKey::F11 => Some(KeyCode::F11),
                NamedKey::F12 => Some(KeyCode::F12),
                _ => None,
            },
            Key::Character(ch) => {
                match ch.as_str() {
                    "a" | "A" => Some(KeyCode::A),
                    "b" | "B" => Some(KeyCode::B),
                    "c" | "C" => Some(KeyCode::C),
                    "d" | "D" => Some(KeyCode::D),
                    "e" | "E" => Some(KeyCode::E),
                    "f" | "F" => Some(KeyCode::F),
                    "g" | "G" => Some(KeyCode::G),
                    "h" | "H" => Some(KeyCode::H),
                    "i" | "I" => Some(KeyCode::I),
                    "j" | "J" => Some(KeyCode::J),
                    "k" | "K" => Some(KeyCode::K),
                    "l" | "L" => Some(KeyCode::L),
                    "m" | "M" => Some(KeyCode::M),
                    "n" | "N" => Some(KeyCode::N),
                    "o" | "O" => Some(KeyCode::O),
                    "p" | "P" => Some(KeyCode::P),
                    "q" | "Q" => Some(KeyCode::Q),
                    "r" | "R" => Some(KeyCode::R),
                    "s" | "S" => Some(KeyCode::S),
                    "t" | "T" => Some(KeyCode::T),
                    "u" | "U" => Some(KeyCode::U),
                    "v" | "V" => Some(KeyCode::V),
                    "w" | "W" => Some(KeyCode::W),
                    "x" | "X" => Some(KeyCode::X),
                    "y" | "Y" => Some(KeyCode::Y),
                    "z" | "Z" => Some(KeyCode::Z),
                    "0" => Some(KeyCode::Key0),
                    "1" => Some(KeyCode::Key1),
                    "2" => Some(KeyCode::Key2),
                    "3" => Some(KeyCode::Key3),
                    "4" => Some(KeyCode::Key4),
                    "5" => Some(KeyCode::Key5),
                    "6" => Some(KeyCode::Key6),
                    "7" => Some(KeyCode::Key7),
                    "8" => Some(KeyCode::Key8),
                    "9" => Some(KeyCode::Key9),
                    "`" | "~" => Some(KeyCode::GraveAccent),
                    "-" => Some(KeyCode::Minus),
                    "=" => Some(KeyCode::Equal),
                    "[" => Some(KeyCode::LeftBracket),
                    "]" => Some(KeyCode::RightBracket),
                    "\\" => Some(KeyCode::Backslash),
                    ";" => Some(KeyCode::Semicolon),
                    "'" => Some(KeyCode::Apostrophe),
                    "," => Some(KeyCode::Comma),
                    "." => Some(KeyCode::Period),
                    "/" => Some(KeyCode::Slash),
                    _ => None,
                }
            }
            Key::Unidentified(_) => None,
            Key::Dead(_) => None,
        }
}

pub struct WinitInputState {
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_released: HashSet<KeyCode>,
    pub keys_held: HashSet<KeyCode>,
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    pub mouse_buttons_released: HashSet<MouseButton>,
    pub mouse_buttons_held: HashSet<MouseButton>,
    pub mouse_delta: Vec2,
    pub mouse_wheel: Vec2,
    pub mouse_position: Vec2,
    pub window_size: (u32, u32),
}

impl WinitInputState {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            keys_held: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            mouse_buttons_held: HashSet::new(),
            mouse_delta: Vec2::ZERO,
            mouse_wheel: Vec2::ZERO,
            mouse_position: Vec2::ZERO,
            window_size: (1920, 1080),
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(keycode) = key_to_keycode(event) {
                    match event.state {
                        ElementState::Pressed => {
                            if !self.keys_held.contains(&keycode) {
                                self.keys_pressed.insert(keycode);
                                self.keys_held.insert(keycode);
                            }
                        }
                        ElementState::Released => {
                            self.keys_released.insert(keycode);
                            self.keys_held.remove(&keycode);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        if !self.mouse_buttons_held.contains(button) {
                            self.mouse_buttons_pressed.insert(*button);
                            self.mouse_buttons_held.insert(*button);
                        }
                    }
                    ElementState::Released => {
                        self.mouse_buttons_released.insert(*button);
                        self.mouse_buttons_held.remove(button);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let old_pos = self.mouse_position;
                self.mouse_position = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_delta = self.mouse_position - old_pos;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        self.mouse_wheel = Vec2::new(*x, *y);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.mouse_wheel = Vec2::new(pos.x as f32, pos.y as f32);
                    }
                }
            }
            WindowEvent::Resized(size) => {
                self.window_size = (size.width, size.height);
            }
            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel = Vec2::ZERO;
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn get_keys_pressed(&self) -> Vec<KeyCode> {
        self.keys_pressed.iter().copied().collect()
    }

    pub fn get_keys_released(&self) -> Vec<KeyCode> {
        self.keys_released.iter().copied().collect()
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_held.contains(&button)
    }

    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }

    pub fn mouse_delta_position(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn mouse_wheel(&self) -> (f32, f32) {
        (self.mouse_wheel.x, self.mouse_wheel.y)
    }

    pub fn screen_width(&self) -> f32 {
        self.window_size.0 as f32
    }

    pub fn screen_height(&self) -> f32 {
        self.window_size.1 as f32
    }
}

