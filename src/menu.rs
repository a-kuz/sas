use crate::game::GameState;
use crate::render;
use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

pub struct MenuState {
    pub current_menu: String,
    pub main_menu_selected: usize,
    pub map_menu_selected: usize,
    pub model_menu_selected: usize,
    pub available_maps: Vec<String>,
    pub available_models: Vec<String>,
    pub menu_move_sound: Option<Sound>,
    pub menu_select_sound: Option<Sound>,
    pub time: f32,
    pub logo_texture: Option<Texture2D>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            current_menu: "main".to_string(),
            main_menu_selected: 0,
            map_menu_selected: 0,
            model_menu_selected: 0,
            available_maps: Vec::new(),
            available_models: Vec::new(),
            menu_move_sound: None,
            menu_select_sound: None,
            time: 0.0,
            logo_texture: None,
        }
    }

    pub async fn init(&mut self) {
        self.available_maps = list_available_maps();
        self.available_models = list_available_models();

        let env_model = std::env::var("SAS_PLAYER_MODEL").unwrap_or_else(|_| "sarge".to_string());
        self.model_menu_selected = self
            .available_models
            .iter()
            .position(|m| m == &env_model)
            .unwrap_or(0);

        self.menu_move_sound = macroquad::audio::load_sound("q3-resources/sound/misc/menu1.wav")
            .await
            .ok();
        self.menu_select_sound = macroquad::audio::load_sound("q3-resources/sound/misc/menu2.wav")
            .await
            .ok();

        // Load logo
        self.logo_texture = load_texture("assets/logo-alfa.png").await.ok();
    }

    pub async fn handle_input(&mut self) -> Option<GameState> {
        let main_menu_items = ["DEATHMATCH", "HOTSEAT", "QUIT"];

        match self.current_menu.as_str() {
            "main" => {
                if is_key_pressed(KeyCode::Down) {
                    self.main_menu_selected = (self.main_menu_selected + 1) % main_menu_items.len();
                    self.play_move_sound();
                }
                if is_key_pressed(KeyCode::Up) {
                    self.main_menu_selected = if self.main_menu_selected == 0 {
                        main_menu_items.len() - 1
                    } else {
                        self.main_menu_selected - 1
                    };
                    self.play_move_sound();
                }

                if let Some(clicked_idx) = self.check_menu_click(main_menu_items.len()) {
                    self.main_menu_selected = clicked_idx;
                    self.play_select_sound();
                    match self.main_menu_selected {
                        0 => self.current_menu = "map_select".to_string(),
                        1 => self.current_menu = "1v1_map_select".to_string(),
                        2 => std::process::exit(0),
                        _ => {}
                    }
                } else if is_key_pressed(KeyCode::Enter) {
                    self.play_select_sound();
                    match self.main_menu_selected {
                        0 => self.current_menu = "map_select".to_string(),
                        1 => self.current_menu = "1v1_map_select".to_string(),
                        2 => std::process::exit(0),
                        _ => {}
                    }
                }
            }
            "map_select" => {
                if is_key_pressed(KeyCode::Down) {
                    self.map_menu_selected =
                        (self.map_menu_selected + 1) % self.available_maps.len();
                }
                if is_key_pressed(KeyCode::Up) {
                    self.map_menu_selected = if self.map_menu_selected == 0 {
                        self.available_maps.len() - 1
                    } else {
                        self.map_menu_selected - 1
                    };
                }

                if let Some(clicked_idx) = self.check_menu_click(self.available_maps.len()) {
                    self.map_menu_selected = clicked_idx;
                    let map_name = &self.available_maps[self.map_menu_selected];
                    println!("[Menu] Selected map: {}", map_name);
                    return Some(GameState::new_async(map_name).await);
                } else if is_key_pressed(KeyCode::Enter) {
                    let map_name = &self.available_maps[self.map_menu_selected];
                    println!("[Menu] Selected map: {}", map_name);
                    return Some(GameState::new_async(map_name).await);
                } else if is_key_pressed(KeyCode::Escape) {
                    self.current_menu = "main".to_string();
                }
            }
            "1v1_map_select" => {
                if is_key_pressed(KeyCode::Down) {
                    self.map_menu_selected =
                        (self.map_menu_selected + 1) % self.available_maps.len();
                }
                if is_key_pressed(KeyCode::Up) {
                    self.map_menu_selected = if self.map_menu_selected == 0 {
                        self.available_maps.len() - 1
                    } else {
                        self.map_menu_selected - 1
                    };
                }

                if let Some(clicked_idx) = self.check_1v1_menu_click() {
                    self.map_menu_selected = clicked_idx;
                    let map_name = &self.available_maps[self.map_menu_selected];
                    println!("[Menu] Selected 1v1 map: {}", map_name);
                    let mut gs = GameState::new_async(map_name).await;
                    gs.is_local_multiplayer = true;
                    return Some(gs);
                } else if is_key_pressed(KeyCode::Enter) {
                    let map_name = &self.available_maps[self.map_menu_selected];
                    println!("[Menu] Selected 1v1 map: {}", map_name);
                    let mut gs = GameState::new_async(map_name).await;
                    gs.is_local_multiplayer = true;
                    return Some(gs);
                } else if is_key_pressed(KeyCode::Escape) {
                    self.current_menu = "main".to_string();
                }
            }
            "settings" => {
                if !self.available_models.is_empty() {
                    if is_key_pressed(KeyCode::Down) {
                        self.model_menu_selected =
                            (self.model_menu_selected + 1) % self.available_models.len();
                    }
                    if is_key_pressed(KeyCode::Up) {
                        self.model_menu_selected = if self.model_menu_selected == 0 {
                            self.available_models.len() - 1
                        } else {
                            self.model_menu_selected - 1
                        };
                    }

                    if let Some(clicked_idx) = self.check_settings_menu_click() {
                        self.model_menu_selected = clicked_idx;
                        self.play_select_sound();
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.current_menu = "main".to_string();
                }
            }
            _ => {}
        }

        None
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn render(&self) {
        let main_menu_items = ["DEATHMATCH", "HOTSEAT", "QUIT"];
        let hover_idx = self.get_hovered_item_index();

        // Draw background for all menus
        render::menu_shader::draw_menu_background(self.time);

        match self.current_menu.as_str() {
            "main" => {
                render::menu_shader::draw_menu_items(
                    self.main_menu_selected,
                    &main_menu_items,
                    self.logo_texture.as_ref(),
                );
            }
            "map_select" => {
                let map_names: Vec<&str> = self.available_maps.iter().map(|s| s.as_str()).collect();
                render::draw_map_select_menu(self.map_menu_selected, &map_names, hover_idx);
            }
            "1v1_map_select" => {
                let map_names: Vec<&str> = self.available_maps.iter().map(|s| s.as_str()).collect();
                render::draw_1v1_map_select_menu(self.map_menu_selected, &map_names, hover_idx);
            }
            "settings" => {
                let model_names: Vec<&str> =
                    self.available_models.iter().map(|s| s.as_str()).collect();
                let selected_model_name = self
                    .available_models
                    .get(self.model_menu_selected)
                    .map(|s| s.as_str())
                    .unwrap_or("sarge");
                render::draw_settings_menu(
                    self.model_menu_selected,
                    &model_names,
                    selected_model_name,
                    hover_idx,
                );
            }
            _ => {}
        }
    }

    fn get_hovered_item_index(&self) -> Option<usize> {
        let mouse_pos = mouse_position();
        let w = screen_width();
        let h = screen_height();

        match self.current_menu.as_str() {
            "main" => {
                let main_menu_items = ["DEATHMATCH", "HOTSEAT", "QUIT"];
                let item_h = 54.0;
                let start_y = h * 0.5 - (main_menu_items.len() as f32 * (item_h + 12.0)) * 0.5;
                let right_margin = 100.0;

                for i in 0..main_menu_items.len() {
                    let y = start_y + (i as f32) * (item_h + 12.0);
                    let size = if i == self.main_menu_selected {
                        36.0
                    } else {
                        30.0
                    };
                    let text_width = crate::render::measure_q3_banner_string(
                        &main_menu_items[i].to_uppercase(),
                        size,
                    );
                    let x = w - text_width - right_margin;

                    if mouse_pos.0 >= x
                        && mouse_pos.0 <= w - right_margin
                        && mouse_pos.1 >= y
                        && mouse_pos.1 <= y + item_h
                    {
                        return Some(i);
                    }
                }
            }
            "map_select" => {
                let num_items = self.available_maps.len();
                let item_h = 54.0;
                let item_w = 400.0;
                let start_y = h * 0.5 - (num_items as f32 * (item_h + 12.0)) * 0.5;

                for i in 0..num_items {
                    let y = start_y + (i as f32) * (item_h + 12.0);
                    let x = w * 0.5 - item_w * 0.5;
                    if mouse_pos.0 >= x
                        && mouse_pos.0 <= x + item_w
                        && mouse_pos.1 >= y
                        && mouse_pos.1 <= y + item_h
                    {
                        return Some(i);
                    }
                }
            }
            "1v1_map_select" => {
                let num_items = self.available_maps.len();
                let item_h = 54.0;
                let item_w = 400.0;
                let start_y = h * 0.5 - (num_items as f32 * (item_h + 12.0)) * 0.5 + 20.0;

                for i in 0..num_items {
                    let y = start_y + (i as f32) * (item_h + 12.0);
                    let x = w * 0.5 - item_w * 0.5;
                    if mouse_pos.0 >= x
                        && mouse_pos.0 <= x + item_w
                        && mouse_pos.1 >= y
                        && mouse_pos.1 <= y + item_h
                    {
                        return Some(i);
                    }
                }
            }
            "settings" => {
                let max_visible = 6;
                let item_h = 54.0;
                let item_w = 400.0;

                let scroll_offset = if self.model_menu_selected >= max_visible {
                    self.model_menu_selected - max_visible + 1
                } else {
                    0
                };

                let num_visible = (self.available_models.len() - scroll_offset).min(max_visible);
                let start_y = h * 0.5 - (num_visible as f32 * (item_h + 12.0)) * 0.5 + 40.0;

                for idx in 0..num_visible {
                    let orig_i = scroll_offset + idx;
                    let y = start_y + (idx as f32) * (item_h + 12.0);
                    let x = w * 0.5 - item_w * 0.5;
                    if mouse_pos.0 >= x
                        && mouse_pos.0 <= x + item_w
                        && mouse_pos.1 >= y
                        && mouse_pos.1 <= y + item_h
                    {
                        return Some(orig_i);
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn play_move_sound(&self) {
        if let Some(sound) = &self.menu_move_sound {
            play_sound(sound, PlaySoundParams::default());
        }
    }

    fn play_select_sound(&self) {
        if let Some(sound) = &self.menu_select_sound {
            play_sound(sound, PlaySoundParams::default());
        }
    }

    pub fn get_selected_model_index(&self) -> usize {
        self.model_menu_selected
    }

    fn check_menu_click(&self, num_items: usize) -> Option<usize> {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return None;
        }

        let mouse_pos = mouse_position();
        let w = screen_width();
        let h = screen_height();

        let main_menu_items = ["DEATHMATCH", "HOTSEAT", "QUIT"];
        let item_h = 54.0;
        let start_y = h * 0.5 - (num_items as f32 * (item_h + 12.0)) * 0.5;
        let right_margin = 100.0;

        for i in 0..num_items {
            let y = start_y + (i as f32) * (item_h + 12.0);
            let size = if i == self.main_menu_selected {
                36.0
            } else {
                30.0
            };
            let text_width =
                crate::render::measure_q3_banner_string(&main_menu_items[i].to_uppercase(), size);
            let x = w - text_width - right_margin;

            if mouse_pos.0 >= x
                && mouse_pos.0 <= w - right_margin
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + item_h
            {
                return Some(i);
            }
        }

        None
    }

    fn check_1v1_menu_click(&self) -> Option<usize> {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return None;
        }

        let mouse_pos = mouse_position();
        let w = screen_width();
        let h = screen_height();

        let num_items = self.available_maps.len();
        let item_h = 54.0;
        let item_w = 400.0;
        let start_y = h * 0.5 - (num_items as f32 * (item_h + 12.0)) * 0.5 + 20.0;

        for i in 0..num_items {
            let y = start_y + (i as f32) * (item_h + 12.0);
            let x = w * 0.5 - item_w * 0.5;

            if mouse_pos.0 >= x
                && mouse_pos.0 <= x + item_w
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + item_h
            {
                return Some(i);
            }
        }

        None
    }

    fn check_settings_menu_click(&self) -> Option<usize> {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return None;
        }

        let mouse_pos = mouse_position();
        let w = screen_width();
        let h = screen_height();

        let max_visible = 6;
        let item_h = 54.0;
        let item_w = 400.0;

        let scroll_offset = if self.model_menu_selected >= max_visible {
            self.model_menu_selected - max_visible + 1
        } else {
            0
        };

        let num_visible = (self.available_models.len() - scroll_offset).min(max_visible);
        let start_y = h * 0.5 - (num_visible as f32 * (item_h + 12.0)) * 0.5 + 40.0;

        for idx in 0..num_visible {
            let orig_i = scroll_offset + idx;
            let y = start_y + (idx as f32) * (item_h + 12.0);
            let x = w * 0.5 - item_w * 0.5;

            if mouse_pos.0 >= x
                && mouse_pos.0 <= x + item_w
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + item_h
            {
                return Some(orig_i);
            }
        }

        None
    }
}

fn list_available_models() -> Vec<String> {
    #[cfg(target_arch = "wasm32")]
    {
        vec![
            "anarki".to_string(),
            "biker".to_string(),
            "bones".to_string(),
            "crash".to_string(),
            "doom".to_string(),
            "grunt".to_string(),
            "hunter".to_string(),
            "keel".to_string(),
            "klesk".to_string(),
            "lucy".to_string(),
            "major".to_string(),
            "mynx".to_string(),
            "orbb".to_string(),
            "ranger".to_string(),
            "razor".to_string(),
            "sarge".to_string(),
            "slash".to_string(),
            "sorlag".to_string(),
            "tankjr".to_string(),
            "uriel".to_string(),
            "visor".to_string(),
            "xaero".to_string(),
        ]
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let base = "q3-resources/models/players";
        let mut out = Vec::new();
        if let Ok(dir) = std::fs::read_dir(base) {
            for entry in dir.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let lower = format!("{}/{}/lower.md3", base, name);
                        let upper = format!("{}/{}/upper.md3", base, name);
                        let head = format!("{}/{}/head.md3", base, name);
                        if std::path::Path::new(&lower).exists()
                            && std::path::Path::new(&upper).exists()
                            && std::path::Path::new(&head).exists()
                        {
                            out.push(name);
                        }
                    }
                }
            }
        }
        out.sort();
        out
    }
}

