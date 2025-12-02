use crate::console::Console;
use crate::cvar;
use crate::game_loop::GameLoop;
use crate::input::winit_input::{WinitInputState, KeyCode};
use crate::menu::MenuState;
use crate::render;
use crate::time;
use pollster;
use winit::window::Window;

pub struct App {
    pub menu_state: Option<MenuState>,
    pub game_loop: Option<GameLoop>,
    pub console: Console,
    was_console_open: bool,
    ignore_mouse_delta_for_one_frame: bool,
    last_time: f64,
}

impl App {
    pub fn new() -> Self {
        cvar::init_default_cvars();
        time::init();
        Self {
            menu_state: Some(MenuState::new()),
            game_loop: None,
            console: Console::new(),
            was_console_open: false,
            ignore_mouse_delta_for_one_frame: false,
            last_time: time::get_time(),
        }
    }

    pub async fn initialize(&mut self) {
        self.console.init();
        self.console.print("SAS III Console initialized\n");

        if let Some(menu_state) = &mut self.menu_state {
            menu_state.init().await;
        }
    }

    pub fn update(&mut self, input_state: &mut WinitInputState, window: &Window) -> bool {
        let current_time = time::get_time();
        let dt = (current_time - self.last_time) as f32;
        self.last_time = current_time;
        let dt = dt.min(0.05);

        let mut should_toggle_console =
            input_state.is_key_pressed(KeyCode::GraveAccent) || input_state.is_key_pressed(KeyCode::F12);

        if !should_toggle_console {
            for key in input_state.get_keys_pressed() {
                let ch = match key {
                    KeyCode::GraveAccent => Some('`'),
                    _ => None,
                };
                if let Some(ch) = ch {
                    if ch == '`' || ch == '~' || ch == 'ё' || ch == 'Ё' {
                        should_toggle_console = true;
                    }
                }
            }
        }

        if should_toggle_console {
            self.console.toggle();
            if self.console.is_open() {
                #[cfg(not(target_os = "macos"))]
                {
                    window.set_cursor_grab(winit::window::CursorGrabMode::None).ok();
                    window.set_cursor_visible(true);
                }
            } else if self.game_loop.is_some() {
                #[cfg(target_os = "macos")]
                {
                    let size = window.inner_size();
                    crate::input::center_mouse_cursor(size.width as f64, size.height as f64);
                }
                input_state.mouse_delta = glam::Vec2::ZERO;
                let show_cursor = cvar::get_cvar_bool("m_show_cursor");
                window.set_cursor_visible(show_cursor);
                #[cfg(not(target_os = "macos"))]
                {
                    let grab_mouse = cvar::get_cvar_bool("m_grab");
                        window.set_cursor_grab(if grab_mouse { winit::window::CursorGrabMode::Locked } else { winit::window::CursorGrabMode::None }).ok();
                }
                self.ignore_mouse_delta_for_one_frame = true;
            }
        }

        if input_state.is_key_pressed(KeyCode::F5) {
            crate::cvar::save_config();
            self.console.print("Config saved to sas_config.cfg\n");
        }

        if self.console.is_open() {
            if input_state.is_key_pressed(KeyCode::Enter) || input_state.is_key_pressed(KeyCode::KpEnter) {
                self.console.handle_key(KeyCode::Enter, None);
            }
            if input_state.is_key_pressed(KeyCode::Backspace) {
                self.console.handle_key(KeyCode::Backspace, None);
            }
            if input_state.is_key_pressed(KeyCode::Delete) {
                self.console.handle_key(KeyCode::Delete, None);
            }
            if input_state.is_key_pressed(KeyCode::Left) {
                self.console.handle_key(KeyCode::Left, None);
            }
            if input_state.is_key_pressed(KeyCode::Right) {
                self.console.handle_key(KeyCode::Right, None);
            }
            if input_state.is_key_pressed(KeyCode::Up) {
                self.console.handle_key(KeyCode::Up, None);
            }
            if input_state.is_key_pressed(KeyCode::Down) {
                self.console.handle_key(KeyCode::Down, None);
            }
            if input_state.is_key_pressed(KeyCode::Home) {
                self.console.handle_key(KeyCode::Home, None);
            }
            if input_state.is_key_pressed(KeyCode::End) {
                self.console.handle_key(KeyCode::End, None);
            }
            if input_state.is_key_pressed(KeyCode::Tab) {
                self.console.handle_key(KeyCode::Tab, None);
            }
            if input_state.is_key_pressed(KeyCode::PageUp) {
                self.console.scroll_up();
            }
            if input_state.is_key_pressed(KeyCode::PageDown) {
                self.console.scroll_down();
            }

            let (_mx, my) = input_state.mouse_wheel();
            if my > 0.0 {
                self.console.scroll_up();
            } else if my < 0.0 {
                self.console.scroll_down();
            }

            let shift = input_state.is_key_down(KeyCode::LeftShift) || input_state.is_key_down(KeyCode::RightShift);
            let ctrl = input_state.is_key_down(KeyCode::LeftControl) || input_state.is_key_down(KeyCode::RightControl);

            if ctrl && input_state.is_key_pressed(KeyCode::C) {
                self.console.copy_to_clipboard();
            }
            if ctrl && input_state.is_key_pressed(KeyCode::V) {
                self.console.paste_from_clipboard();
            }

            if !should_toggle_console {
                let chars = get_chars_from_keys(input_state, shift, ctrl);
                for ch in chars {
                    self.console.handle_character(ch);
                }
            }
        }

        if let Some(menu_state) = &mut self.menu_state {
            window.set_cursor_visible(true);
            menu_state.update(dt);

            if let Some(game_state) = menu_state.handle_input(input_state) {
                let selected_model_idx = menu_state.get_selected_model_index();
                let available_models = menu_state.available_models.clone();

                let mut game_loop = pollster::block_on(GameLoop::new(game_state, selected_model_idx, available_models));
                pollster::block_on(game_loop.initialize_game());

                self.game_loop = Some(game_loop);
                self.menu_state = None;

                let show_cursor = cvar::get_cvar_bool("m_show_cursor");
                window.set_cursor_visible(show_cursor);
                let grab_mouse = cvar::get_cvar_bool("m_grab");
                if grab_mouse {
                    window.set_cursor_grab(winit::window::CursorGrabMode::Locked).ok();
                }

                #[cfg(target_os = "macos")]
                {
                    let size = window.inner_size();
                    crate::input::center_mouse_cursor(size.width as f64, size.height as f64);
                }

                input_state.mouse_delta = glam::Vec2::ZERO;
                self.ignore_mouse_delta_for_one_frame = true;
                self.was_console_open = false;
            }
        } else if let Some(game_loop) = &mut self.game_loop {
            if !self.console.is_open() {
                if self.was_console_open {
                    let show_cursor = cvar::get_cvar_bool("m_show_cursor");
                    window.set_cursor_visible(show_cursor);
                    #[cfg(not(target_os = "macos"))]
                    {
                        let grab_mouse = cvar::get_cvar_bool("m_grab");
                        window.set_cursor_grab(if grab_mouse { winit::window::CursorGrabMode::Locked } else { winit::window::CursorGrabMode::None }).ok();
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let size = window.inner_size();
                        crate::input::center_mouse_cursor(size.width as f64, size.height as f64);
                    }
                    input_state.mouse_delta = glam::Vec2::ZERO;
                    self.ignore_mouse_delta_for_one_frame = true;
                    self.was_console_open = false;
                }
            } else {
                self.was_console_open = true;
            }

                let ignore_delta = self.ignore_mouse_delta_for_one_frame;
                if !pollster::block_on(game_loop.run(&mut self.console, ignore_delta, input_state)) {
                println!("[Main] Returning to menu...");
                #[cfg(not(target_os = "macos"))]
                window.set_cursor_grab(false).ok();
                window.set_cursor_visible(true);
                self.menu_state = Some(MenuState::new());
                if let Some(menu_state) = &mut self.menu_state {
                    pollster::block_on(menu_state.init());
                }
                self.game_loop = None;
            }
            self.ignore_mouse_delta_for_one_frame = false;
        }

        self.console.update(dt);
        // self.console.draw(); // Removed macroquad draw call. Console is rendered in main_wgpu_full.rs or similar

        input_state.end_frame();
        true
    }
}

