use macroquad::prelude::*;
use crate::game::{GameState, player::Player, weapon::Weapon};
use crate::input::{Input, LocalMultiplayerInput};
use crate::render::Camera;
use crate::audio;
use crate::weapon_handler::WeaponHandler;
use crate::bot_handler::BotHandler;
use crate::hud_scoreboard::HudScoreboard;
use crate::profiler;
use crate::profiler_display;

pub struct GameLoop {
    pub game_state: GameState,
    pub input: Input,
    pub local_mp_input: LocalMultiplayerInput,
    pub camera: Camera,
    pub audio: audio::AudioSystem,
    pub perf_mode: bool,
    pub selected_model_idx: usize,
    pub available_models: Vec<String>,
    pub frame_count: u32,
    pub fps_display: i32,
    pub last_profiler_print: f64,
    pub last_time: f64,
    pub fps_samples: Vec<f64>,
    pub last_fps_log: f64,
    pub fps_display_samples: Vec<f64>,
}


impl GameLoop {
    pub async fn new(
        game_state: GameState,
        selected_model_idx: usize,
        available_models: Vec<String>,
    ) -> Self {
        let audio = audio::init_audio().await;
        let input = Input::new();
        let local_mp_input = LocalMultiplayerInput::new();
        let camera = Camera::new();

        Self {
            game_state,
            input,
            local_mp_input,
            camera,
            audio,
            perf_mode: false,
            selected_model_idx,
            available_models,
            frame_count: 0,
            fps_display: 0,
            last_profiler_print: get_time(),
            last_time: get_time(),
            fps_samples: Vec::with_capacity(100),
            last_fps_log: get_time(),
            fps_display_samples: Vec::with_capacity(200),
        }
    }

    pub async fn initialize_game(&mut self) {
        eprintln!("[INIT] Starting game initialization...");
        self.game_state.weapon_hit_texture_cache.load_all().await;
        self.game_state.muzzle_flash_cache.load_all().await;
        self.setup_players().await;
        eprintln!("[INIT] About to preload assets...");
        self.preload_assets().await;
        eprintln!("[INIT] Game initialization complete!");
    }

    async fn setup_players(&mut self) {
        let env_skin = std::env::var("SAS_PLAYER_SKIN").unwrap_or_else(|_| "default".to_string());

        if self.game_state.is_local_multiplayer {
            self.setup_local_multiplayer_players().await;
        } else {
            self.setup_single_player_and_bots().await;
        }

        self.preload_player_models(&env_skin).await;
    }

    async fn setup_local_multiplayer_players(&mut self) {
        let mut player1 = Player::new(1, "Player 1".to_string(), false);
        let model_cvar = crate::cvar::get_cvar_string("cg_model");
        player1.model = if !model_cvar.is_empty() {
            model_cvar
        } else {
            self.available_models.get(self.selected_model_idx)
                .cloned()
                .unwrap_or_else(|| "sarge".to_string())
        };

        let mut player2 = Player::new(2, "Player 2".to_string(), false);
        let model2_cvar = crate::cvar::get_cvar_string("cg_model2");
        player2.model = if !model2_cvar.is_empty() {
            model2_cvar
        } else {
            self.available_models.get(self.selected_model_idx.saturating_add(1))
                .cloned()
                .unwrap_or_else(|| "visor".to_string())
        };

        let spawn_points = &self.game_state.map.spawn_points;
        if spawn_points.len() >= 2 {
            let sp1 = &spawn_points[0];
            let sp2 = &spawn_points[1];
            player1.spawn(sp1.x, sp1.y, &self.game_state.map);
            player2.spawn(sp2.x, sp2.y, &self.game_state.map);
        } else {
            let (spawn_x, spawn_y) = self.game_state.map.find_safe_spawn_position();
            player1.spawn(spawn_x, spawn_y, &self.game_state.map);
            player2.spawn(spawn_x + 100.0, spawn_y, &self.game_state.map);
        }

        player1.has_weapon = [true, true, false, false, false, false, true, false, false];
        player1.ammo = [255, 100, 0, 0, 0, 0, 50, 0, 0];
        player1.weapon = Weapon::Railgun;

        player2.has_weapon = [true, true, false, false, false, false, false, false, false];
        player2.ammo = [255, 100, 0, 0, 0, 0, 0, 0, 0];
        player2.weapon = Weapon::MachineGun;

        self.game_state.players.push(player1);
        self.game_state.players.push(player2);
    }

