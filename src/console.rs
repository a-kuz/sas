use macroquad::prelude::*;
use crate::cvar;
use crate::game::tile_shader::{TileShaderRenderer, TileShader};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

const CON_TEXTSIZE: usize = 32768;
const NUM_CON_TIMES: usize = 4;
const COMMAND_HISTORY: usize = 32;
const HISTORY_FILE: &str = "sas_history.txt";

pub struct Console {
    initialized: bool,
    text: Vec<ConChar>,
    current: i32,
    x: i32,
    display: i32,
    linewidth: i32,
    totallines: i32,
    display_frac: f32,
    final_frac: f32,
    input_line: String,
    cursor_pos: usize,
    history: Vec<String>,
    history_line: i32,
    times: [f64; NUM_CON_TIMES],
    shader_renderer: Option<TileShaderRenderer>,
    pub bot_add_request: Option<String>,
    pub bot_remove_request: bool,
    pub bot_afk_request: bool,
    pub connect_request: Option<(String, String)>,
    pub disconnect_request: bool,
    pub net_stats_toggle: bool,
    pub net_graph_toggle: bool,
    pub net_showmiss_toggle: bool,
    pub server_bot_add_request: bool,
    pub is_connected_to_server: bool,
    pub end_match_request: bool,
}

#[derive(Clone, Copy)]
struct ConChar {
    ch: u8,
    color: u8,
}

impl Console {
    pub fn new() -> Self {
        Self {
            initialized: false,
            text: vec![ConChar { ch: b' ', color: 7 }; CON_TEXTSIZE],
            current: 0,
            x: 0,
            display: 0,
            linewidth: 78,
            totallines: CON_TEXTSIZE as i32 / 78,
            display_frac: 0.0,
            final_frac: 0.0,
            input_line: String::new(),
            cursor_pos: 0,
            history: Vec::new(),
            history_line: 0,
            times: [0.0; NUM_CON_TIMES],
            shader_renderer: None,
            bot_add_request: None,
            bot_remove_request: false,
            bot_afk_request: false,
            connect_request: None,
            disconnect_request: false,
            net_stats_toggle: false,
            net_graph_toggle: false,
            net_showmiss_toggle: false,
            server_bot_add_request: false,
            is_connected_to_server: false,
            end_match_request: false,
        }
    }

    pub fn init(&mut self) {
        self.check_resize();
        self.load_history();
        self.initialized = true;
    }
    
    fn get_history_path() -> PathBuf {
        PathBuf::from(HISTORY_FILE)
    }
    