fn get_chars_from_keys(input_state: &WinitInputState, shift: bool, ctrl: bool) -> Vec<char> {
    let mut chars = Vec::new();
    for key in input_state.get_keys_pressed() {
        if ctrl {
            continue;
        }

        let ch = match key {
            KeyCode::Space => Some(' '),
            KeyCode::A => Some(if shift { 'A' } else { 'a' }),
            KeyCode::B => Some(if shift { 'B' } else { 'b' }),
            KeyCode::C => Some(if shift { 'C' } else { 'c' }),
            KeyCode::D => Some(if shift { 'D' } else { 'd' }),
            KeyCode::E => Some(if shift { 'E' } else { 'e' }),
            KeyCode::F => Some(if shift { 'F' } else { 'f' }),
            KeyCode::G => Some(if shift { 'G' } else { 'g' }),
            KeyCode::H => Some(if shift { 'H' } else { 'h' }),
            KeyCode::I => Some(if shift { 'I' } else { 'i' }),
            KeyCode::J => Some(if shift { 'J' } else { 'j' }),
            KeyCode::K => Some(if shift { 'K' } else { 'k' }),
            KeyCode::L => Some(if shift { 'L' } else { 'l' }),
            KeyCode::M => Some(if shift { 'M' } else { 'm' }),
            KeyCode::N => Some(if shift { 'N' } else { 'n' }),
            KeyCode::O => Some(if shift { 'O' } else { 'o' }),
            KeyCode::P => Some(if shift { 'P' } else { 'p' }),
            KeyCode::Q => Some(if shift { 'Q' } else { 'q' }),
            KeyCode::R => Some(if shift { 'R' } else { 'r' }),
            KeyCode::S => Some(if shift { 'S' } else { 's' }),
            KeyCode::T => Some(if shift { 'T' } else { 't' }),
            KeyCode::U => Some(if shift { 'U' } else { 'u' }),
            KeyCode::V => Some(if shift { 'V' } else { 'v' }),
            KeyCode::W => Some(if shift { 'W' } else { 'w' }),
            KeyCode::X => Some(if shift { 'X' } else { 'x' }),
            KeyCode::Y => Some(if shift { 'Y' } else { 'y' }),
            KeyCode::Z => Some(if shift { 'Z' } else { 'z' }),
            KeyCode::Key0 | KeyCode::Kp0 => Some(if shift { ')' } else { '0' }),
            KeyCode::Key1 | KeyCode::Kp1 => Some(if shift { '!' } else { '1' }),
            KeyCode::Key2 | KeyCode::Kp2 => Some(if shift { '@' } else { '2' }),
            KeyCode::Key3 | KeyCode::Kp3 => Some(if shift { '#' } else { '3' }),
            KeyCode::Key4 | KeyCode::Kp4 => Some(if shift { '$' } else { '4' }),
            KeyCode::Key5 | KeyCode::Kp5 => Some(if shift { '%' } else { '5' }),
            KeyCode::Key6 | KeyCode::Kp6 => Some(if shift { '^' } else { '6' }),
            KeyCode::Key7 | KeyCode::Kp7 => Some(if shift { '&' } else { '7' }),
            KeyCode::Key8 | KeyCode::Kp8 => Some(if shift { '*' } else { '8' }),
            KeyCode::Key9 | KeyCode::Kp9 => Some(if shift { '(' } else { '9' }),
            KeyCode::Period => Some(if shift { '>' } else { '.' }),
            KeyCode::Minus => Some(if shift { '_' } else { '-' }),
            KeyCode::Slash => Some(if shift { '?' } else { '/' }),
            KeyCode::Backslash => Some(if shift { '|' } else { '\\' }),
            KeyCode::Semicolon => Some(if shift { ':' } else { ';' }),
            KeyCode::Apostrophe => Some(if shift { '"' } else { '\'' }),
            KeyCode::Comma => Some(if shift { '<' } else { ',' }),
            KeyCode::Equal => Some(if shift { '+' } else { '=' }),
            KeyCode::LeftBracket => Some(if shift { '{' } else { '[' }),
            KeyCode::RightBracket => Some(if shift { '}' } else { ']' }),
            KeyCode::GraveAccent => Some(if shift { '~' } else { '`' }),
            _ => None,
        };
        if let Some(ch) = ch {
            chars.push(ch);
        }
    }
    chars
}