    async fn setup_single_player_and_bots(&mut self) {
        let mut local_player = Player::new(1, "slash".to_string(), false);
        let model_cvar = crate::cvar::get_cvar_string("cg_model");
        local_player.model = if !model_cvar.is_empty() {
            model_cvar
        } else {
            "slash".to_string()
        };

        let (spawn_x, spawn_y) = if !self.game_state.map.spawn_points.is_empty() {
            let sp = &self.game_state.map.spawn_points[0];
            (sp.x, sp.y)
        } else {
            self.game_state.map.find_safe_spawn_position()
        };

        local_player.spawn(spawn_x, spawn_y, &self.game_state.map);
        local_player.has_weapon = [true, true, false, false, false, false, true, false, false];
        local_player.ammo = [255, 100, 0, 0, 0, 0, 50, 0, 0];
        local_player.weapon = Weapon::Railgun;
        self.game_state.players.push(local_player);

        if self.game_state.story_mode.is_some() {
            return;
        }

        let all_bot_skins = [
            "visor", "sarge", "grunt", "razor", "doom", "hunter", "keel", "bones",
            "anarki", "biker", "bitterman", "bones", "crash", "orbb", "slash",
            "uriel", "xaero", "major", "tankjr", "lucy", "sorlag"
        ];
        
        let num_bots = (self.game_state.map.spawn_points.len() - 1).min(all_bot_skins.len());
        let mut used_skins = Vec::new();
        
        for i in 0..num_bots {
            let mut skin;
            loop {
                let idx = rand::gen_range(0, all_bot_skins.len());
                skin = all_bot_skins[idx];
                if !used_skins.contains(&skin) {
                    used_skins.push(skin);
                    break;
                }
            }
            
            let mut bot = Player::new((i + 2) as u16, skin.to_string(), true);
            bot.model = skin.to_string();
            let spawn_point = &self.game_state.map.spawn_points[(i + 1) % self.game_state.map.spawn_points.len()];
            bot.spawn(spawn_point.x, spawn_point.y, &self.game_state.map);
            bot.has_weapon = [true, true, false, false, false, false, false, false, false];
            bot.ammo = [255, 50, 0, 0, 0, 0, 0, 0, 0];
            self.game_state.players.push(bot);
        }
    }