fn list_available_maps() -> Vec<String> {
    #[cfg(target_arch = "wasm32")]
    {
        vec![
            "0-arena".to_string(),
            "1-arena".to_string(),
            "2-arena".to_string(),
            "my_arena".to_string(),
            "new_map".to_string(),
        ]
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let maps_dir = "maps";
        let mut maps = Vec::new();

        println!("[MENU] Scanning maps directory: {}", maps_dir);

        match std::fs::read_dir(maps_dir) {
            Ok(dir) => {
                let mut file_count = 0;
                for entry in dir.flatten() {
                    file_count += 1;
                    if let Ok(ft) = entry.file_type() {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        println!(
                            "[MENU] Found file: {} (is_file: {})",
                            file_name,
                            ft.is_file()
                        );

                        if ft.is_file() {
                            if file_name.ends_with(".json")
                                && !file_name.ends_with("_navgraph.json")
                                && !file_name.ends_with("_defrag.json")
                            {
                                let map_name = file_name.trim_end_matches(".json").to_string();
                                println!("[MENU] Added map: {}", map_name);
                                maps.push(map_name);
                            }
                        }
                    }
                }
                println!("[MENU] Total files scanned: {}", file_count);
            }
            Err(e) => {
                println!("[MENU] ERROR: Failed to read maps directory: {}", e);
            }
        }

        if maps.is_empty() {
            println!("[MENU] No maps found, using defaults");
            maps.push("soldat".to_string());
            maps.push("q3dm6".to_string());
        }

        maps.sort();
        println!("[MENU] Total maps loaded: {} - {:?}", maps.len(), maps);
        maps
    }
}