    fn load_history(&mut self) {
        let path = Self::get_history_path();
        if let Ok(file) = fs::File::open(&path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if !line.is_empty() {
                    self.history.push(line);
                }
            }
            if self.history.len() > COMMAND_HISTORY {
                self.history = self.history.split_off(self.history.len() - COMMAND_HISTORY);
            }
            self.history_line = self.history.len() as i32;
        }
    }
    
    fn save_history(&self) {
        let path = Self::get_history_path();
        if let Ok(mut file) = fs::File::create(&path) {
            for cmd in &self.history {
                let _ = writeln!(file, "{}", cmd);
            }
        }
    }

    pub async fn load_texture(&mut self) {
        let mut renderer = TileShaderRenderer::new();
        
        renderer.load_texture("gfx/misc/console01.tga").await;
        renderer.load_texture("gfx/misc/console02.tga").await;
        
        use crate::game::tile_shader::{ShaderStage, BlendMode};
        
        let console_shader = TileShader {
            name: "console".to_string(),
            base_texture: String::new(),
            stages: vec![
                ShaderStage {
                    texture_path: "gfx/misc/console01.tga".to_string(),
                    blend_mode: BlendMode::None,
                    scroll_x: 0.02,
                    scroll_y: 0.0,
                    scale_x: 2.0,
                    scale_y: 1.0,
                    alpha: 1.0,
                    ..Default::default()
                },
                ShaderStage {
                    texture_path: "gfx/misc/console02.tga".to_string(),
                    blend_mode: BlendMode::Add,
                    scroll_x: 0.2,
                    scroll_y: 0.1,
                    scale_x: 2.0,
                    scale_y: 1.0,
                    alpha: 0.5,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        
        renderer.add_shader(console_shader);
        self.shader_renderer = Some(renderer);
    }

    fn check_resize(&mut self) {
        let width = screen_width() as i32;
        let char_width = 8;
        let new_linewidth = (width / char_width - 2).max(20);
        
        if new_linewidth == self.linewidth {
            return;
        }

        self.linewidth = new_linewidth;
        self.totallines = CON_TEXTSIZE as i32 / self.linewidth;
    }

    pub fn toggle(&mut self) {
        if self.final_frac == 0.0 {
            self.final_frac = 0.5;
        } else {
            self.final_frac = 0.0;
        }
    }

    pub fn is_open(&self) -> bool {
        self.display_frac > 0.01
    }

    pub fn print(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '\n' => self.linefeed(),
                '\r' => self.x = 0,
                _ => {
                    if self.x >= self.linewidth {
                        self.linefeed();
                    }
                    let y = (self.current % self.totallines) as usize;
                    let idx = y * self.linewidth as usize + self.x as usize;
                    if idx < self.text.len() {
                        self.text[idx] = ConChar {
                            ch: ch as u8,
                            color: 7,
                        };
                    }
                    self.x += 1;
                }
            }
        }

        if self.current >= 0 {
            self.times[self.current as usize % NUM_CON_TIMES] = get_time();
        }
    }

    fn linefeed(&mut self) {
        self.x = 0;
        if self.display == self.current {
            self.display += 1;
        }
        self.current += 1;
        
        let y = (self.current % self.totallines) as usize;
        for i in 0..self.linewidth as usize {
            let idx = y * self.linewidth as usize + i;
            if idx < self.text.len() {
                self.text[idx] = ConChar { ch: b' ', color: 7 };
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.display < self.current {
            self.display += 10;
            if self.display > self.current {
                self.display = self.current;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        self.display -= 10;
        if self.display < 0 {
            self.display = 0;
        }
    }

    pub fn handle_character(&mut self, ch: char) {
        if ch.is_ascii() && !ch.is_control() {
            self.input_line.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
        }
    }
    
    pub fn copy_to_clipboard(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(&self.input_line);
            }
        }
    }
    
    pub fn paste_from_clipboard(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    for ch in text.chars() {
                        if ch.is_ascii() && !ch.is_control() {
                            self.input_line.insert(self.cursor_pos, ch);
                            self.cursor_pos += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _character: Option<char>) {
        match key {
            KeyCode::Enter | KeyCode::KpEnter => {
                if !self.input_line.is_empty() {
                    self.execute_command();
                }
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.input_line.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input_line.len() {
                    self.input_line.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input_line.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.input_line.len();
            }
            KeyCode::Up => {
                if !self.history.is_empty() && self.history_line > 0 {
                    self.history_line -= 1;
                    self.input_line = self.history[self.history_line as usize].clone();
                    self.cursor_pos = self.input_line.len();
                }
            }
            KeyCode::Down => {
                if !self.history.is_empty() && self.history_line < self.history.len() as i32 - 1 {
                    self.history_line += 1;
                    self.input_line = self.history[self.history_line as usize].clone();
                    self.cursor_pos = self.input_line.len();
                } else if self.history_line == self.history.len() as i32 - 1 {
                    self.history_line = self.history.len() as i32;
                    self.input_line.clear();
                    self.cursor_pos = 0;
                }
            }
            KeyCode::Tab => {
                self.autocomplete();
            }
            _ => {}
        }
    }

    fn autocomplete(&mut self) {
        let input = self.input_line.trim_start();
        
        let commands = vec![
            "clear", "cls", "cvarlist", "toggle", "echo", "reset", 
            "writeconfig", "bot_add", "addbot", "bot_remove", "removebot",
            "bot_afk", "help", "set", "endmatch"
        ];
        
        let mut all_matches = Vec::new();
        
        for cmd in &commands {
            if cmd.starts_with(input) {
                all_matches.push(cmd.to_string());
            }
        }
        
        let cvar_matches = cvar::find_cvar_matches(input);
        all_matches.extend(cvar_matches.clone());
        
        if all_matches.is_empty() {
            return;
        }
        
        if all_matches.len() == 1 {
            self.input_line = all_matches[0].clone() + " ";
            self.cursor_pos = self.input_line.len();
        } else {
            let common_prefix = find_common_prefix(&all_matches);
            if common_prefix.len() > input.len() {
                self.input_line = common_prefix;
                self.cursor_pos = self.input_line.len();
            }
            
            self.print(&format!("]{}  \n", self.input_line));
            for m in &all_matches {
                if let Some(cvar) = cvar::get_cvar(m) {
                    self.print(&format!("  {} = \"{}\"\n", m, cvar.value));
                } else {
                    self.print(&format!("  {}\n", m));
                }
            }
        }
    }

    fn execute_command(&mut self) {
        self.print(&format!("]{}  \n", self.input_line));
        
        self.history.push(self.input_line.clone());
        if self.history.len() > COMMAND_HISTORY {
            self.history.remove(0);
        }
        self.history_line = self.history.len() as i32;
        self.save_history();

        let parts: Vec<&str> = self.input_line.split_whitespace().collect();
        if parts.is_empty() {
            self.input_line.clear();
            self.cursor_pos = 0;
            return;
        }

        let cmd = parts[0];
        
        if cmd == "clear" || cmd == "cls" {
            self.clear();
        } else if cmd == "cvarlist" {
            let cvars = cvar::find_cvar_matches("");
            self.print(&format!("{} cvars:\n", cvars.len()));
            for cvar_name in cvars {
                if let Some(cv) = cvar::get_cvar(&cvar_name) {
                    self.print(&format!("  {} = \"{}\"\n", cvar_name, cv.value));
                }
            }
        } else if cmd == "toggle" && parts.len() >= 2 {
            let var_name = parts[1];
            if let Some(cv) = cvar::get_cvar(var_name) {
                let new_val = if cv.get_integer() == 0 { "1" } else { "0" };
                cvar::set_cvar(var_name, new_val);
                self.print(&format!("{} toggled to {}\n", var_name, new_val));
            } else {
                self.print(&format!("Unknown cvar: {}\n", var_name));
            }
        } else if cmd == "echo" {
            let text = parts[1..].join(" ");
            self.print(&format!("{}\n", text));
        } else if cmd == "reset" && parts.len() >= 2 {
            let var_name = parts[1];
            if let Some(cv) = cvar::get_cvar(var_name) {
                cvar::set_cvar(var_name, &cv.default_value);
                self.print(&format!("{} reset to default: {}\n", var_name, cv.default_value));
            } else {
                self.print(&format!("Unknown cvar: {}\n", var_name));
            }
        } else if cmd == "writeconfig" {
            cvar::save_config();
            self.print("Config saved to sas_config.cfg\n");
        } else if cmd == "bot_add" || cmd == "addbot" {
            if self.is_connected_to_server {
                self.server_bot_add_request = true;
                self.print("Requesting bot from server...\n");
            } else {
                let model = if parts.len() >= 2 { 
                    parts[1].to_string()
                } else { 
                    "sarge".to_string()
                };
                self.bot_add_request = Some(model.clone());
                self.print(&format!("Adding bot: {}\n", model));
            }
        } else if cmd == "bot_remove" || cmd == "removebot" {
            self.bot_remove_request = true;
            self.print("Removing bot\n");
        } else if cmd == "bot_afk" {
            self.bot_afk_request = true;
            self.print("Toggling bot AFK mode\n");
        } else if cmd == "connect" {
            if parts.len() >= 2 {
                let server = parts[1].to_string();
                let player_name = if parts.len() >= 3 {
                    parts[2].to_string()
                } else {
                    "Player".to_string()
                };
                self.connect_request = Some((server.clone(), player_name.clone()));
                self.print(&format!("Connecting to {} as {}...\n", server, player_name));
            } else {
                self.print("Usage: connect <server:port> [playername]\n");
            }
        } else if cmd == "disconnect" {
            self.disconnect_request = true;
            self.print("Disconnecting from server...\n");
        } else if cmd == "net_stats" {
            self.net_stats_toggle = true;
            self.print("Network stats toggled\n");
        } else if cmd == "net_graph" {
            self.net_graph_toggle = true;
            self.print("Network graph toggled\n");
        } else if cmd == "net_showmiss" || cmd == "cl_showmiss" {
            self.net_showmiss_toggle = true;
            self.print("Prediction errors visualization toggled\n");
        } else if cmd == "endmatch" {
            self.end_match_request = true;
            self.print("Ending match...\n");
        } else if cmd == "help" || cmd == "?" {
            self.print("Console commands:\n");
            self.print("  clear/cls - Clear console\n");
            self.print("  cvarlist - List all cvars\n");
            self.print("  toggle <cvar> - Toggle cvar 0/1\n");
            self.print("  echo <text> - Print text\n");
            self.print("  reset <cvar> - Reset to default\n");
            self.print("  writeconfig - Save config\n");
            self.print("  bot_add [model] - Add bot\n");
            self.print("  bot_remove - Remove bot\n");
            self.print("  bot_afk - Toggle bot AFK mode\n");
            self.print("  connect <server:port> [name] - Connect to server\n");
            self.print("  disconnect - Disconnect from server\n");
            self.print("  endmatch - End current match\n");
            self.print("  set <cvar> <value> - Set cvar\n");
            self.print("  <cvar> - Show cvar value\n");
            self.print("  <cvar> <value> - Set cvar value\n");
        } else if cmd == "set" && parts.len() >= 3 {
            let var_name = parts[1];
            let value = parts[2..].join(" ");
            cvar::set_cvar(var_name, &value);
            self.print(&format!("{} set to {}\n", var_name, value));
        } else if parts.len() == 1 {
            if let Some(cvar) = cvar::get_cvar(cmd) {
                self.print(&format!("\"{}\" is \"{}\" default: \"{}\"\n", 
                    cvar.name, cvar.value, cvar.default_value));
            } else {
                self.print(&format!("Unknown command: {}\n", cmd));
            }
        } else if parts.len() == 2 {
            if let Some(_) = cvar::get_cvar(cmd) {
                cvar::set_cvar(cmd, parts[1]);
                self.print(&format!("{} set to {}\n", cmd, parts[1]));
            } else {
                self.print(&format!("Unknown command: {}\n", cmd));
            }
        } else {
            self.print(&format!("Unknown command: {}\n", cmd));
        }

        self.input_line.clear();
        self.cursor_pos = 0;
    }

    pub fn clear(&mut self) {
        for i in 0..self.text.len() {
            self.text[i] = ConChar { ch: b' ', color: 7 };
        }
        self.current = 0;
        self.x = 0;
        self.display = 0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.display_frac != self.final_frac {
            let speed = 3.0;
            if self.display_frac < self.final_frac {
                self.display_frac += speed * dt;
                if self.display_frac > self.final_frac {
                    self.display_frac = self.final_frac;
                }
            } else {
                self.display_frac -= speed * dt;
                if self.display_frac < self.final_frac {
                    self.display_frac = self.final_frac;
                }
            }
        }
    }

    pub fn draw(&self) {
        if self.display_frac <= 0.0 {
            return;
        }

        let screen_w = screen_width();
        let screen_h = screen_height();
        let y = (self.display_frac * screen_h).round();
        
        if y < 1.0 {
            return;
        }

        if let Some(ref renderer) = self.shader_renderer {
            let time = get_time() as f32;
            renderer.render_tile_with_shader("console", 0.0, 0.0, screen_w, y, 0.0, 0.0, time);
        } else {
            draw_rectangle(0.0, 0.0, screen_w, y, Color::from_rgba(0, 0, 0, 200));
        }

        draw_rectangle(0.0, y, screen_w, 2.0, Color::from_rgba(255, 0, 0, 255));

        let char_height = 12.0;
        let char_width = 8.0;
        let rows = ((y - char_height * 2.5) / char_height) as i32;
        
        let mut text_y = y - char_height * 2.5;
        let mut row = self.display;
        
        if self.x == 0 {
            row -= 1;
        }

        for _ in 0..rows {
            if row < 0 {
                break;
            }
            if self.current - row >= self.totallines {
                row -= 1;
                text_y -= char_height;
                continue;
            }

            let text_row = (row % self.totallines) as usize;
            let start_idx = text_row * self.linewidth as usize;
            
            for x in 0..self.linewidth {
                let idx = start_idx + x as usize;
                if idx >= self.text.len() {
                    break;
                }
                
                let con_char = self.text[idx];
                if con_char.ch != b' ' {
                    crate::render::draw_q3_small_char(
                        (x + 1) as f32 * char_width,
                        text_y,
                        char_height,
                        con_char.ch,
                        Color::from_rgba(255, 255, 255, 255),
                    );
                }
            }

            row -= 1;
            text_y -= char_height;
        }

        let input_y = y - char_height * 1.5;
        crate::render::draw_q3_small_char(char_width, input_y, char_height, b']', WHITE);
        
        for (i, ch) in self.input_line.chars().enumerate() {
            crate::render::draw_q3_small_char(
                (i as f32 + 2.0) * char_width,
                input_y,
                char_height,
                ch as u8,
                WHITE,
            );
        }
        
        let cursor_time = (get_time() * 2.0) as i32;
        if cursor_time % 2 == 0 {
            crate::render::draw_q3_small_char(
                (self.cursor_pos as f32 + 2.0) * char_width,
                input_y,
                char_height,
                b'_',
                Color::from_rgba(255, 255, 0, 255),
            );
        }
    }
}

fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    
    let first = &strings[0];
    let mut prefix = String::new();
    
    for (i, ch) in first.chars().enumerate() {
        if strings.iter().all(|s| {
            s.chars().nth(i).map(|c| c.to_ascii_lowercase()) == Some(ch.to_ascii_lowercase())
        }) {
            prefix.push(ch);
        } else {
            break;
        }
    }
    
    prefix
}