    async fn preload_player_models(&mut self, env_skin: &str) {
        use std::collections::HashSet;
        let mut unique_models: HashSet<String> = HashSet::new();
        for p in &self.game_state.players {
            unique_models.insert(p.model.clone());
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            for model_name in unique_models.iter() {
                let _ = self.game_state.model_cache.get_or_load_async(model_name).await;
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            for model_name in unique_models.iter() {
                let _ = self.game_state.model_cache.get_or_load(model_name);
            }
        }
        
        for model_name in unique_models.iter() {
            if let Some(model) = self.game_state.model_cache.get_mut(model_name) {
                model.load_textures(model_name, env_skin).await;
            }
        }
        
        for model_name in unique_models.iter() {
            self.audio.load_player_sounds(model_name).await;
        }
    }

    async fn preload_assets(&mut self) {
        self.preload_item_models().await;
        self.preload_projectile_models().await;
        self.preload_weapon_models().await;
        self.game_state.tile_textures.load_default_textures().await;
        
        eprintln!("[PRELOAD] Loading tile shaders...");
        self.load_tile_shaders().await;
        eprintln!("[PRELOAD] ✓ Tile shaders loaded!");
        
        eprintln!("[PRELOAD] Loading border textures...");
        self.game_state.border_renderer.load_border_textures().await;
        eprintln!("[PRELOAD] ✓ Border textures loaded!");
    }

    async fn preload_item_models(&mut self) {
        use crate::game::item_model::ItemModelType;
        
        eprintln!("[PRELOAD] Starting item models preload...");
        
        let item_types = [
            ItemModelType::HealthMedium,
            ItemModelType::HealthLarge,
            ItemModelType::HealthMega,
            ItemModelType::ArmorYellow,
            ItemModelType::ArmorRed,
            ItemModelType::WeaponShotgun,
            ItemModelType::WeaponGrenadeLauncher,
            ItemModelType::WeaponRocketLauncher,
            ItemModelType::WeaponRailgun,
            ItemModelType::WeaponPlasmagun,
            ItemModelType::WeaponBFG,
            ItemModelType::PowerupQuad,
            ItemModelType::PowerupRegen,
            ItemModelType::PowerupBattleSuit,
            ItemModelType::PowerupFlight,
            ItemModelType::PowerupHaste,
            ItemModelType::PowerupInvis,
        ];

        for item_type in item_types {
            match self.game_state.item_model_cache.load(item_type).await {
                Ok(_) => eprintln!("[PRELOAD] ✓ Loaded {:?}", item_type),
                Err(e) => eprintln!("[PRELOAD] ✗ Failed to load {:?}: {}", item_type, e),
            }
        }
        
        eprintln!("[PRELOAD] Item models preload complete!");
    }

    async fn preload_projectile_models(&mut self) {
        self.game_state.projectile_model_cache
            .get_or_load_model(crate::game::projectile_model_cache::ProjectileModelType::Rocket);
    }

    async fn preload_weapon_models(&mut self) {
        let weapons = [
            Weapon::Gauntlet,
            Weapon::MachineGun,
            Weapon::Shotgun,
            Weapon::GrenadeLauncher,
            Weapon::RocketLauncher,
            Weapon::Lightning,
            Weapon::Railgun,
            Weapon::Plasmagun,
            Weapon::BFG,
        ];

        for weapon in weapons {
            self.game_state.weapon_model_cache.preload(weapon).await;
        }
    }
    
    async fn load_tile_shaders(&mut self) {
        if let Ok(content) = std::fs::read_to_string("tile_shaders.json") {
            if let Ok(shaders) = serde_json::from_str::<Vec<crate::game::tile_shader::TileShader>>(&content) {
                eprintln!("[Shaders] Loading {} custom shaders from JSON", shaders.len());
                
                for shader in shaders {
                    if !shader.base_texture.is_empty() {
                        self.game_state.shader_renderer.load_texture(&shader.base_texture).await;
                    }
                    
                    for stage in &shader.stages {
                        if !stage.texture_path.is_empty() {
                            self.game_state.shader_renderer.load_texture(&stage.texture_path).await;
                        }
                    }
                    
                    self.game_state.shader_renderer.add_shader(shader);
                }
                
                eprintln!("[Shaders] ✓ Custom shaders loaded!");
            }
        } else {
            eprintln!("[Shaders] No tile_shaders.json found, skipping");
        }
        
        use crate::game::q3_shader_parser::Q3ShaderParser;
        let mut parser = Q3ShaderParser::new();
        parser.load_all_shader_files();
        
        eprintln!("[Shaders] Loaded {} Q3 shaders from .shader files", parser.get_all_shaders().len());
        
        for (_name, shader) in parser.get_all_shaders() {
            if !shader.base_texture.is_empty() {
                self.game_state.shader_renderer.load_texture(&shader.base_texture).await;
            }
            
            for stage in &shader.stages {
                if !stage.texture_path.is_empty() {
                    self.game_state.shader_renderer.load_texture(&stage.texture_path).await;
                }
            }
            
            self.game_state.shader_renderer.add_shader(shader.clone());
        }
    }

    pub async fn run(&mut self, console: &mut crate::console::Console, ignore_mouse_delta: bool) -> bool {
        console.is_connected_to_server = self.game_state.is_connected_to_server();
        
        if let Some(bot_model) = console.bot_add_request.take() {
            self.add_bot(&bot_model).await;
        }
        
        if console.bot_remove_request {
            console.bot_remove_request = false;
            self.decrease_bot_count();
        }
        
        if console.bot_afk_request {
            console.bot_afk_request = false;
            self.toggle_bot_afk();
        }
        
        if console.server_bot_add_request {
            console.server_bot_add_request = false;
            if let Err(e) = self.game_state.send_chat("addbot".to_string()) {
                console.print(&format!("Failed to request bot: {}\n", e));
            }
        }

        if let Some((server, player_name)) = console.connect_request.take() {
            if let Err(e) = self.game_state.connect_to_server(&server, &player_name) {
                console.print(&format!("Failed to connect: {}\n", e));
            }
        }

        if console.disconnect_request {
            console.disconnect_request = false;
            self.game_state.disconnect_from_server();
            console.print("Disconnected from server\n");
        }
        
        if console.net_stats_toggle {
            console.net_stats_toggle = false;
            self.game_state.net_hud.toggle_stats();
        }
        
        if console.net_graph_toggle {
            console.net_graph_toggle = false;
            self.game_state.net_hud.toggle_graph();
        }
        
        if console.net_showmiss_toggle {
            console.net_showmiss_toggle = false;
            self.game_state.net_hud.toggle_prediction_errors();
        }
        
        if console.end_match_request {
            console.end_match_request = false;
            self.game_state.end_match();
            console.print("Match ended\n");
        }

        self.game_state.update_network();
        
        let current_time = get_time();
        let mut dt = (current_time - self.last_time) as f32;
        self.last_time = current_time;
        if dt > 0.05 {
            dt = 0.05;
        }

        let t_frame_start = get_time();
        
        //self.fps_samples.push(current_time);
        self.fps_display_samples.push(current_time);
        
        self.fps_display_samples.retain(|&t| current_time - t <= 1.0);
        if self.fps_display_samples.len() > 1 {
            let time_span = self.fps_display_samples[self.fps_display_samples.len() - 1] 
                - self.fps_display_samples[0];
            if time_span > 0.0 {
                self.fps_display = ((self.fps_display_samples.len() - 1) as f64 / time_span) as i32;
            }
        }

        if !console.is_open() {
            self.handle_input(dt, ignore_mouse_delta);
        }
        
        if is_key_pressed(KeyCode::Escape) && !console.is_open() {
            return false;
        }
        
        if is_key_pressed(KeyCode::GraveAccent) {
            return true;
        }
        
        if !console.is_open() {
            self.handle_defrag_keys();
            self.handle_debug_keys().await;
        }
        
        self.handle_camera(dt);
        
        let shoot_actions = if !self.game_state.game_results.show {
            if self.game_state.is_multiplayer {
                let shoot_data = if let Some(ref client) = self.game_state.network_client {
                    client.player_id().and_then(|player_id| {
                        WeaponHandler::handle_weapon_input_multiplayer(
                            &mut self.game_state,
                            &self.input,
                            player_id,
                        )
                    })
                } else {
                    None
                };

                if let Some((shoot_x, shoot_y, angle, weapon)) = shoot_data {
                    if let Some(ref mut network_client) = self.game_state.network_client {
                        if let Some(player_id) = network_client.player_id() {
                            let msg = crate::network::NetMessage::PlayerShoot {
                                player_id,
                                weapon,
                                origin: (shoot_x, shoot_y),
                                direction: angle,
                            };
                            network_client.send_message(msg).ok();
                        }
                    }
                }
                Vec::new()
            } else {
                WeaponHandler::handle_shooting(
                &mut self.game_state,
                &self.input,
                &self.local_mp_input,
                )
            }
        } else {
            Vec::new()
        };

        if !self.game_state.game_results.show {
            WeaponHandler::handle_weapon_switching(
                &mut self.game_state,
                &self.input,
                &self.local_mp_input,
            );
        }

        let bot_actions = if !console.is_open() && !self.game_state.game_results.show {
            BotHandler::process_bot_ai(&mut self.game_state, dt)
        } else {
            Vec::new()
        };
        
        let mut all_actions = shoot_actions;
        all_actions.extend(bot_actions);
        
        if !self.game_state.game_results.show {
            WeaponHandler::process_weapon_fire(&mut self.game_state, all_actions);
        }

        if !console.is_open() {
            self.update_game_state(dt);
        }
        
        if let Some(new_map) = self.check_story_level_transition().await {
            return self.load_next_story_level(&new_map).await;
        }
        
        self.process_audio();
        self.render_frame(dt, t_frame_start);

        self.frame_count += 1;
        
        if current_time - self.last_fps_log >= 0.1 {
            self.fps_samples.retain(|&t| current_time - t <= 0.1);
            
            if self.fps_samples.len() > 1 {
                let time_span = self.fps_samples[self.fps_samples.len() - 1] - self.fps_samples[0];
                let avg_fps = if time_span > 0.0 {
                    (self.fps_samples.len() - 1) as f64 / time_span
                } else {
                    0.0
                };
                println!("[FPS] {:.1} fps (avg over {:.0}ms)", avg_fps, time_span * 1000.0);
            }
            
            self.last_fps_log = current_time;
        }
        
        true
    }

    fn handle_defrag_keys(&mut self) {
        if let Some(defrag) = &mut self.game_state.defrag_mode {
            if is_key_pressed(KeyCode::R) {
                defrag.reset();
                println!("[Defrag] Run reset!");
                
                if let Some(player) = self.game_state.players.get_mut(0) {
                    let (spawn_x, spawn_y) = defrag.start_pos;
                    player.x = spawn_x;
                    player.y = spawn_y;
                    player.vel_x = 0.0;
                    player.vel_y = 0.0;
                    player.dead = false;
                    player.gibbed = false;
                    player.health = 100;
                }
            }
            
            if is_key_pressed(KeyCode::Backspace) && !defrag.run_finished {
                if let Some(player) = self.game_state.players.get_mut(0) {
                    let (spawn_x, spawn_y) = defrag.get_respawn_position();
                    player.x = spawn_x;
                    player.y = spawn_y;
                    player.vel_x = 0.0;
                    player.vel_y = 0.0;
                    println!("[Defrag] Respawned at checkpoint");
                }
            }
        }
    }
    
    fn handle_input(&mut self, dt: f32, ignore_mouse_delta: bool) {
        let _scope = profiler::scope("input");
        
        if self.game_state.is_multiplayer {
            self.input.update(ignore_mouse_delta);
            let cmd = crate::game::usercmd::UserCmd::from_input(&self.input, screen_width(), screen_height());
            
            if let Some(ref mut network_client) = self.game_state.network_client {
                if let Some(player_id) = network_client.player_id() {
                    let move_forward = if self.input.jump { 1.0 } else if self.input.crouch { -1.0 } else { 0.0 };
                    let move_right = cmd.right;
                    
                    let mut buttons = 0u32;
                    if self.input.shoot { buttons |= 1; }
                    if self.input.jump { buttons |= 2; }
                    if self.input.crouch { buttons |= 4; }
                    
                    let angle = self.input.aim_angle;
                    
                    static mut LAST_INPUT_PRINT: f64 = 0.0;
                    static mut LAST_NONZERO_INPUT: f64 = 0.0;
                    unsafe {
                        if move_right.abs() > 0.1 || buttons != 0 {
                            LAST_NONZERO_INPUT = macroquad::prelude::get_time();
                        }
                        if macroquad::prelude::get_time() - LAST_INPUT_PRINT > 2.0 && 
                           macroquad::prelude::get_time() - LAST_NONZERO_INPUT < 0.5 {
                            println!("[INPUT] right={:.1} buttons={}", move_right, buttons);
                            LAST_INPUT_PRINT = macroquad::prelude::get_time();
                        }
                    }
                    
                    network_client.send_input(move_forward, move_right, angle, buttons).ok();
                    
                    if let Some(predicted) = network_client.predict_local_player(&self.game_state.map) {
                        if let Some(player) = self.game_state.players.iter_mut().find(|p| p.id == player_id) {
                            player.x = predicted.x;
                            player.y = predicted.y;
                            player.vel_x = predicted.vel_x;
                            player.vel_y = predicted.vel_y;
                            player.angle = angle;
                            player.was_in_air = predicted.was_in_air;
                            player.crouch = self.input.crouch;

                            if let Some((corr_x, corr_y)) = network_client.get_prediction_mut().apply_error_correction(dt) {
                                player.x += corr_x;
                                player.y += corr_y;
                            }

                            let moving = player.vel_x.abs() > 0.5;
                            let shooting = self.input.shoot || player.refire > 0.0;
                            player.animation.update(!player.was_in_air, moving, shooting, player.vel_x.abs());
                            
                            if predicted.hit_jumppad {
                                self.game_state.audio_events.push(crate::audio::events::AudioEvent::JumpPad { x: player.x });
                            }
                            if predicted.landed {
                                self.game_state.audio_events.push(crate::audio::events::AudioEvent::PlayerLand { x: player.x });
                            }
                        }
                    }
                }
            }
            return;
        }
        
        if !self.game_state.game_results.show {
            if self.game_state.is_local_multiplayer {
                self.local_mp_input.update(ignore_mouse_delta);
                
                for (player_idx, player_input) in [&self.local_mp_input.player1, &self.local_mp_input.player2]
                    .iter()
                    .enumerate()
                {
                    if let Some(player) = self.game_state.players.get_mut(player_idx) {
                        player.manual_flip_x = Some(player_input.flip_x);
                        let cmd = crate::game::usercmd::UserCmd::from_player_input(player_input, player.x, player.y);
                        let pmove_events = player.pmove(&cmd, dt, &self.game_state.map);
                        for event in pmove_events {
                            self.game_state.audio_events.push(event);
                        }
                    }
                }
            } else {
                self.input.update(ignore_mouse_delta);
                let cmd = crate::game::usercmd::UserCmd::from_input(&self.input, screen_width(), screen_height());
                
                if let Some(player) = self.game_state.players.get_mut(0) {
                    let pmove_events = player.pmove(&cmd, dt, &self.game_state.map);
                    for event in pmove_events {
                        self.game_state.audio_events.push(event);
                    }
                }
            }
        }
    }

    async fn handle_debug_keys(&mut self) {
        if is_key_pressed(KeyCode::F9) {
            self.perf_mode = !self.perf_mode;
        }

        if is_key_pressed(KeyCode::F7) {
            self.game_state.debug_md3 = !self.game_state.debug_md3;
        }

        if is_key_pressed(KeyCode::F8) {
            profiler::toggle();
        }

        if is_key_pressed(KeyCode::F3) {
            self.game_state.debug_hitboxes = !self.game_state.debug_hitboxes;
            println!("[Debug] Hitbox debug: {}", if self.game_state.debug_hitboxes { "ON" } else { "OFF" });
        }

        if is_key_pressed(KeyCode::F4) {
            self.decrease_bot_count();
        }

        if is_key_pressed(KeyCode::F10) {
            self.game_state.use_item_icons = !self.game_state.use_item_icons;
            println!("[Debug] Item icons mode: {}", if self.game_state.use_item_icons { "ON (2D icons)" } else { "OFF (3D models)" });
        }

        if is_key_pressed(KeyCode::F11) {
            self.game_state.disable_shadows = !self.game_state.disable_shadows;
            println!("[Perf] Shadows: {}", if self.game_state.disable_shadows { "OFF" } else { "ON" });
        }

        if is_key_pressed(KeyCode::F12) {
            self.game_state.disable_dynamic_lights = !self.game_state.disable_dynamic_lights;
            println!("[Perf] Dynamic lights: {}", if self.game_state.disable_dynamic_lights { "OFF" } else { "ON" });
        }

        if is_key_pressed(KeyCode::F6) {
            self.game_state.disable_particles = !self.game_state.disable_particles;
            println!("[Perf] Particles: {}", if self.game_state.disable_particles { "OFF" } else { "ON" });
        }

        if is_key_pressed(KeyCode::F2) {
            self.game_state.disable_deferred = !self.game_state.disable_deferred;
            println!("[Perf] Deferred rendering: {}", if self.game_state.disable_deferred { "OFF (forward)" } else { "ON" });
        }

        if is_key_pressed(KeyCode::F1) {
            self.game_state.render_scale = if self.game_state.render_scale > 1.5 { 1.0 } else { 2.0 };
            println!("[Perf] Render scale: {}x", self.game_state.render_scale);
        }

        if is_key_pressed(KeyCode::Key1) && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)) {
            self.game_state.cartoon_shader = !self.game_state.cartoon_shader;
            println!("[Visual] Cartoon shader: {}", if self.game_state.cartoon_shader { "ON" } else { "OFF" });
        }

