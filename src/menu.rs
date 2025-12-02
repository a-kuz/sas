use crate::input::winit_input::{WinitInputState, KeyCode};
use crate::game::GameState;

pub struct MenuState {
    pub current_menu: String,
    pub main_menu_selected: usize,
    pub map_menu_selected: usize,
    pub model_menu_selected: usize,
    pub available_maps: Vec<String>,
    pub available_models: Vec<String>,
    pub time: f32,
    pub menu_move_sound: Option<()>,
    pub menu_select_sound: Option<()>,
    pub logo_texture: Option<()>,
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
            time: 0.0,
            menu_move_sound: None,
            menu_select_sound: None,
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

        self.menu_move_sound = None;
        self.menu_select_sound = None;

        self.logo_texture = None;
    }

    pub fn handle_input(&mut self, input_state: &WinitInputState) -> Option<GameState> {
        let main_menu_items = ["DEATHMATCH", "HOTSEAT", "QUIT"];

        match self.current_menu.as_str() {
            "main" => {
                if input_state.is_key_pressed(KeyCode::Down) {
                    self.main_menu_selected = (self.main_menu_selected + 1) % main_menu_items.len();
                }
                if input_state.is_key_pressed(KeyCode::Up) {
                    self.main_menu_selected = if self.main_menu_selected == 0 {
                        main_menu_items.len() - 1
                    } else {
                        self.main_menu_selected - 1
                    };
                }

                if input_state.is_key_pressed(KeyCode::Enter) || input_state.is_key_pressed(KeyCode::KpEnter) {
                    match self.main_menu_selected {
                        0 => {
                            self.current_menu = "map_select".to_string();
                        }
                        1 => {
                            self.current_menu = "1v1_map_select".to_string();
                        }
                        2 => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
            }
            "map_select" => {
                if input_state.is_key_pressed(KeyCode::Down) {
                    if !self.available_maps.is_empty() {
                        self.map_menu_selected = (self.map_menu_selected + 1) % self.available_maps.len();
                    }
                }
                if input_state.is_key_pressed(KeyCode::Up) {
                    if !self.available_maps.is_empty() {
                        self.map_menu_selected = if self.map_menu_selected == 0 {
                            self.available_maps.len() - 1
                        } else {
                            self.map_menu_selected - 1
                        };
                    }
                }
                if input_state.is_key_pressed(KeyCode::Enter) || input_state.is_key_pressed(KeyCode::KpEnter) {
                    if let Some(map_name) = self.available_maps.get(self.map_menu_selected) {
                        return Some(GameState::new(map_name));
                    }
                }
                if input_state.is_key_pressed(KeyCode::Escape) {
                    self.current_menu = "main".to_string();
                }
            }
            "1v1_map_select" => {
                if input_state.is_key_pressed(KeyCode::Down) {
                    if !self.available_maps.is_empty() {
                        self.map_menu_selected = (self.map_menu_selected + 1) % self.available_maps.len();
                    }
                }
                if input_state.is_key_pressed(KeyCode::Up) {
                    if !self.available_maps.is_empty() {
                        self.map_menu_selected = if self.map_menu_selected == 0 {
                            self.available_maps.len() - 1
                        } else {
                            self.map_menu_selected - 1
                        };
                    }
                }
                if input_state.is_key_pressed(KeyCode::Enter) || input_state.is_key_pressed(KeyCode::KpEnter) {
                    if let Some(map_name) = self.available_maps.get(self.map_menu_selected) {
                        return Some(GameState::new(map_name));
                    }
                }
                if input_state.is_key_pressed(KeyCode::Escape) {
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

    pub fn get_selected_model_index(&self) -> usize {
        self.model_menu_selected
    }

    pub fn render_wgpu(&self, renderer: &mut crate::wgpu_renderer::WgpuRenderer) {
        use std::sync::OnceLock;
        use std::sync::Mutex;
        static UI_RENDERER: OnceLock<Mutex<crate::wgpu_renderer::ui_renderer::UIRenderer>> = OnceLock::new();
        static FONT_TEXTURE: OnceLock<std::sync::Arc<crate::wgpu_renderer::texture::WgpuTexture>> = OnceLock::new();
        
        let (width, height) = renderer.get_viewport_size();
        let w = width as f32;
        let h = height as f32;

        if let Some(frame) = renderer.begin_frame() {
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Menu Render Encoder"),
            });

            let ui_renderer_mutex = UI_RENDERER.get_or_init(|| {
                Mutex::new(crate::wgpu_renderer::ui_renderer::UIRenderer::new(
                    renderer.device.clone(),
                    renderer.queue.clone(),
                    renderer.surface_config.format,
                ))
            });

            let mut ui_renderer = ui_renderer_mutex.lock().unwrap();

            if ui_renderer.font_texture.is_none() {
                if let Ok(img) = image::open("q3-resources/menu/art/font2_prop.png") {
                    let dyn_img = img.to_rgba8();
                    let wgpu_texture = std::sync::Arc::new(
                        crate::wgpu_renderer::texture::WgpuTexture::from_image(
                            &renderer.device,
                            &renderer.queue,
                            &image::DynamicImage::ImageRgba8(dyn_img),
                        )
                    );
                    ui_renderer.set_font_texture(wgpu_texture.clone());
                    let _ = FONT_TEXTURE.set(wgpu_texture);
                }
            }

            let mut text_buffers = Vec::new();
            match self.current_menu.as_str() {
                "main" => {
                    let items = ["DEATHMATCH", "HOTSEAT", "QUIT"];
                    let item_h = 54.0;
                    let start_y = h * 0.5 - (items.len() as f32 * (item_h + 12.0)) * 0.5;
                    let right_margin = 100.0;

                    for (i, item) in items.iter().enumerate() {
                        let y = start_y + (i as f32) * (item_h + 12.0);
                        let text_y = y + 10.0;
                        
                        let size = if i == self.main_menu_selected { 36.0 } else { 30.0 };
                        let text_color = if i == self.main_menu_selected {
                            [1.0, 0.25, 0.25, 1.0]
                        } else {
                            [0.82, 0.86, 0.90, 1.0]
                        };
                        
                        let mut text_width = 0.0;
                        for ch in item.chars() {
                            let upper = ch.to_ascii_uppercase();
                            if upper == ' ' {
                                const PROPB_SPACE_WIDTH: f32 = 12.0;
                                const PROPB_GAP_WIDTH: f32 = 4.0;
                                const PROPB_HEIGHT: f32 = 36.0;
                                let size_scale = size / PROPB_HEIGHT;
                                text_width += (PROPB_SPACE_WIDTH + PROPB_GAP_WIDTH) * size_scale;
                            } else if ('A'..='Z').contains(&upper) {
                                const PROPB_MAP: [(u16, u16, u16); 26] = [
                                    (11, 12, 33), (49, 12, 31), (85, 12, 31), (120, 12, 30),
                                    (156, 12, 21), (183, 12, 21), (207, 12, 32), (13, 55, 30),
                                    (49, 55, 13), (66, 55, 29), (101, 55, 31), (135, 55, 21),
                                    (158, 55, 40), (204, 55, 32), (12, 97, 31), (48, 97, 31),
                                    (82, 97, 30), (118, 97, 30), (153, 97, 30), (185, 97, 25),
                                    (213, 97, 30), (11, 139, 32), (42, 139, 51), (93, 139, 32),
                                    (126, 139, 31), (158, 139, 25),
                                ];
                                const PROPB_GAP_WIDTH: f32 = 4.0;
                                const PROPB_HEIGHT: f32 = 36.0;
                                let idx = (upper as u8 - b'A') as usize;
                                let (_sx, _sy, w) = PROPB_MAP[idx];
                                let size_scale = size / PROPB_HEIGHT;
                                let aw = (w as f32) * size_scale;
                                text_width += (aw + PROPB_GAP_WIDTH * size_scale).round();
                            } else {
                                text_width += size.round();
                            }
                        }
                        
                        let mut text_x = w - text_width - right_margin;

                        for ch in item.chars() {
                            let (vb, ib, advance) = ui_renderer.create_prop_char_buffers(
                                text_x,
                                text_y,
                                size,
                                w,
                                h,
                                ch,
                                text_color,
                            );
                            text_buffers.push(((vb, ib), ch as u8));
                            text_x += advance;
                        }
                    }
                }
                "map_select" | "1v1_map_select" => {
                    let title_text = "SELECT MAP";
                    let title_size = 40.0;
                    let title_y = 80.0;
                    let title_color = [1.0, 0.69, 0.0, 1.0];
                    
                    let mut title_width = 0.0;
                    for ch in title_text.chars() {
                        let upper = ch.to_ascii_uppercase();
                        if upper == ' ' {
                            const PROPB_SPACE_WIDTH: f32 = 12.0;
                            const PROPB_GAP_WIDTH: f32 = 4.0;
                            const PROPB_HEIGHT: f32 = 36.0;
                            let size_scale = title_size / PROPB_HEIGHT;
                            title_width += (PROPB_SPACE_WIDTH + PROPB_GAP_WIDTH) * size_scale;
                        } else if ('A'..='Z').contains(&upper) {
                            const PROPB_MAP: [(u16, u16, u16); 26] = [
                                (11, 12, 33), (49, 12, 31), (85, 12, 31), (120, 12, 30),
                                (156, 12, 21), (183, 12, 21), (207, 12, 32), (13, 55, 30),
                                (49, 55, 13), (66, 55, 29), (101, 55, 31), (135, 55, 21),
                                (158, 55, 40), (204, 55, 32), (12, 97, 31), (48, 97, 31),
                                (82, 97, 30), (118, 97, 30), (153, 97, 30), (185, 97, 25),
                                (213, 97, 30), (11, 139, 32), (42, 139, 51), (93, 139, 32),
                                (126, 139, 31), (158, 139, 25),
                            ];
                            const PROPB_GAP_WIDTH: f32 = 4.0;
                            const PROPB_HEIGHT: f32 = 36.0;
                            let idx = (upper as u8 - b'A') as usize;
                            let (_sx, _sy, w) = PROPB_MAP[idx];
                            let size_scale = title_size / PROPB_HEIGHT;
                            let aw = (w as f32) * size_scale;
                            title_width += (aw + PROPB_GAP_WIDTH * size_scale).round();
                        } else {
                            title_width += title_size.round();
                        }
                    }
                    
                    let mut title_x = w * 0.5 - title_width * 0.5;
                    for ch in title_text.chars() {
                        let (vb, ib, advance) = ui_renderer.create_prop_char_buffers(
                            title_x,
                            title_y,
                            title_size,
                            w,
                            h,
                            ch,
                            title_color,
                        );
                        text_buffers.push(((vb, ib), ch as u8));
                        title_x += advance;
                    }
                    
                    if self.available_maps.is_empty() {
                        let no_maps_text = "NO MAPS FOUND";
                        let no_maps_size = 32.0;
                        let no_maps_y = h * 0.5;
                        let no_maps_color = [1.0, 0.39, 0.39, 1.0];
                        
                        let mut no_maps_width = 0.0;
                        for ch in no_maps_text.chars() {
                            let upper = ch.to_ascii_uppercase();
                            if upper == ' ' {
                                const PROPB_SPACE_WIDTH: f32 = 12.0;
                                const PROPB_GAP_WIDTH: f32 = 4.0;
                                const PROPB_HEIGHT: f32 = 36.0;
                                let size_scale = no_maps_size / PROPB_HEIGHT;
                                no_maps_width += (PROPB_SPACE_WIDTH + PROPB_GAP_WIDTH) * size_scale;
                            } else if ('A'..='Z').contains(&upper) {
                                const PROPB_MAP: [(u16, u16, u16); 26] = [
                                    (11, 12, 33), (49, 12, 31), (85, 12, 31), (120, 12, 30),
                                    (156, 12, 21), (183, 12, 21), (207, 12, 32), (13, 55, 30),
                                    (49, 55, 13), (66, 55, 29), (101, 55, 31), (135, 55, 21),
                                    (158, 55, 40), (204, 55, 32), (12, 97, 31), (48, 97, 31),
                                    (82, 97, 30), (118, 97, 30), (153, 97, 30), (185, 97, 25),
                                    (213, 97, 30), (11, 139, 32), (42, 139, 51), (93, 139, 32),
                                    (126, 139, 31), (158, 139, 25),
                                ];
                                const PROPB_GAP_WIDTH: f32 = 4.0;
                                const PROPB_HEIGHT: f32 = 36.0;
                                let idx = (upper as u8 - b'A') as usize;
                                let (_sx, _sy, w) = PROPB_MAP[idx];
                                let size_scale = no_maps_size / PROPB_HEIGHT;
                                let aw = (w as f32) * size_scale;
                                no_maps_width += (aw + PROPB_GAP_WIDTH * size_scale).round();
                            } else {
                                no_maps_width += no_maps_size.round();
                            }
                        }
                        
                        let mut no_maps_x = w * 0.5 - no_maps_width * 0.5;
                        for ch in no_maps_text.chars() {
                            let (vb, ib, advance) = ui_renderer.create_prop_char_buffers(
                                no_maps_x,
                                no_maps_y,
                                no_maps_size,
                                w,
                                h,
                                ch,
                                no_maps_color,
                            );
                            text_buffers.push(((vb, ib), ch as u8));
                            no_maps_x += advance;
                        }
                    } else {
                        let item_h = 54.0;
                        let start_y = h * 0.5 - (self.available_maps.len() as f32 * (item_h + 12.0)) * 0.5;
                        let start_x = w * 0.5 - 200.0;

                        for (i, map) in self.available_maps.iter().enumerate() {
                            let y = start_y + (i as f32) * (item_h + 12.0);
                            let text_y = y + 10.0;
                            
                            let size = if i == self.map_menu_selected { 36.0 } else { 30.0 };
                            let text_color = if i == self.map_menu_selected {
                                [1.0, 0.25, 0.25, 1.0]
                            } else {
                                [0.82, 0.86, 0.90, 1.0]
                            };
                            
                            let mut text_x = start_x + 18.0;
                            for ch in map.chars() {
                                let (vb, ib, advance) = ui_renderer.create_prop_char_buffers(
                                    text_x,
                                    text_y,
                                    size,
                                    w,
                                    h,
                                    ch,
                                    text_color,
                                );
                                text_buffers.push(((vb, ib), ch as u8));
                                text_x += advance;
                            }
                        }
                    }
                }
                _ => {}
            }

            let text_bind_group = if !text_buffers.is_empty() {
                if let Some(ref font_texture) = ui_renderer.font_texture {
                    if let Some(ref bind_group_layout) = ui_renderer.text_bind_group_layout {
                        Some(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("UI Text Bind Group"),
                            layout: bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&font_texture.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&font_texture.sampler),
                                },
                            ],
                        }))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Menu Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.07,
                                g: 0.09,
                                b: 0.11,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                if let Some(ref text_pipeline) = ui_renderer.text_pipeline {
                    if let Some(ref bind_group) = text_bind_group {
                        render_pass.set_pipeline(text_pipeline);
                        render_pass.set_bind_group(0, bind_group, &[]);

                        for ((vb, ib), _ch) in text_buffers.iter() {
                            if vb.size() > 0 && ib.size() > 0 {
                                render_pass.set_vertex_buffer(0, vb.slice(..));
                                render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                                render_pass.draw_indexed(0..6, 0, 0..1);
                            }
                        }
                    }
                }
            }

            renderer.queue.submit(std::iter::once(encoder.finish()));
            renderer.end_frame(frame);
        }
    }
}

fn list_available_maps() -> Vec<String> {
    let maps_dir = std::path::Path::new("maps");
    let mut maps = Vec::new();

    if maps_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(maps_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json")
                                && !name.ends_with("_navgraph.json")
                                && !name.ends_with("_defrag.json")
                            {
                                let map_name = name.trim_end_matches(".json").to_string();
                                maps.push(map_name);
                            }
                        }
                    }
                }
            }
        }
    }
    
    if maps.is_empty() {
        maps.push("soldat".to_string());
        maps.push("q3dm6".to_string());
    }
    
    maps.sort();
    println!("[MENU] Total maps loaded: {} - {:?}", maps.len(), maps);
    maps
}

fn list_available_models() -> Vec<String> {
    let mut models = Vec::new();
    let models_dir = std::path::Path::new("models/players");
    
    if models_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(models_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            models.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    if models.is_empty() {
        models.push("sarge".to_string());
        models.push("visor".to_string());
        models.push("grunt".to_string());
    }
    
    models.sort();
    models
}