        if let Some(new_model) = self.handle_model_switching() {
            if let Some(p) = self.game_state.players.get_mut(0) {
                p.model = new_model.clone();
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                let _ = self.game_state.model_cache.get_or_load_async(&new_model).await;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = self.game_state.model_cache.get_or_load(&new_model);
            }
            
            if let Some(model) = self.game_state.model_cache.get_mut(&new_model) {
                let env_skin = std::env::var("SAS_PLAYER_SKIN").unwrap_or_else(|_| "default".to_string());
                model.load_textures(&new_model, &env_skin).await;
            }
            self.audio.load_player_sounds(&new_model).await;
        }
    }

    fn handle_model_switching(&mut self) -> Option<String> {
        if is_key_pressed(KeyCode::F6) && !self.available_models.is_empty() {
            self.selected_model_idx = (self.selected_model_idx + 1) % self.available_models.len();
            return self.available_models.get(self.selected_model_idx).cloned();
        }
        
        if is_key_pressed(KeyCode::F5) && !self.available_models.is_empty() {
            self.selected_model_idx = if self.selected_model_idx == 0 {
                self.available_models.len() - 1
            } else {
                self.selected_model_idx - 1
            };
            return self.available_models.get(self.selected_model_idx).cloned();
        }
        
        None
    }

    fn handle_camera(&mut self, dt: f32) {
        if self.game_state.is_local_multiplayer && self.game_state.players.len() >= 2 {
            self.camera.tracking_projectile_id = None;
            self.camera.follow_two_players(
                self.game_state.players[0].x,
                self.game_state.players[0].y,
                self.game_state.players[1].x,
                self.game_state.players[1].y,
            );
        } else if let Some(player) = self.game_state.players.get(0) {
            if player.dead {
                if let Some(tracking_id) = self.camera.tracking_projectile_id {
                    if let Some(projectile) = self.game_state.projectiles.iter().find(|p| p.id == tracking_id && p.active) {
                        self.camera.follow_projectile(projectile.x, projectile.y);
                    } else {
                        self.camera.tracking_projectile_id = None;
                    }
                } else {
                    let player_projectile = self.game_state.projectiles.iter()
                        .find(|p| p.owner_id == player.id 
                            && p.active 
                            && matches!(p.weapon_type, crate::game::weapon::Weapon::RocketLauncher | crate::game::weapon::Weapon::GrenadeLauncher));
                    
                    if let Some(projectile) = player_projectile {
                        self.camera.tracking_projectile_id = Some(projectile.id);
                        self.camera.follow_projectile(projectile.x, projectile.y);
                    } else {
                        self.camera.follow(player.x, player.y);
                    }
                }
            } else {
                if self.camera.tracking_projectile_id.is_some() {
                    // Player alive, clear tracking
                }
                self.camera.tracking_projectile_id = None;
                self.camera.follow(player.x, player.y);
            }
        }

        self.camera.update(
            dt,
            self.game_state.map.width as f32,
            self.game_state.map.height as f32,
        );
    }

    fn update_game_state(&mut self, dt: f32) {
        let _scope = profiler::scope("game_update");
        self.game_state.update(dt);
    }

    fn process_audio(&mut self) {
        let listener_x = if let Some(player) = self.game_state.players.get(0) {
            player.x
        } else {
            self.camera.x
        };

        let audio_events = self.game_state.audio_events.drain();
        for event in audio_events {
            self.audio.process_event(&event, listener_x);
        }
    }

    fn render_frame(&mut self, dt: f32, t_frame_start: f64) {
        let model_cvar = crate::cvar::get_cvar_string("cg_model");
        if !model_cvar.is_empty() {
            if let Some(player) = self.game_state.players.get_mut(0) {
                if !player.is_bot && player.model != model_cvar {
                    player.model = model_cvar.clone();
                }
            }
        }
        
        if self.game_state.is_local_multiplayer {
            let model2_cvar = crate::cvar::get_cvar_string("cg_model2");
            if !model2_cvar.is_empty() {
                if let Some(player) = self.game_state.players.get_mut(1) {
                    if !player.is_bot && player.model != model2_cvar {
                        player.model = model2_cvar.clone();
                    }
                }
            }
        }
        
        self.game_state.render(self.camera.x, self.camera.y, self.camera.zoom);
        self.game_state.render_messages();
        
        {
            let _scope = profiler::scope("render_hud");
            HudScoreboard::render_hud(&self.game_state);
            HudScoreboard::render_crosshair(&self.game_state, &self.camera, &self.input);
            HudScoreboard::render_crosshair_local_mp(&self.game_state, &self.camera, &self.local_mp_input);
            HudScoreboard::render_scoreboard(&self.game_state);
            self.game_state.render_defrag_hud();
            
            if let Some(ref net_client) = self.game_state.network_client {
                self.game_state.net_hud.render(
                    net_client.get_stats(),
                    net_client.get_prediction().get_prediction_error()
                );
            }
        }
        
        profiler::end_frame();

        let frame_time = (get_time() - t_frame_start) * 1000.0;
        
        {
            let _scope = profiler::scope("render_debug_ui");
            let show_fps = crate::cvar::get_cvar_bool("cg_drawFPS");
            HudScoreboard::render_debug_info(
                &self.game_state,
                if show_fps { self.fps_display } else { 0 },
                self.perf_mode,
                dt,
                frame_time as f32,
            );

            profiler_display::draw_profiler();
            self.render_profiler_output();
        }
    }

    async fn add_bot(&mut self, model: &str) {
        if self.game_state.is_local_multiplayer {
            return;
        }

        let player_count = self.game_state.players.len();
        let bot_id = (player_count + 1) as u16;
        
        let mut bot = crate::game::player::Player::new(bot_id, model.to_string(), true);
        bot.model = model.to_string();
        
        let spawn_idx = player_count % self.game_state.map.spawn_points.len().max(1);
        if let Some(spawn_point) = self.game_state.map.spawn_points.get(spawn_idx) {
            bot.spawn(spawn_point.x, spawn_point.y, &self.game_state.map);
        } else {
            let (spawn_x, spawn_y) = self.game_state.map.find_safe_spawn_position();
            bot.spawn(spawn_x, spawn_y, &self.game_state.map);
        }
        
        bot.has_weapon = [true, true, false, false, false, false, false, false, false];
        bot.ammo = [255, 50, 0, 0, 0, 0, 0, 0, 0];
        
        #[cfg(target_arch = "wasm32")]
        {
            let _ = self.game_state.model_cache.get_or_load_async(model).await;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.game_state.model_cache.get_or_load(model);
        }
        
        if let Some(player_model) = self.game_state.model_cache.get_mut(model) {
            let env_skin = std::env::var("SAS_PLAYER_SKIN").unwrap_or_else(|_| "default".to_string());
            player_model.load_textures(model, &env_skin).await;
        }
        
        self.audio.load_player_sounds(model).await;
        
        self.game_state.players.push(bot);
        println!("[GameLoop] Added bot: {} (total bots: {})", model, self.game_state.players.iter().filter(|p| p.is_bot).count());
    }

    fn decrease_bot_count(&mut self) {
        if self.game_state.is_local_multiplayer {
            return;
        }

        let bot_count = self.game_state.players.iter().filter(|p| p.is_bot).count();
        if bot_count > 0 {
            if let Some(bot_index) = self.game_state.players.iter().rposition(|p| p.is_bot) {
                let bot_name = self.game_state.players[bot_index].name.clone();
                self.game_state.players.remove(bot_index);
                println!("[GameLoop] Removed bot: {} (F4). Remaining bots: {}", bot_name, bot_count - 1);
            }
        } else {
            println!("[GameLoop] No bots to remove (F4)");
        }
    }

    fn toggle_bot_afk(&mut self) {
        if self.game_state.is_local_multiplayer {
            return;
        }

        let mut toggled_count = 0;
        for player in &mut self.game_state.players {
            if player.is_bot {
                if let Some(ref mut ai) = player.bot_ai {
                    ai.afk = !ai.afk;
                    let status = if ai.afk { "AFK" } else { "Active" };
                    println!("[GameLoop] Bot {} is now {}", player.name, status);
                    toggled_count += 1;
                }
            }
        }

        if toggled_count == 0 {
            println!("[GameLoop] No bots to toggle AFK");
        }
    }

    fn render_profiler_output(&mut self) {
        let current_time = get_time();
        if profiler::is_enabled() && current_time - self.last_profiler_print >= 1.0 {
            self.last_profiler_print = current_time;
            let samples = profiler::get_samples();
            println!("\n=== PROFILER (F8) ===");
            println!("{:<20} {:>8} {:>8} {:>8} {:>8}", "Name", "Current", "Avg", "Min", "Max");
            println!("{}", "-".repeat(60));
            for (name, current, avg, min, max) in samples.iter().take(15) {
                let status = if *current > 5.0 {
                    "🔴"
                } else if *current > 2.0 {
                    "🟡"
                } else {
                    "🟢"
                };
                println!(
                    "{} {:<18} {:>7.2}ms {:>7.2}ms {:>7.2}ms {:>7.2}ms",
                    status, name, current, avg, min, max
                );
            }
            if let Some((_, total, _, _, _)) = samples.iter().find(|(n, _, _, _, _)| *n == "render_total") {
                println!("{}", "-".repeat(60));
                println!("Total render: {:.2}ms | Target: <8.33ms (120 FPS)", total);
                
                let shader_stats = profiler::get_shader_stats();
                if !shader_stats.is_empty() {
                    let total_draws: usize = shader_stats.iter().map(|(_, count)| count).sum();
                    println!("Draw calls: {}", total_draws);
                    println!("\n--- SHADER STATS ---");
                    println!("{:<25} {:>10}", "Shader", "Calls");
                    println!("{}", "-".repeat(40));
                    for (shader_name, count) in shader_stats.iter().take(15) {
                        println!("{:<25} {:>10}", shader_name, count);
                    }
                } else {
                    println!("Draw calls: 0");
                }
            }
            println!("======================\n");
        }
    }

    async fn check_story_level_transition(&mut self) -> Option<String> {
        if let Some(ref mut story) = self.game_state.story_mode {
            if story.next_level_ready {
                return story.advance_to_next_level();
            }
        }
        None
    }

    async fn load_next_story_level(&mut self, map_name: &str) -> bool {
        println!("[Story] Loading next level: {}", map_name);
        
        let story_backup = self.game_state.story_mode.clone();
        
        self.game_state = GameState::new_async(map_name).await;
        self.game_state.story_mode = story_backup;
        
        self.setup_players().await;
        self.preload_assets().await;
        
        true
    }
}
