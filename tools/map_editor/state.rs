use macroquad::prelude::*;
use std::fs;

use super::tools::*;
use super::textures::{self, *};
use super::icons::ItemIcons;
use super::shaders::*;
use super::map;
use super::map_loader;
use super::deferred_renderer;
use super::procedural_tiles;
use super::nav_graph;
use super::nav_graph_generator;

#[path = "../../src/game/tile_shader.rs"]
mod tile_shader;

#[path = "../../src/game/tile_shader_materials.rs"]
mod tile_shader_materials;

#[path = "../../src/game/q3_shader_parser.rs"]
mod q3_shader_parser;

mod rgb_shader;

pub const TILE_WIDTH: f32 = 32.0;
pub const TILE_HEIGHT: f32 = 16.0;

pub struct EditorState {
    map: map::Map,
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
    current_tool: EditorTool,
    current_texture: u16,
    current_item_type: ItemPlaceType,
    map_name: String,
    show_grid: bool,
    show_help: bool,
    tile_textures: std::collections::HashMap<u16, Texture2D>,
    wall_textures: Vec<WallTexture>,
    background_texture: Option<Texture2D>,
    background_textures: Vec<BackgroundTexture>,
    current_bg_texture: usize,
    current_bg_additive: bool,
    current_bg_alpha: f32,
    current_bg_scale: f32,
    bg_snap_to_grid: bool,
    texture_picker_scroll: f32,
    texture_scale: f32,
    texture_offset_x: f32,
    texture_offset_y: f32,
    brush_size: i32,
    line_draw_start: Option<(i32, i32)>,
    last_bg_pos: Option<(f32, f32)>,
    current_shader: Option<String>,
    available_shaders: Vec<String>,
    show_shader_picker: bool,
    shader_renderer: tile_shader::TileShaderRenderer,
    item_icons: ItemIcons,
    deferred_renderer: Option<deferred_renderer::DeferredRenderer>,
    ambient_light: f32,
    needs_lightmap_rebuild: bool,
    lightmap_rebuild_timer: f32,
    selected_teleporter_index: Option<usize>,
    selected_object: Option<SelectedObject>,
    dragging_object: bool,
    show_properties: bool,
    property_hover: Option<String>,
    nav_graph: Option<nav_graph::NavGraph>,
    show_nav_graph: bool,
    show_nav_edges: bool,
    show_texture_picker: bool,
    show_bg_texture_picker: bool,
    input_handler: super::input::InputHandler,
    help_panel: super::help::HelpPanel,
}

impl EditorState {
    pub fn new(map_name: &str) -> Self {
        let map = if let Ok(loaded_map) = map::Map::load_from_file(map_name) {
            loaded_map
        } else {
            let mut tiles = vec![vec![map::Tile { 
                solid: false, 
                texture_id: 0,
                shader_name: None,
                detail_texture: None,
                glow_texture: None,
                blend_alpha: 1.0,
            }; 50]; 60];
            for x in 0..60 {
                tiles[x][48].solid = true;
                tiles[x][48].texture_id = 1;
                tiles[x][49].solid = true;
                tiles[x][49].texture_id = 1;
            }
            for y in 0..50 {
                tiles[0][y].solid = true;
                tiles[0][y].texture_id = 1;
                tiles[59][y].solid = true;
                tiles[59][y].texture_id = 1;
            }
            
            map::Map {
                width: 60,
                height: 50,
                tiles,
                spawn_points: vec![
                    map::SpawnPoint { x: 960.0, y: 760.0, team: 0 },
                ],
                items: vec![],
                jumppads: vec![],
                teleporters: vec![],
                lights: vec![],
                background_elements: vec![],
            }
        };
        
        EditorState {
            map,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
            current_tool: EditorTool::Draw,
            current_texture: 1,
            current_item_type: ItemPlaceType::RocketLauncher,
            map_name: map_name.to_string(),
            show_grid: true,
            show_help: false,
            tile_textures: std::collections::HashMap::new(),
            wall_textures: Vec::new(),
            background_texture: None,
            background_textures: Vec::new(),
            current_bg_texture: 0,
            current_bg_additive: false,
            current_bg_alpha: 0.5,
            current_bg_scale: 1.0,
            bg_snap_to_grid: true,
            texture_picker_scroll: 0.0,
            texture_scale: 1.0,
            texture_offset_x: 0.0,
            texture_offset_y: 0.0,
            brush_size: 1,
            line_draw_start: None,
            last_bg_pos: None,
            current_shader: None,
            available_shaders: vec![
                "border11light".to_string(),
                "dooreye_purple".to_string(),
                "metal_glow".to_string(),
                "tech_panel".to_string(),
                "concrete_detail".to_string(),
                "metal_trim".to_string(),
            ],
            show_shader_picker: false,
            shader_renderer: tile_shader::TileShaderRenderer::new(),
            item_icons: ItemIcons::new(),
            deferred_renderer: None,
            ambient_light: 0.06,
            needs_lightmap_rebuild: true,
            lightmap_rebuild_timer: 0.0,
            selected_teleporter_index: None,
            selected_object: None,
            dragging_object: false,
            show_properties: true,
            property_hover: None,
            nav_graph: None,
            show_nav_graph: false,
            show_nav_edges: true,
            show_texture_picker: false,
            show_bg_texture_picker: false,
            input_handler: super::input::InputHandler::new(),
            help_panel: super::help::HelpPanel::new(),
        }
    }
    
    fn mark_lightmap_dirty(&mut self) {
        self.needs_lightmap_rebuild = true;
        self.lightmap_rebuild_timer = 1.0;
    }
    
    fn update_lightmap_rebuild(&mut self, delta_time: f32) {
        if self.needs_lightmap_rebuild {
            self.lightmap_rebuild_timer -= delta_time;
            if self.lightmap_rebuild_timer <= 0.0 {
        if let Some(renderer) = &mut self.deferred_renderer {
            renderer.mark_static_lights_dirty();
                }
                self.needs_lightmap_rebuild = false;
                self.lightmap_rebuild_timer = 0.0;
            }
        }
    }
    
    fn generate_nav_graph(&mut self) {
        println!("Generating navigation graph...");
        let generator = nav_graph_generator::NavGraphGenerator::new(self.map.clone());
        let graph = generator.generate();
        println!("Generated {} nodes and {} edges", graph.nodes.len(), graph.edges.len());
        self.nav_graph = Some(graph);
    }
    
    fn save_nav_graph(&self) {
        if let Some(ref graph) = self.nav_graph {
            let output_path = format!("maps/{}_navgraph.json", self.map_name);
            let json = serde_json::to_string_pretty(&graph).unwrap();
            fs::write(&output_path, json).expect("Failed to write navigation graph");
            println!("Saved navigation graph to: {}", output_path);
        }
    }
    
    fn load_nav_graph(&mut self) {
        let input_path = format!("maps/{}_navgraph.json", self.map_name);
        if let Ok(json) = fs::read_to_string(&input_path) {
            if let Ok(graph) = serde_json::from_str(&json) {
                self.nav_graph = Some(graph);
                println!("Loaded navigation graph from: {}", input_path);
            }
        }
    }
    
    pub async fn load_textures(&mut self) {
        println!("[Editor] Scanning for all available textures...");
        let wall_texture_paths = textures::scan_all_textures().await;
        println!("[Editor] Found {} textures", wall_texture_paths.len());
        
        let mut loaded_count = 0;
        let mut failed_count = 0;
        
        for (idx, path) in wall_texture_paths.iter().enumerate() {
            if loaded_count >= 2500 {
                println!("[Editor] Reached texture limit (2500)");
                break;
            }
            
            match load_image(path).await {
                Ok(image) => {
                    let texture = Texture2D::from_image(&image);
                    texture.set_filter(FilterMode::Linear);
                    let name = path.split('/').last().unwrap_or("unknown")
                        .replace(".png", "")
                        .replace(".jpg", "")
                        .replace(".tga", "");
                    self.wall_textures.push(WallTexture {
                        texture: texture.clone(),
                        name,
                        path: path.clone(),
                    });
                    self.tile_textures.insert(loaded_count as u16, texture);
                    loaded_count += 1;
                    
                    if loaded_count % 100 == 0 {
                        println!("[Editor] Loaded {}/{} textures...", loaded_count, wall_texture_paths.len());
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    if failed_count <= 5 {
                        println!("[Editor] ✗ Failed to load texture {}: {} ({})", idx, path, e);
                    }
                }
            }
        }
        
        println!("[Editor] ✓ Loaded {} textures, {} failed", loaded_count, failed_count);
        
        println!("[Editor] Adding all textures to background list...");
        for wall_tex in &self.wall_textures {
            self.background_textures.push(BackgroundTexture {
                frames: vec![wall_tex.texture.clone()],
                name: wall_tex.name.clone(),
                base_path: wall_tex.path.clone(),
                is_animated: false,
                has_wave_effect: false,
            });
        }
        println!("[Editor] ✓ Added {} textures to background list", self.wall_textures.len());
        
        if let Ok(image) = load_image("q3-resources/textures/sfx/b_x_condark.png").await {
            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Linear);
            self.background_texture = Some(texture);
            println!("[Editor] ✓ Loaded default background texture");
        }
        
        println!("[Editor] Loading animated shaders...");
        let mut shader_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir("q3-resources/scripts") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("shader") {
                    if let Some(path_str) = path.to_str() {
                        shader_files.push(path_str.to_string());
                    }
                }
            }
        }
        shader_files.sort();
        
        let mut loaded_shaders = std::collections::HashSet::new();
        let mut animated_count = 0;
        
        for shader_file in shader_files {
            if let Ok(_) = std::fs::metadata(&shader_file) {
                let shaders = parse_shader_file(&shader_file);
                
                for (shader_name, tex_paths, _is_additive, has_wave) in shaders {
                    if loaded_shaders.contains(&shader_name) {
                        continue;
                    }
                    
                    let mut frames = Vec::new();
                    
                    for tex_path in &tex_paths {
                        let full_path = if tex_path.starts_with("textures/") {
                            format!("q3-resources/{}", tex_path)
                        } else {
                            tex_path.clone()
                        };
                        
                        if let Ok(image) = load_image(&full_path).await {
                            let texture = Texture2D::from_image(&image);
                            texture.set_filter(FilterMode::Linear);
                            frames.push(texture);
                        }
                    }
                    
                    if !frames.is_empty() && (frames.len() > 1 || has_wave) {
                        let name = format!("[Anim] {}", shader_name.split('/').last().unwrap_or(&shader_name));
                        let is_animated = frames.len() > 1;
                        
                        self.background_textures.push(BackgroundTexture {
                            frames,
                            name,
                            base_path: shader_name.clone(),
                            is_animated,
                            has_wave_effect: has_wave,
                        });
                        
                        loaded_shaders.insert(shader_name);
                        animated_count += 1;
                    }
                }
            }
        }
        println!("[Editor] ✓ Loaded {} animated shaders", animated_count);
        
        println!("[Editor] Loading tile shaders...");
        if let Ok(content) = std::fs::read_to_string("tile_shaders.json") {
            if let Ok(shaders) = serde_json::from_str::<Vec<tile_shader::TileShader>>(&content) {
                println!("[Editor] Loading {} custom shaders from JSON", shaders.len());
                
                self.available_shaders.clear();
                
                for shader in shaders {
                    if !shader.base_texture.is_empty() {
                        self.shader_renderer.load_texture(&shader.base_texture).await;
                    }
                    
                    for stage in &shader.stages {
                        if !stage.texture_path.is_empty() {
                            self.shader_renderer.load_texture(&stage.texture_path).await;
                        }
                    }
                    
                    self.available_shaders.push(shader.name.clone());
                    self.shader_renderer.add_shader(shader);
                }
                
                println!("[Editor] ✓ Loaded {} tile shaders", self.available_shaders.len());
            }
        } else {
            println!("[Editor] No tile_shaders.json found - using defaults");
        }
        
        println!("[Editor] Loading item icons...");
        self.item_icons.load_all().await;
        println!("[Editor] Item icons loaded!");
    }
    
    fn find_floor_y(&self, x: f32, y: f32) -> f32 {
        let tile_x = (x / TILE_WIDTH) as usize;
        let mut search_y = (y / TILE_HEIGHT) as usize;
        
        while search_y < self.map.height && !self.map.tiles.get(tile_x).and_then(|col| col.get(search_y)).map_or(false, |t| t.solid) {
            search_y += 1;
        }
        
        if search_y < self.map.height {
            (search_y as f32 * TILE_HEIGHT) - 20.0
        } else {
            y
        }
    }
    
    fn save(&mut self) {
        fs::create_dir_all("maps").ok();
        let map_file = map_loader::MapFile::from_map(&self.map, TILE_WIDTH, TILE_HEIGHT, &self.map_name);
        match map_file.save_to_file(&format!("maps/{}.json", self.map_name)) {
            Ok(_) => {
                println!("Map saved: maps/{}.json", self.map_name);
                
                println!("Auto-generating navigation graph...");
                self.generate_nav_graph();
                self.save_nav_graph();
            },
            Err(e) => println!("Error saving map: {}", e),
        }
    }
    
    pub fn handle_input(&mut self) {
        let is_shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        
        if let Some(obj) = self.selected_object.clone() {
            let move_step = 4.0;
            let size_step = 4.0;
            
            if is_key_pressed(KeyCode::Left) {
                if is_shift {
                    if let SelectedObject::Teleporter(idx) = &obj {
                        if *idx < self.map.teleporters.len() {
                            self.map.teleporters[*idx].width = (self.map.teleporters[*idx].width - size_step).max(16.0);
                        }
                    }
                } else {
                    match &obj {
                        SelectedObject::SpawnPoint(idx) => {
                            if *idx < self.map.spawn_points.len() {
                                self.map.spawn_points[*idx].x -= move_step;
                            }
                        },
                        SelectedObject::Item(idx) => {
                            if *idx < self.map.items.len() {
                                self.map.items[*idx].x -= move_step;
                            }
                        },
                        SelectedObject::JumpPad(idx) => {
                            if *idx < self.map.jumppads.len() {
                                self.map.jumppads[*idx].x -= move_step;
                            }
                        },
                        SelectedObject::Teleporter(idx) => {
                            if *idx < self.map.teleporters.len() {
                                self.map.teleporters[*idx].x -= move_step;
                            }
                        },
                        SelectedObject::Light(idx) => {
                            if *idx < self.map.lights.len() {
                                self.map.lights[*idx].x -= move_step;
                                self.mark_lightmap_dirty();
                            }
                        },
                        SelectedObject::BackgroundElement(idx) => {
                            if *idx < self.map.background_elements.len() {
                                self.map.background_elements[*idx].x -= move_step;
                            }
                        },
                    }
                }
            }
            
            if is_key_pressed(KeyCode::Right) {
                if is_shift {
                    if let SelectedObject::Teleporter(idx) = &obj {
                        if *idx < self.map.teleporters.len() {
                            self.map.teleporters[*idx].width = (self.map.teleporters[*idx].width + size_step).min(256.0);
                        }
                    }
                } else {
                    match &obj {
                        SelectedObject::SpawnPoint(idx) => {
                            if *idx < self.map.spawn_points.len() {
                                self.map.spawn_points[*idx].x += move_step;
                            }
                        },
                        SelectedObject::Item(idx) => {
                            if *idx < self.map.items.len() {
                                self.map.items[*idx].x += move_step;
                            }
                        },
                        SelectedObject::JumpPad(idx) => {
                            if *idx < self.map.jumppads.len() {
                                self.map.jumppads[*idx].x += move_step;
                            }
                        },
                        SelectedObject::Teleporter(idx) => {
                            if *idx < self.map.teleporters.len() {
                                self.map.teleporters[*idx].x += move_step;
                            }
                        },
                        SelectedObject::Light(idx) => {
                            if *idx < self.map.lights.len() {
                                self.map.lights[*idx].x += move_step;
                                self.mark_lightmap_dirty();
                            }
                        },
                        SelectedObject::BackgroundElement(idx) => {
                            if *idx < self.map.background_elements.len() {
                                self.map.background_elements[*idx].x += move_step;
                            }
                        },
                    }
                }
            }
            
            if is_key_pressed(KeyCode::Up) {
                if is_shift {
                    if let SelectedObject::Teleporter(idx) = &obj {
                        if *idx < self.map.teleporters.len() {
                            self.map.teleporters[*idx].height = (self.map.teleporters[*idx].height - size_step).max(16.0);
                        }
                    }
                } else {
                    match &obj {
                        SelectedObject::SpawnPoint(idx) => {
                            if *idx < self.map.spawn_points.len() {
                                self.map.spawn_points[*idx].y -= move_step;
                            }
                        },
                        SelectedObject::Item(idx) => {
                            if *idx < self.map.items.len() {
                                self.map.items[*idx].y -= move_step;
                            }
                        },
                        SelectedObject::JumpPad(idx) => {
                            if *idx < self.map.jumppads.len() {
                                self.map.jumppads[*idx].y -= move_step;
                            }
                        },
                        SelectedObject::Teleporter(idx) => {
                            if *idx < self.map.teleporters.len() {
                                self.map.teleporters[*idx].y -= move_step;
                            }
                        },
                        SelectedObject::Light(idx) => {
                            if *idx < self.map.lights.len() {
                                self.map.lights[*idx].y -= move_step;
                                self.mark_lightmap_dirty();
                            }
                        },
                        SelectedObject::BackgroundElement(idx) => {
                            if *idx < self.map.background_elements.len() {
                                self.map.background_elements[*idx].y -= move_step;
                            }
                        },
                    }
                }
            }
            
            if is_key_pressed(KeyCode::Down) {
                if is_shift {
                    if let SelectedObject::Teleporter(idx) = &obj {
                        if *idx < self.map.teleporters.len() {
                            self.map.teleporters[*idx].height = (self.map.teleporters[*idx].height + size_step).min(256.0);
                        }
                    }
                } else {
                    match &obj {
                        SelectedObject::SpawnPoint(idx) => {
                            if *idx < self.map.spawn_points.len() {
                                self.map.spawn_points[*idx].y += move_step;
                            }
                        },
                        SelectedObject::Item(idx) => {
                            if *idx < self.map.items.len() {
                                self.map.items[*idx].y += move_step;
                            }
                        },
                        SelectedObject::JumpPad(idx) => {
                            if *idx < self.map.jumppads.len() {
                                self.map.jumppads[*idx].y += move_step;
                            }
                        },
                        SelectedObject::Teleporter(idx) => {
                            if *idx < self.map.teleporters.len() {
                                self.map.teleporters[*idx].y += move_step;
                            }
                        },
                        SelectedObject::Light(idx) => {
                            if *idx < self.map.lights.len() {
                                self.map.lights[*idx].y += move_step;
                                self.mark_lightmap_dirty();
                            }
                        },
                        SelectedObject::BackgroundElement(idx) => {
                            if *idx < self.map.background_elements.len() {
                                self.map.background_elements[*idx].y += move_step;
                            }
                        },
                    }
                }
            }
        }
        
        let camera_speed = 10.0;
        if is_key_down(KeyCode::A) {
            self.camera_x -= camera_speed;
        }
        if is_key_down(KeyCode::D) {
            self.camera_x += camera_speed;
        }
        if is_key_down(KeyCode::W) {
            self.camera_y -= camera_speed;
        }
        if is_key_down(KeyCode::S) {
            self.camera_y += camera_speed;
        }
        
        if is_key_pressed(KeyCode::Key0) { self.current_tool = EditorTool::Select; self.selected_object = None; }
        if is_key_pressed(KeyCode::Key1) { 
            self.current_tool = EditorTool::Draw; 
            self.selected_object = None; 
            self.selected_teleporter_index = None; 
            self.line_draw_start = None;
        }
        if is_key_pressed(KeyCode::Key2) { 
            self.current_tool = EditorTool::Erase; 
            self.selected_object = None; 
            self.selected_teleporter_index = None; 
            self.line_draw_start = None;
        }
        if is_key_pressed(KeyCode::Key3) { self.current_tool = EditorTool::SpawnPoint; self.selected_object = None; self.selected_teleporter_index = None; }
        if is_key_pressed(KeyCode::Key4) { self.current_tool = EditorTool::Item; self.selected_object = None; self.selected_teleporter_index = None; }
        if is_key_pressed(KeyCode::Key5) { self.current_tool = EditorTool::JumpPad; self.selected_object = None; self.selected_teleporter_index = None; }
        if is_key_pressed(KeyCode::Key6) { 
            self.current_tool = EditorTool::Teleporter;
            self.selected_object = None;
            self.selected_teleporter_index = None;
        }
        if is_key_pressed(KeyCode::Key7) { self.current_tool = EditorTool::Light; self.selected_object = None; self.selected_teleporter_index = None; }
        if is_key_pressed(KeyCode::Key8) { self.current_tool = EditorTool::Background; self.selected_object = None; self.selected_teleporter_index = None; }
        
        if is_key_pressed(KeyCode::P) {
            self.show_properties = !self.show_properties;
        }
        
        if is_key_pressed(KeyCode::Q) {
            if self.current_tool == EditorTool::Draw || self.current_tool == EditorTool::Erase {
                self.brush_size = match self.brush_size {
                    1 => 2,
                    2 => 4,
                    _ => 1,
                };
                println!("Brush size: {}x{}", self.brush_size, self.brush_size);
            }
        }
        
        if is_key_pressed(KeyCode::X) {
            if self.current_tool == EditorTool::Draw {
                if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                    self.show_shader_picker = !self.show_shader_picker;
                } else {
                    if self.current_shader.is_some() {
                        self.current_shader = None;
                        println!("Shader disabled - using normal texture");
                    } else if !self.available_shaders.is_empty() {
                        self.current_shader = Some(self.available_shaders[0].clone());
                        println!("Shader: {}", self.available_shaders[0]);
                    }
                }
            }
        }
        
        if is_key_pressed(KeyCode::R) {
            if let Some(SelectedObject::Teleporter(idx)) = &self.selected_object {
                self.selected_teleporter_index = Some(*idx);
                self.current_tool = EditorTool::TeleporterDestination;
            }
        }
        
        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            if let Some(obj) = &self.selected_object {
                match obj {
                    SelectedObject::SpawnPoint(idx) => {
                        if *idx < self.map.spawn_points.len() {
                            self.map.spawn_points.remove(*idx);
                        }
                    },
                    SelectedObject::Item(idx) => {
                        if *idx < self.map.items.len() {
                            self.map.items.remove(*idx);
                        }
                    },
                    SelectedObject::JumpPad(idx) => {
                        if *idx < self.map.jumppads.len() {
                            self.map.jumppads.remove(*idx);
                        }
                    },
                    SelectedObject::Teleporter(idx) => {
                        if *idx < self.map.teleporters.len() {
                            self.map.teleporters.remove(*idx);
                        }
                    },
                    SelectedObject::Light(idx) => {
                        if *idx < self.map.lights.len() {
                            self.map.lights.remove(*idx);
                            self.mark_lightmap_dirty();
                        }
                    },
                    SelectedObject::BackgroundElement(idx) => {
                        if *idx < self.map.background_elements.len() {
                            self.map.background_elements.remove(*idx);
                        }
                    },
                }
                self.selected_object = None;
            }
        }
        
        let context = super::input::get_current_context(self.current_tool, &self.selected_object);
        
        self.input_handler.handle_texture_scale(&mut self.texture_scale, context);
        self.input_handler.handle_background_controls(&mut self.current_bg_alpha, &mut self.current_bg_scale, context);
        
        if is_key_pressed(KeyCode::LeftBracket) {
            if let Some(SelectedObject::Light(idx)) = &self.selected_object {
                if *idx < self.map.lights.len() {
                    self.map.lights[*idx].radius = (self.map.lights[*idx].radius - 20.0).max(50.0);
                    self.mark_lightmap_dirty();
                }
            } else if let Some(SelectedObject::JumpPad(idx)) = &self.selected_object {
                if *idx < self.map.jumppads.len() {
                    self.map.jumppads[*idx].force_x -= 0.2;
                }
            }
        }
        
        if is_key_pressed(KeyCode::RightBracket) {
            if let Some(SelectedObject::Light(idx)) = &self.selected_object {
                if *idx < self.map.lights.len() {
                    self.map.lights[*idx].radius = (self.map.lights[*idx].radius + 20.0).min(2000.0);
                    self.mark_lightmap_dirty();
                }
            } else if let Some(SelectedObject::JumpPad(idx)) = &self.selected_object {
                if *idx < self.map.jumppads.len() {
                    self.map.jumppads[*idx].force_x += 0.2;
                }
            }
        }
        
        if is_key_pressed(KeyCode::T) {
            if self.current_tool == EditorTool::Draw {
                if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                    self.show_texture_picker = !self.show_texture_picker;
                } else {
                    self.current_texture = (self.current_texture + 1) % self.wall_textures.len() as u16;
                }
            } else if self.current_tool == EditorTool::Light {
                let world_x_t = (mouse_position().0 / self.zoom) + self.camera_x;
                let world_y_t = (mouse_position().1 / self.zoom) + self.camera_y;
                let mut changed = false;
                for light in &mut self.map.lights {
                    let dx = light.x - world_x_t;
                    let dy = light.y - world_y_t;
                    if dx * dx + dy * dy < 400.0 {
                        let radii = [100.0, 150.0, 200.0, 300.0, 400.0, 500.0];
                        let current_idx = radii.iter().position(|&r| (r - light.radius).abs() < 10.0).unwrap_or(2);
                        let next_idx = (current_idx + 1) % radii.len();
                        light.radius = radii[next_idx];
                        changed = true;
                    }
                }
                if changed {
                    self.mark_lightmap_dirty();
                }
            }
        }
        
        if is_key_pressed(KeyCode::I) && self.current_tool == EditorTool::Item {
            self.current_item_type = self.current_item_type.next();
        }
        
        if self.current_tool == EditorTool::Background {
            if is_key_pressed(KeyCode::B) {
                if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                    self.show_bg_texture_picker = !self.show_bg_texture_picker;
                } else if !self.background_textures.is_empty() {
                    self.current_bg_texture = (self.current_bg_texture + 1) % self.background_textures.len();
                }
            }
            if is_key_pressed(KeyCode::A) {
                self.current_bg_additive = !self.current_bg_additive;
            }
            if is_key_pressed(KeyCode::V) {
                self.bg_snap_to_grid = !self.bg_snap_to_grid;
            }
            if is_key_pressed(KeyCode::Equal) {
                self.current_bg_scale = (self.current_bg_scale + 0.1).min(10.0);
            }
            if is_key_pressed(KeyCode::Minus) {
                self.current_bg_scale = (self.current_bg_scale - 0.1).max(0.1);
            }
        }
        
        if is_key_pressed(KeyCode::G) {
            self.show_grid = !self.show_grid;
        }
        
        if is_key_pressed(KeyCode::N) {
            if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                self.generate_nav_graph();
            } else {
                self.show_nav_graph = !self.show_nav_graph;
            }
        }
        
        if is_key_pressed(KeyCode::E) && self.show_nav_graph {
            self.show_nav_edges = !self.show_nav_edges;
        }
        
        if is_key_pressed(KeyCode::L) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            self.load_nav_graph();
        }
        
        if is_key_pressed(KeyCode::S) && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)) {
            self.save_nav_graph();
        }
        
        if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::F1) {
            self.show_help = !self.show_help;
        }
        
        if is_key_pressed(KeyCode::S) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            self.save();
        }
        
        if is_key_pressed(KeyCode::Escape) {
            if self.show_texture_picker {
                self.show_texture_picker = false;
            } else if self.show_bg_texture_picker {
                self.show_bg_texture_picker = false;
            } else if self.show_shader_picker {
                self.show_shader_picker = false;
            } else {
                self.current_tool = EditorTool::Draw;
            }
        }

        self.input_handler.handle_zoom(&mut self.zoom);
        
        let (mouse_x, mouse_y) = mouse_position();
        let world_x_temp = (mouse_x / self.zoom) + self.camera_x;
        let world_y_temp = (mouse_y / self.zoom) + self.camera_y;
        
        let context = super::input::get_current_context(self.current_tool, &self.selected_object);
        if self.input_handler.handle_light_radius_scroll(&mut self.map.lights, world_x_temp, world_y_temp, context) {
            self.mark_lightmap_dirty();
        }
        
        let is_cmd = is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper);
        
        if is_cmd && self.current_tool == EditorTool::Draw && self.selected_object.is_none() {
            let offset_step = if is_shift { 0.1 } else { 1.0 };
            
            if let Some(texture) = self.tile_textures.get(&self.current_texture) {
                let tex_w = texture.width() * self.texture_scale;
                let tex_h = texture.height() * self.texture_scale;
                
                if is_key_pressed(KeyCode::Left) {
                    self.texture_offset_x -= offset_step;
                    while self.texture_offset_x < 0.0 {
                        self.texture_offset_x += tex_w;
                    }
                    self.texture_offset_x = self.texture_offset_x % tex_w;
                    println!("Texture offset: ({:.1}, {:.1})", self.texture_offset_x, self.texture_offset_y);
                }
                if is_key_pressed(KeyCode::Right) {
                    self.texture_offset_x += offset_step;
                    self.texture_offset_x = self.texture_offset_x % tex_w;
                    println!("Texture offset: ({:.1}, {:.1})", self.texture_offset_x, self.texture_offset_y);
                }
                if is_key_pressed(KeyCode::Up) {
                    self.texture_offset_y -= offset_step;
                    while self.texture_offset_y < 0.0 {
                        self.texture_offset_y += tex_h;
                    }
                    self.texture_offset_y = self.texture_offset_y % tex_h;
                    println!("Texture offset: ({:.1}, {:.1})", self.texture_offset_x, self.texture_offset_y);
                }
                if is_key_pressed(KeyCode::Down) {
                    self.texture_offset_y += offset_step;
                    self.texture_offset_y = self.texture_offset_y % tex_h;
                    println!("Texture offset: ({:.1}, {:.1})", self.texture_offset_x, self.texture_offset_y);
                }
                if is_key_pressed(KeyCode::Key0) {
                    self.texture_offset_x = 0.0;
                    self.texture_offset_y = 0.0;
                    println!("Texture offset reset to (0, 0)");
                }
            }
        }
        
        if is_key_pressed(KeyCode::F) && self.current_tool == EditorTool::Light {
            for light in &mut self.map.lights {
                let dx = light.x - world_x_temp;
                let dy = light.y - world_y_temp;
                if dx * dx + dy * dy < 400.0 {
                    light.flicker = !light.flicker;
                }
            }
        }
        
        if is_key_pressed(KeyCode::Equal) {
            if let Some(obj) = &self.selected_object {
                match obj {
                    SelectedObject::Light(idx) => {
                        if *idx < self.map.lights.len() {
                            self.map.lights[*idx].intensity = (self.map.lights[*idx].intensity + 0.1).min(10.0);
                            self.mark_lightmap_dirty();
                        }
                    },
                    SelectedObject::JumpPad(idx) => {
                        if *idx < self.map.jumppads.len() {
                            self.map.jumppads[*idx].force_y -= 0.5;
                        }
                    },
                    _ => {}
                }
            } else if self.current_tool == EditorTool::Light {
                let mut changed = false;
                for light in &mut self.map.lights {
                    let dx = light.x - world_x_temp;
                    let dy = light.y - world_y_temp;
                    if dx * dx + dy * dy < 400.0 {
                        light.intensity = (light.intensity + 0.2).min(10.0);
                        changed = true;
                    }
                }
                if changed {
                    self.mark_lightmap_dirty();
                }
            } else {
                self.ambient_light = (self.ambient_light + 0.01).min(0.5);
                self.mark_lightmap_dirty();
            }
        }
        
        if is_key_pressed(KeyCode::Minus) {
            if let Some(obj) = &self.selected_object {
                match obj {
                    SelectedObject::Light(idx) => {
                        if *idx < self.map.lights.len() {
                            self.map.lights[*idx].intensity = (self.map.lights[*idx].intensity - 0.1).max(0.0);
                            self.mark_lightmap_dirty();
                        }
                    },
                    SelectedObject::JumpPad(idx) => {
                        if *idx < self.map.jumppads.len() {
                            self.map.jumppads[*idx].force_y += 0.5;
                        }
                    },
                    _ => {}
                }
            } else if self.current_tool == EditorTool::Light {
                let mut changed = false;
                for light in &mut self.map.lights {
                    let dx = light.x - world_x_temp;
                    let dy = light.y - world_y_temp;
                    if dx * dx + dy * dy < 400.0 {
                        light.intensity = (light.intensity - 0.2).max(0.0);
                        changed = true;
                    }
                }
                if changed {
                    self.mark_lightmap_dirty();
                }
            } else {
                self.ambient_light = (self.ambient_light - 0.01).max(0.0);
                self.mark_lightmap_dirty();
            }
        }
        
        if is_key_pressed(KeyCode::LeftBracket) {
            if let Some(SelectedObject::Light(idx)) = &self.selected_object {
                if *idx < self.map.lights.len() {
                    self.map.lights[*idx].radius = (self.map.lights[*idx].radius - 20.0).max(50.0);
                    self.mark_lightmap_dirty();
                }
            } else if let Some(SelectedObject::JumpPad(idx)) = &self.selected_object {
                if *idx < self.map.jumppads.len() {
                    self.map.jumppads[*idx].force_x -= 0.2;
                }
            }
        }
        
        if is_key_pressed(KeyCode::RightBracket) {
            if let Some(SelectedObject::Light(idx)) = &self.selected_object {
                if *idx < self.map.lights.len() {
                    self.map.lights[*idx].radius = (self.map.lights[*idx].radius + 20.0).min(2000.0);
                    self.mark_lightmap_dirty();
                }
            } else if let Some(SelectedObject::JumpPad(idx)) = &self.selected_object {
                if *idx < self.map.jumppads.len() {
                    self.map.jumppads[*idx].force_x += 0.2;
                }
            }
        }
        
        self.handle_mouse_input();
    }
    
    fn draw_background_at(&mut self, world_x: f32, world_y: f32) {
        if !self.background_textures.is_empty() {
            let bg_tex = &self.background_textures[self.current_bg_texture];
            let default_size = 64.0 * self.current_bg_scale;
            let anim_speed = if bg_tex.is_animated { 10.0 } else { 0.0 };
            
            let (final_x, final_y) = if self.bg_snap_to_grid {
                let grid_size = 32.0;
                (
                    (world_x / grid_size).round() * grid_size,
                    (world_y / grid_size).round() * grid_size
                )
            } else {
                (world_x, world_y)
            };
            
            if let Some((last_x, last_y)) = self.last_bg_pos {
                let dx = final_x - last_x;
                let dy = final_y - last_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < default_size * 0.8 {
                    return;
                }
            }
            
            self.last_bg_pos = Some((final_x, final_y));
            
            self.map.background_elements.push(map::BackgroundElement {
                x: final_x - default_size / 2.0,
                y: final_y - default_size / 2.0,
                width: default_size,
                height: default_size,
                texture_name: bg_tex.base_path.clone(),
                alpha: self.current_bg_alpha,
                additive: self.current_bg_additive,
                scale: self.current_bg_scale,
                animation_speed: anim_speed,
            });
        }
    }
    
    fn draw_brush_at(&mut self, tile_x: i32, tile_y: i32) {
        let half_size = self.brush_size / 2;
        
        for bx in 0..self.brush_size {
            for by in 0..self.brush_size {
                let x = tile_x - half_size + bx;
                let y = tile_y - half_size + by;
                
                if x >= 0 && x < self.map.width as i32 && y >= 0 && y < self.map.height as i32 {
                    match self.current_tool {
                        EditorTool::Draw => {
                            self.map.tiles[x as usize][y as usize].solid = true;
                            self.map.tiles[x as usize][y as usize].texture_id = self.current_texture;
                            self.map.tiles[x as usize][y as usize].shader_name = self.current_shader.clone();
                            self.mark_lightmap_dirty();
                        }
                        EditorTool::Erase => {
                            self.map.tiles[x as usize][y as usize].solid = false;
                            self.map.tiles[x as usize][y as usize].texture_id = 0;
                            self.mark_lightmap_dirty();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    fn handle_mouse_input(&mut self) {
        let world_x = (mouse_position().0 / self.zoom) + self.camera_x;
        let world_y = (mouse_position().1 / self.zoom) + self.camera_y;
        let tile_x = (world_x / TILE_WIDTH) as i32;
        let tile_y = (world_y / TILE_HEIGHT) as i32;
        
        let is_shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        
        
        if !is_mouse_button_down(MouseButton::Left) {
            self.last_bg_pos = None;
        }
        
        if self.current_tool == EditorTool::Draw || self.current_tool == EditorTool::Erase {
            if is_shift {
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.line_draw_start = Some((tile_x, tile_y));
                }
                
                if is_mouse_button_released(MouseButton::Left) && self.line_draw_start.is_some() {
                    if let Some((start_x, start_y)) = self.line_draw_start {
                        let dx = (tile_x - start_x).abs();
                        let dy = (tile_y - start_y).abs();
                        let steps = dx.max(dy).max(1);
                        
                        for i in 0..=steps {
                            let t = i as f32 / steps as f32;
                            let x = (start_x as f32 + (tile_x - start_x) as f32 * t) as i32;
                            let y = (start_y as f32 + (tile_y - start_y) as f32 * t) as i32;
                            
                            self.draw_brush_at(x, y);
                        }
                        
                        self.line_draw_start = None;
                    }
                }
            } else {
                if !is_mouse_button_down(MouseButton::Left) {
                    self.line_draw_start = None;
                }
                
                if is_mouse_button_down(MouseButton::Left) {
                    self.draw_brush_at(tile_x, tile_y);
                }
            }
        }
        
        
        if self.current_tool == EditorTool::Background && is_mouse_button_down(MouseButton::Left) {
            self.draw_background_at(world_x, world_y);
        }
        
        if is_mouse_button_pressed(MouseButton::Left) {
            match self.current_tool {
                EditorTool::Select => {
                    self.selected_object = None;
                    
                    for (idx, sp) in self.map.spawn_points.iter().enumerate() {
                        let dx = sp.x - world_x;
                        let dy = sp.y - world_y;
                        if dx * dx + dy * dy < 225.0 {
                            self.selected_object = Some(SelectedObject::SpawnPoint(idx));
                            break;
                        }
                    }
                    
                    if self.selected_object.is_none() {
                        for (idx, item) in self.map.items.iter().enumerate() {
                            let dx = item.x - world_x;
                            let dy = item.y - world_y;
                            if dx * dx + dy * dy < 225.0 {
                                self.selected_object = Some(SelectedObject::Item(idx));
                                break;
                            }
                        }
                    }
                    
                    if self.selected_object.is_none() {
                        for (idx, jp) in self.map.jumppads.iter().enumerate() {
                            if world_x >= jp.x && world_x <= jp.x + jp.width &&
                               world_y >= jp.y && world_y <= jp.y + 32.0 {
                                self.selected_object = Some(SelectedObject::JumpPad(idx));
                                break;
                            }
                        }
                    }
                    
                    if self.selected_object.is_none() {
                        for (idx, tp) in self.map.teleporters.iter().enumerate() {
                            if world_x >= tp.x && world_x <= tp.x + tp.width &&
                               world_y >= tp.y && world_y <= tp.y + tp.height {
                                self.selected_object = Some(SelectedObject::Teleporter(idx));
                                break;
                            }
                        }
                    }
                    
                    if self.selected_object.is_none() {
                        for (idx, light) in self.map.lights.iter().enumerate() {
                            let dx = light.x - world_x;
                            let dy = light.y - world_y;
                            if dx * dx + dy * dy < 400.0 {
                                self.selected_object = Some(SelectedObject::Light(idx));
                                break;
                            }
                        }
                    }
                }
                EditorTool::SpawnPoint => {
                    self.map.spawn_points.push(map::SpawnPoint {
                        x: world_x,
                        y: world_y,
                        team: 0,
                    });
                }
                EditorTool::TeleporterDestination => {
                    if let Some(idx) = self.selected_teleporter_index {
                        if idx < self.map.teleporters.len() {
                            self.map.teleporters[idx].dest_x = world_x;
                            self.map.teleporters[idx].dest_y = world_y;
                            self.current_tool = EditorTool::Teleporter;
                            self.selected_teleporter_index = None;
                        }
                    }
                }
                EditorTool::Item => {
                    let item_type = match self.current_item_type {
                        ItemPlaceType::Health25 => map::ItemType::Health25,
                        ItemPlaceType::Health50 => map::ItemType::Health50,
                        ItemPlaceType::Health100 => map::ItemType::Health100,
                        ItemPlaceType::Armor50 => map::ItemType::Armor50,
                        ItemPlaceType::Armor100 => map::ItemType::Armor100,
                        ItemPlaceType::Shotgun => map::ItemType::Shotgun,
                        ItemPlaceType::GrenadeLauncher => map::ItemType::GrenadeLauncher,
                        ItemPlaceType::RocketLauncher => map::ItemType::RocketLauncher,
                        ItemPlaceType::Railgun => map::ItemType::Railgun,
                        ItemPlaceType::Plasmagun => map::ItemType::Plasmagun,
                        ItemPlaceType::BFG => map::ItemType::BFG,
                        ItemPlaceType::Quad => map::ItemType::Quad,
                        ItemPlaceType::Regen => map::ItemType::Regen,
                        ItemPlaceType::Battle => map::ItemType::Battle,
                        ItemPlaceType::Flight => map::ItemType::Flight,
                        ItemPlaceType::Haste => map::ItemType::Haste,
                        ItemPlaceType::Invis => map::ItemType::Invis,
                    };
                    let aligned_y = self.find_floor_y(world_x, world_y);
                    self.map.items.push(map::Item {
                        x: world_x,
                        y: aligned_y,
                        item_type,
                        respawn_time: 0,
                        active: true,
                        vel_x: 0.0,
                        vel_y: 0.0,
                        dropped: false,
                        yaw: 0.0,
                        spin_yaw: 0.0,
                        pitch: 0.0,
                        roll: 0.0,
                        spin_pitch: 0.0,
                        spin_roll: 0.0,
                    });
                }
                EditorTool::JumpPad => {
                    self.map.jumppads.push(map::JumpPad {
                        x: world_x,
                        y: world_y,
                        width: 64.0,
                        force_x: 0.0,
                        force_y: -5.0,
                        cooldown: 0,
                    });
                }
                EditorTool::Teleporter => {
                    let mut clicked_on_existing = false;
                    for (idx, tp) in self.map.teleporters.iter().enumerate() {
                        if world_x >= tp.x && world_x <= tp.x + tp.width &&
                           world_y >= tp.y && world_y <= tp.y + tp.height {
                            self.selected_teleporter_index = Some(idx);
                            self.current_tool = EditorTool::TeleporterDestination;
                            clicked_on_existing = true;
                            break;
                        }
                    }
                    
                    if !clicked_on_existing {
                        self.map.teleporters.push(map::Teleporter {
                            x: world_x,
                            y: world_y,
                            width: 64.0,
                            height: 64.0,
                            dest_x: world_x + 200.0,
                            dest_y: world_y,
                        });
                    }
                }
                EditorTool::Light => {
                    self.map.lights.push(map::LightSource {
                        x: world_x,
                        y: world_y,
                        radius: 250.0,
                        r: 255,
                        g: 230,
                        b: 200,
                        intensity: 1.0,
                        flicker: false,
                    });
                    self.mark_lightmap_dirty();
                }
                EditorTool::Background => {
                    self.draw_background_at(world_x, world_y);
                }
                _ => {}
            }
        }
        
        if is_mouse_button_pressed(MouseButton::Right) {
            self.map.spawn_points.retain(|sp| {
                let dx = sp.x - world_x;
                let dy = sp.y - world_y;
                dx * dx + dy * dy > 400.0
            });
            
            self.map.items.retain(|item| {
                let dx = item.x - world_x;
                let dy = item.y - world_y;
                dx * dx + dy * dy > 400.0
            });
            
            self.map.jumppads.retain(|jp| {
                let dx = jp.x - world_x;
                let dy = jp.y - world_y;
                dx * dx + dy * dy > 900.0
            });
            
            self.map.teleporters.retain(|tp| {
                let dx = tp.x - world_x;
                let dy = tp.y - world_y;
                dx * dx + dy * dy > 900.0
            });
            
            let lights_before = self.map.lights.len();
            self.map.lights.retain(|light| {
                let dx = light.x - world_x;
                let dy = light.y - world_y;
                dx * dx + dy * dy > 400.0
            });
            if self.map.lights.len() != lights_before {
                self.mark_lightmap_dirty();
            }
            
            self.map.background_elements.retain(|bg| {
                let dx = bg.x + bg.width / 2.0 - world_x;
                let dy = bg.y + bg.height / 2.0 - world_y;
                dx * dx + dy * dy > (bg.width * bg.width + bg.height * bg.height) / 4.0
            });
        }
    }
    
    pub fn render(&mut self) {
        let delta_time = get_frame_time();
        self.update_lightmap_rebuild(delta_time);
        
        if self.deferred_renderer.is_none() {
            self.deferred_renderer = Some(deferred_renderer::DeferredRenderer::new(&self.map));
        }
        
        let renderer = self.deferred_renderer.as_mut().unwrap();
        renderer.begin_scene();
        
        if let Some(bg_tex) = &self.background_texture {
            let bg_scale = 2.0 * self.zoom;
            let bg_w = bg_tex.width() * bg_scale;
            let bg_h = bg_tex.height() * bg_scale;
            let start_bg_x = ((self.camera_x * self.zoom / bg_w).floor() * bg_w) - self.camera_x * self.zoom;
            let start_bg_y = ((self.camera_y * self.zoom / bg_h).floor() * bg_h) - self.camera_y * self.zoom;
            
            let tiles_x = ((screen_width() / bg_w).ceil() as i32) + 2;
            let tiles_y = ((screen_height() / bg_h).ceil() as i32) + 2;
            
            for tx in 0..tiles_x {
                for ty in 0..tiles_y {
                    draw_texture_ex(
                        bg_tex,
                        start_bg_x + tx as f32 * bg_w,
                        start_bg_y + ty as f32 * bg_h,
                        Color::from_rgba(255, 255, 255, 40),
                        DrawTextureParams {
                            dest_size: Some(vec2(bg_w, bg_h)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
        
        let time = get_time() as f32;
        
        for bg_elem in &self.map.background_elements {
            let matching_tex = self.background_textures.iter().find(|t| t.base_path == bg_elem.texture_name);
            
            if let Some(bg_tex) = matching_tex {
                let frame_idx = if bg_tex.is_animated && bg_elem.animation_speed > 0.0 {
                    ((time * bg_elem.animation_speed) as usize) % bg_tex.frames.len()
                } else {
                    0
                };
                
                let tex = &bg_tex.frames[frame_idx];
                let screen_x = (bg_elem.x - self.camera_x) * self.zoom;
                let screen_y = (bg_elem.y - self.camera_y) * self.zoom;
                
                let alpha = (bg_elem.alpha * 255.0) as u8;
                let color = Color::from_rgba(255, 255, 255, alpha);
                
                if bg_elem.additive {
                    gl_use_material(get_bg_additive_material());
                    
                    draw_texture_ex(
                        tex,
                        screen_x,
                        screen_y,
                        color,
                        DrawTextureParams {
                            dest_size: Some(vec2(bg_elem.width * self.zoom, bg_elem.height * self.zoom)),
                            ..Default::default()
                        },
                    );
                    
                    gl_use_default_material();
                } else {
                    draw_texture_ex(
                        tex,
                        screen_x,
                        screen_y,
                        color,
                        DrawTextureParams {
                            dest_size: Some(vec2(bg_elem.width * self.zoom, bg_elem.height * self.zoom)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
        
        let start_x = ((self.camera_x / TILE_WIDTH).floor() as i32).max(0);
        let end_x = (((self.camera_x + screen_width() / self.zoom) / TILE_WIDTH).ceil() as i32).min(self.map.width as i32);
        let start_y = ((self.camera_y / TILE_HEIGHT).floor() as i32).max(0);
        let end_y = (((self.camera_y + screen_height() / self.zoom) / TILE_HEIGHT).ceil() as i32).min(self.map.height as i32);
        
        let mut visited = vec![vec![false; self.map.height]; self.map.width];
        
        for x in start_x..end_x {
            for y in start_y..end_y {
                let tile = &self.map.tiles[x as usize][y as usize];
                
                if tile.solid && !visited[x as usize][y as usize] {
                    let tex_id = tile.texture_id;
                    let mut block_width = 1;
                    let mut block_height = 1;
                    
                    while x + block_width < end_x && 
                          self.map.tiles[(x + block_width) as usize][y as usize].solid && 
                          self.map.tiles[(x + block_width) as usize][y as usize].texture_id == tex_id &&
                          !visited[(x + block_width) as usize][y as usize] {
                        block_width += 1;
                    }
                    
                    let mut height_ok = true;
                    while y + block_height < end_y && height_ok {
                        for bx in 0..block_width {
                            if !self.map.tiles[(x + bx) as usize][(y + block_height) as usize].solid ||
                               self.map.tiles[(x + bx) as usize][(y + block_height) as usize].texture_id != tex_id ||
                               visited[(x + bx) as usize][(y + block_height) as usize] {
                                height_ok = false;
                                break;
                            }
                        }
                        if height_ok {
                            block_height += 1;
                        }
                    }
                    
                    for bx in 0..block_width {
                        for by in 0..block_height {
                            visited[(x + bx) as usize][(y + by) as usize] = true;
                        }
                    }
                    
                    let screen_x = ((x as f32 * TILE_WIDTH) - self.camera_x) * self.zoom;
                    let screen_y = ((y as f32 * TILE_HEIGHT) - self.camera_y) * self.zoom;
                    let block_pixel_w = block_width as f32 * TILE_WIDTH * self.zoom;
                    let block_pixel_h = block_height as f32 * TILE_HEIGHT * self.zoom;
                    
                    let world_x = x as f32 * TILE_WIDTH + self.texture_offset_x;
                    let world_y = y as f32 * TILE_HEIGHT + self.texture_offset_y;
                    
                    if let Some(ref shader_name) = tile.shader_name {
                        let time = get_time() as f32;
                        self.shader_renderer.render_tile_with_shader(
                            shader_name,
                            screen_x,
                            screen_y,
                            block_pixel_w,
                            block_pixel_h,
                            world_x,
                            world_y,
                            time,
                        );
                    } else if let Some(texture) = self.tile_textures.get(&tex_id) {
                        let base_tex_w = texture.width();
                        let base_tex_h = texture.height();
                        
                        let scaled_tex_w = base_tex_w * self.texture_scale * self.zoom;
                        let scaled_tex_h = base_tex_h * self.texture_scale * self.zoom;
                        
                        let start_offset_x = world_x % scaled_tex_w;
                        let start_offset_y = world_y % scaled_tex_h;
                        
                        let tiles_x = ((block_pixel_w + start_offset_x) / scaled_tex_w).ceil() as i32 + 1;
                        let tiles_y = ((block_pixel_h + start_offset_y) / scaled_tex_h).ceil() as i32 + 1;
                        
                        for ty in 0..tiles_y {
                            for tx in 0..tiles_x {
                                let tile_x_world = world_x + tx as f32 * scaled_tex_w - start_offset_x;
                                let tile_y_world = world_y + ty as f32 * scaled_tex_h - start_offset_y;
                                
                                let tile_x_screen = (tile_x_world - self.camera_x) * self.zoom;
                                let tile_y_screen = (tile_y_world - self.camera_y) * self.zoom;
                                
                                let clip_left = (screen_x - tile_x_screen).max(0.0);
                                let clip_top = (screen_y - tile_y_screen).max(0.0);
                                let clip_right = ((tile_x_screen + scaled_tex_w) - (screen_x + block_pixel_w)).max(0.0);
                                let clip_bottom = ((tile_y_screen + scaled_tex_h) - (screen_y + block_pixel_h)).max(0.0);
                                
                                let visible_w = scaled_tex_w - clip_left - clip_right;
                                let visible_h = scaled_tex_h - clip_top - clip_bottom;
                                
                                if visible_w > 0.0 && visible_h > 0.0 {
                                    let uv_x = clip_left / scaled_tex_w;
                                    let uv_y = clip_top / scaled_tex_h;
                                    let uv_w = visible_w / scaled_tex_w;
                                    let uv_h = visible_h / scaled_tex_h;
                                    
                                    draw_texture_ex(
                                        texture,
                                        tile_x_screen + clip_left,
                                        tile_y_screen + clip_top,
                                        WHITE,
                                        DrawTextureParams {
                                            dest_size: Some(vec2(visible_w, visible_h)),
                                            source: Some(Rect::new(
                                                uv_x * base_tex_w, 
                                                uv_y * base_tex_h, 
                                                uv_w * base_tex_w, 
                                                uv_h * base_tex_h
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                }
                            }
                        }
                    } else {
                        let base_color = procedural_tiles::get_base_color_pub(tex_id);
                        draw_rectangle(screen_x, screen_y, block_pixel_w, block_pixel_h, base_color);
                    }
                    
                    draw_rectangle_lines(
                        screen_x,
                        screen_y,
                        block_pixel_w,
                        block_pixel_h,
                        2.0,
                        Color::from_rgba(0, 0, 0, 80)
                    );
                }
                
                if self.show_grid {
                    let screen_x = ((x as f32 * TILE_WIDTH) - self.camera_x) * self.zoom;
                    let screen_y = ((y as f32 * TILE_HEIGHT) - self.camera_y) * self.zoom;
                    draw_rectangle_lines(screen_x, screen_y, TILE_WIDTH * self.zoom, TILE_HEIGHT * self.zoom, 1.0, Color::from_rgba(50, 50, 60, 60));
                }
            }
        }
        
        for (idx, sp) in self.map.spawn_points.iter().enumerate() {
            let screen_x = (sp.x - self.camera_x) * self.zoom;
            let screen_y = (sp.y - self.camera_y) * self.zoom;
            let is_selected = matches!(&self.selected_object, Some(SelectedObject::SpawnPoint(i)) if *i == idx);
            
            if is_selected {
                draw_circle(screen_x, screen_y, 16.0, Color::from_rgba(255, 255, 0, 200));
            }
            draw_circle(screen_x, screen_y, 12.0, GREEN);
            draw_circle(screen_x, screen_y, 10.0, Color::from_rgba(0, 255, 0, 100));
            draw_text("S", screen_x - 4.0, screen_y + 4.0, 16.0, BLACK);
        }
        
        for (idx, item) in self.map.items.iter().enumerate() {
            let screen_x = (item.x - self.camera_x) * self.zoom;
            let screen_y = (item.y - self.camera_y) * self.zoom;
            let is_selected = matches!(&self.selected_object, Some(SelectedObject::Item(i)) if *i == idx);
            
            if is_selected {
                draw_circle(screen_x, screen_y, 18.0, Color::from_rgba(255, 255, 0, 200));
            }
            
            if let Some(icon) = self.item_icons.get_icon(&item.item_type) {
                let icon_size = 24.0;
                draw_texture_ex(
                    icon,
                    screen_x - icon_size / 2.0,
                    screen_y - icon_size / 2.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(icon_size, icon_size)),
                        ..Default::default()
                    },
                );
            } else {
            let color = match item.item_type {
                map::ItemType::Health25 | map::ItemType::Health50 | map::ItemType::Health100 => RED,
                map::ItemType::Armor50 | map::ItemType::Armor100 => YELLOW,
                map::ItemType::Quad | map::ItemType::Regen | map::ItemType::Battle | 
                map::ItemType::Flight | map::ItemType::Haste | map::ItemType::Invis => PURPLE,
                _ => BLUE,
            };
            draw_circle(screen_x, screen_y, 8.0, color);
            }
        }
        
        for (idx, jp) in self.map.jumppads.iter().enumerate() {
            let screen_x = (jp.x - self.camera_x) * self.zoom;
            let screen_y = (jp.y - self.camera_y) * self.zoom;
            let is_selected = matches!(&self.selected_object, Some(SelectedObject::JumpPad(i)) if *i == idx);
            
            if is_selected {
                draw_rectangle(screen_x - 2.0, screen_y - 2.0, jp.width + 4.0, 20.0, Color::from_rgba(255, 255, 0, 200));
            }
            draw_rectangle(screen_x, screen_y, jp.width * self.zoom, 16.0 * self.zoom, Color::from_rgba(255, 180, 60, 200));
        }
        
        for (idx, tp) in self.map.teleporters.iter().enumerate() {
            let screen_x = (tp.x - self.camera_x) * self.zoom;
            let screen_y = (tp.y - self.camera_y) * self.zoom;
            let is_selected_teleporter = self.selected_teleporter_index == Some(idx);
            let is_selected_object = matches!(&self.selected_object, Some(SelectedObject::Teleporter(i)) if *i == idx);
            
            let teleporter_color = if is_selected_teleporter {
                Color::from_rgba(100, 200, 255, 200)
            } else {
                Color::from_rgba(60, 120, 180, 150)
            };
            
            draw_rectangle(screen_x, screen_y, tp.width * self.zoom, tp.height * self.zoom, teleporter_color);
            
            if is_selected_teleporter {
                draw_rectangle_lines(screen_x - 2.0, screen_y - 2.0, tp.width + 4.0, tp.height + 4.0, 3.0, Color::from_rgba(255, 255, 100, 255));
            } else if is_selected_object {
                draw_rectangle_lines(screen_x - 2.0, screen_y - 2.0, tp.width + 4.0, tp.height + 4.0, 3.0, Color::from_rgba(255, 255, 0, 255));
            }
            
            let dest_screen_x = (tp.dest_x - self.camera_x) * self.zoom;
            let dest_screen_y = (tp.dest_y - self.camera_y) * self.zoom;
            
            let line_color = if is_selected_teleporter || is_selected_object {
                Color::from_rgba(255, 255, 100, 255)
            } else {
                Color::from_rgba(0, 255, 255, 200)
            };
            
            draw_line(screen_x + tp.width * self.zoom / 2.0, screen_y + tp.height * self.zoom / 2.0, dest_screen_x, dest_screen_y, if is_selected_teleporter || is_selected_object { 3.0 } else { 2.0 }, line_color);
            
            draw_circle(dest_screen_x, dest_screen_y, 6.0, line_color);
            draw_circle(dest_screen_x, dest_screen_y, 3.0, Color::from_rgba(255, 255, 255, 200));
        }
        
        renderer.end_scene();
        
        let empty_linear_lights = Vec::new();
        renderer.apply_lighting(&self.map, &self.map.lights, &self.map.lights, &empty_linear_lights, self.camera_x, self.camera_y, self.zoom, self.ambient_light, false, false, false);
        
        if self.show_nav_graph {
            if let Some(ref graph) = self.nav_graph {
                graph.render(self.camera_x, self.camera_y, self.show_nav_edges);
            }
        }
        
        let (mouse_x_vis, mouse_y_vis) = mouse_position();
        let world_x_vis = mouse_x_vis + self.camera_x;
        let world_y_vis = mouse_y_vis + self.camera_y;
        
        for (idx, light) in self.map.lights.iter().enumerate() {
            let screen_x = (light.x - self.camera_x) * self.zoom;
            let screen_y = (light.y - self.camera_y) * self.zoom;
            let is_selected = matches!(&self.selected_object, Some(SelectedObject::Light(i)) if *i == idx);
            
            if is_selected {
                draw_circle(screen_x, screen_y, 12.0, Color::from_rgba(255, 255, 0, 200));
            }
            
            draw_circle(screen_x, screen_y, 8.0, Color::from_rgba(255, 255, 255, 200));
            draw_circle(screen_x, screen_y, 4.0, Color::from_rgba(light.r, light.g, light.b, 255));
            if light.flicker {
                draw_circle_lines(screen_x, screen_y, 10.0, 1.5, Color::from_rgba(255, 200, 100, 180));
            }
            
            let dx = light.x - world_x_vis;
            let dy = light.y - world_y_vis;
            if (dx * dx + dy * dy < 400.0 && self.current_tool == EditorTool::Light) || is_selected {
                let info = format!("R:{:.0} I:{:.1}", light.radius, light.intensity);
                draw_text(&info, screen_x + 12.0, screen_y - 8.0, 16.0, WHITE);
            }
        }
        
        let (mouse_x, mouse_y) = mouse_position();
        let world_x = mouse_x + self.camera_x;
        let world_y = mouse_y + self.camera_y;
        let tile_x = (world_x / TILE_WIDTH) as i32;
        let tile_y = (world_y / TILE_HEIGHT) as i32;
        
        if tile_x >= 0 && tile_x < self.map.width as i32 && tile_y >= 0 && tile_y < self.map.height as i32 {
            match self.current_tool {
                EditorTool::Draw | EditorTool::Erase => {
                    let half_size = self.brush_size / 2;
                    
                    for bx in 0..self.brush_size {
                        for by in 0..self.brush_size {
                            let x = tile_x - half_size + bx;
                            let y = tile_y - half_size + by;
                            
                            if x >= 0 && x < self.map.width as i32 && y >= 0 && y < self.map.height as i32 {
                                let sx = ((x as f32 * TILE_WIDTH) - self.camera_x) * self.zoom;
                                let sy = ((y as f32 * TILE_HEIGHT) - self.camera_y) * self.zoom;
                                
                                let color = if self.current_tool == EditorTool::Draw {
                                    Color::from_rgba(255, 255, 255, 100)
                                } else {
                                    Color::from_rgba(255, 0, 0, 100)
                                };
                                
                                draw_rectangle(sx, sy, TILE_WIDTH * self.zoom, TILE_HEIGHT * self.zoom, color);
                            }
                        }
                    }
                    
                    if let Some((start_x, start_y)) = self.line_draw_start {
                        let start_sx = ((start_x as f32 * TILE_WIDTH + TILE_WIDTH / 2.0) - self.camera_x) * self.zoom;
                        let start_sy = ((start_y as f32 * TILE_HEIGHT + TILE_HEIGHT / 2.0) - self.camera_y) * self.zoom;
                        let end_sx = ((tile_x as f32 * TILE_WIDTH + TILE_WIDTH / 2.0) - self.camera_x) * self.zoom;
                        let end_sy = ((tile_y as f32 * TILE_HEIGHT + TILE_HEIGHT / 2.0) - self.camera_y) * self.zoom;
                        
                        draw_line(start_sx, start_sy, end_sx, end_sy, 3.0, Color::from_rgba(255, 255, 0, 200));
                        draw_circle(start_sx, start_sy, 5.0, Color::from_rgba(255, 255, 0, 255));
                    }
                }
                EditorTool::Background => {
                    let cursor_size = 64.0 * self.current_bg_scale * self.zoom;
                    let (cursor_x, cursor_y) = if self.bg_snap_to_grid {
                        let grid_x = (((world_x / 32.0).round() * 32.0) - self.camera_x) * self.zoom;
                        let grid_y = (((world_y / 32.0).round() * 32.0) - self.camera_y) * self.zoom;
                        (grid_x, grid_y)
                    } else {
                        (mouse_x, mouse_y)
                    };
                    
                    draw_rectangle_lines(
                        cursor_x - cursor_size / 2.0,
                        cursor_y - cursor_size / 2.0,
                        cursor_size,
                        cursor_size,
                        2.0,
                        Color::from_rgba(100, 200, 255, 200)
                    );
                    
                    draw_circle(cursor_x, cursor_y, 3.0, Color::from_rgba(100, 200, 255, 255));
                }
                _ => {
                    draw_circle(mouse_x, mouse_y, 8.0, Color::from_rgba(255, 255, 255, 150));
                }
            }
        }
        
        self.render_ui();
    }
    
    fn draw_property_button(&self, x: f32, y: f32, text: &str, is_hover: bool) -> bool {
        let btn_w = 25.0;
        let btn_h = 20.0;
        
        let bg_color = if is_hover {
            Color::from_rgba(80, 100, 140, 255)
        } else {
            Color::from_rgba(50, 60, 80, 255)
        };
        
        draw_rectangle(x, y, btn_w, btn_h, bg_color);
        draw_rectangle_lines(x, y, btn_w, btn_h, 1.0, Color::from_rgba(120, 130, 150, 255));
        
        let text_dims = measure_text(text, None, 16, 1.0);
        draw_text(text, x + (btn_w - text_dims.width) / 2.0, y + 15.0, 16.0, WHITE);
        
        is_hover && is_mouse_button_pressed(MouseButton::Left)
    }
    
    fn render_texture_picker(&mut self) {
        let picker_w = 800.0;
        let picker_h = 600.0;
        let picker_x = screen_width() / 2.0 - picker_w / 2.0;
        let picker_y = screen_height() / 2.0 - picker_h / 2.0;
        
        draw_rectangle(picker_x, picker_y, picker_w, picker_h, Color::from_rgba(15, 18, 22, 250));
        draw_rectangle_lines(picker_x, picker_y, picker_w, picker_h, 3.0, Color::from_rgba(100, 120, 140, 255));
        
        draw_text("Wall Textures", picker_x + 20.0, picker_y + 35.0, 26.0, WHITE);
        draw_text("Click to select | Ctrl+T or ESC to close | Mouse wheel to scroll", picker_x + 20.0, picker_y + 60.0, 16.0, Color::from_rgba(180, 180, 180, 255));
        
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            self.texture_picker_scroll -= scroll * 30.0;
            self.texture_picker_scroll = self.texture_picker_scroll.max(0.0);
        }
        
        let thumb_size = 80.0;
        let padding = 10.0;
        let start_y = picker_y + 80.0 - self.texture_picker_scroll;
        let start_x = picker_x + 20.0;
        let cols = ((picker_w - 40.0) / (thumb_size + padding)) as usize;
        
        let (mx, my) = mouse_position();
        
        for (idx, wall_tex) in self.wall_textures.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;
            
            let tx = start_x + col as f32 * (thumb_size + padding);
            let ty = start_y + row as f32 * (thumb_size + padding + 20.0);
            
            if ty + thumb_size + 20.0 < picker_y + 70.0 {
                continue;
            }
            
            if ty > picker_y + picker_h - 20.0 {
                continue;
            }
            
            let is_selected = idx == self.current_texture as usize;
            let is_hover = mx >= tx && mx <= tx + thumb_size && my >= ty && my <= ty + thumb_size 
                && my >= picker_y + 70.0 && my <= picker_y + picker_h;
            
            let border_color = if is_selected {
                Color::from_rgba(100, 200, 255, 255)
            } else if is_hover {
                Color::from_rgba(180, 180, 180, 255)
            } else {
                Color::from_rgba(60, 65, 70, 255)
            };
            
            let border_width = if is_selected { 3.0 } else { 2.0 };
            
            draw_rectangle(tx, ty, thumb_size, thumb_size, Color::from_rgba(30, 32, 35, 255));
            
            draw_texture_ex(
                &wall_tex.texture,
                tx,
                ty,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(thumb_size, thumb_size)),
                    ..Default::default()
                },
            );
            
            draw_rectangle_lines(tx, ty, thumb_size, thumb_size, border_width, border_color);
            
            let name_y = ty + thumb_size + 14.0;
            let short_name = if wall_tex.name.len() > 10 {
                format!("{}...", &wall_tex.name[..7])
            } else {
                wall_tex.name.clone()
            };
            draw_text(&short_name, tx + 2.0, name_y, 12.0, Color::from_rgba(200, 200, 200, 255));
            
            if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                self.current_texture = idx as u16;
                self.show_texture_picker = false;
            }
        }
    }
    
    fn render_bg_texture_picker(&mut self) {
        let picker_w = 900.0;
        let picker_h = 700.0;
        let picker_x = screen_width() / 2.0 - picker_w / 2.0;
        let picker_y = screen_height() / 2.0 - picker_h / 2.0;
        
        draw_rectangle(picker_x, picker_y, picker_w, picker_h, Color::from_rgba(15, 18, 22, 250));
        draw_rectangle_lines(picker_x, picker_y, picker_w, picker_h, 3.0, Color::from_rgba(100, 120, 140, 255));
        
        draw_text("Background Shaders & Effects", picker_x + 20.0, picker_y + 35.0, 26.0, WHITE);
        draw_text("Click to select | Ctrl+B or ESC to close | Mouse wheel to scroll", picker_x + 20.0, picker_y + 60.0, 16.0, Color::from_rgba(180, 180, 180, 255));
        
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            self.texture_picker_scroll -= scroll * 30.0;
            self.texture_picker_scroll = self.texture_picker_scroll.max(0.0);
        }
        
        let thumb_size = 72.0;
        let padding = 8.0;
        let mut start_y = picker_y + 90.0 - self.texture_picker_scroll;
        let start_x = picker_x + 20.0;
        let cols = ((picker_w - 40.0) / (thumb_size + padding)) as usize;
        
        let (mx, my) = mouse_position();
        
        let static_textures: Vec<(usize, &BackgroundTexture)> = self.background_textures.iter()
            .enumerate()
            .filter(|(_, t)| !t.is_animated && !t.has_wave_effect)
            .collect();
        
        let animated_textures: Vec<(usize, &BackgroundTexture)> = self.background_textures.iter()
            .enumerate()
            .filter(|(_, t)| t.is_animated)
            .collect();
        
        let wave_textures: Vec<(usize, &BackgroundTexture)> = self.background_textures.iter()
            .enumerate()
            .filter(|(_, t)| t.has_wave_effect && !t.is_animated)
            .collect();
        
        if !static_textures.is_empty() {
            draw_text("STATIC", start_x, start_y, 20.0, Color::from_rgba(100, 200, 255, 255));
            start_y += 25.0;
            
            for (list_idx, (idx, bg_tex)) in static_textures.iter().enumerate() {
                let col = list_idx % cols;
                let row = list_idx / cols;
                
                let tx = start_x + col as f32 * (thumb_size + padding);
                let ty = start_y + row as f32 * (thumb_size + padding + 18.0);
                
                if ty + thumb_size + 18.0 < picker_y + 80.0 || ty > picker_y + picker_h - 20.0 {
                    continue;
                }
                
                let is_selected = *idx == self.current_bg_texture;
                let is_hover = mx >= tx && mx <= tx + thumb_size && my >= ty && my <= ty + thumb_size 
                    && my >= picker_y + 80.0 && my <= picker_y + picker_h;
                
                self.render_bg_thumb(tx, ty, thumb_size, bg_tex, is_selected, is_hover);
                
                if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                    self.current_bg_texture = *idx;
                    self.show_bg_texture_picker = false;
                }
            }
            
            start_y += ((static_textures.len() + cols - 1) / cols) as f32 * (thumb_size + padding + 18.0) + 20.0;
        }
        
        if !animated_textures.is_empty() {
            draw_text("ANIMATED", start_x, start_y, 20.0, Color::from_rgba(255, 200, 100, 255));
            start_y += 25.0;
            
            for (list_idx, (idx, bg_tex)) in animated_textures.iter().enumerate() {
                let col = list_idx % cols;
                let row = list_idx / cols;
                
                let tx = start_x + col as f32 * (thumb_size + padding);
                let ty = start_y + row as f32 * (thumb_size + padding + 18.0);
                
                if ty + thumb_size + 18.0 < picker_y + 80.0 || ty > picker_y + picker_h - 20.0 {
                    continue;
                }
                
                let is_selected = *idx == self.current_bg_texture;
                let is_hover = mx >= tx && mx <= tx + thumb_size && my >= ty && my <= ty + thumb_size 
                    && my >= picker_y + 80.0 && my <= picker_y + picker_h;
                
                self.render_bg_thumb(tx, ty, thumb_size, bg_tex, is_selected, is_hover);
                
                if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                    self.current_bg_texture = *idx;
                    self.show_bg_texture_picker = false;
                }
            }
            
            start_y += ((animated_textures.len() + cols - 1) / cols) as f32 * (thumb_size + padding + 18.0) + 20.0;
        }
        
        if !wave_textures.is_empty() {
            draw_text("WAVE EFFECTS", start_x, start_y, 20.0, Color::from_rgba(255, 100, 255, 255));
            start_y += 25.0;
            
            for (list_idx, (idx, bg_tex)) in wave_textures.iter().enumerate() {
                let col = list_idx % cols;
                let row = list_idx / cols;
                
                let tx = start_x + col as f32 * (thumb_size + padding);
                let ty = start_y + row as f32 * (thumb_size + padding + 18.0);
                
                if ty + thumb_size + 18.0 < picker_y + 80.0 || ty > picker_y + picker_h - 20.0 {
                    continue;
                }
                
                let is_selected = *idx == self.current_bg_texture;
                let is_hover = mx >= tx && mx <= tx + thumb_size && my >= ty && my <= ty + thumb_size 
                    && my >= picker_y + 80.0 && my <= picker_y + picker_h;
                
                self.render_bg_thumb(tx, ty, thumb_size, bg_tex, is_selected, is_hover);
                
                if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                    self.current_bg_texture = *idx;
                    self.show_bg_texture_picker = false;
                }
            }
        }
    }
    
    fn render_bg_thumb(&self, tx: f32, ty: f32, thumb_size: f32, bg_tex: &BackgroundTexture, is_selected: bool, is_hover: bool) {
        let border_color = if is_selected {
            Color::from_rgba(100, 200, 255, 255)
        } else if is_hover {
            Color::from_rgba(180, 180, 180, 255)
        } else {
            Color::from_rgba(60, 65, 70, 255)
        };
        
        let border_width = if is_selected { 3.0 } else { 2.0 };
        
        draw_rectangle(tx, ty, thumb_size, thumb_size, Color::from_rgba(30, 32, 35, 255));
        
        let frame_idx = if bg_tex.is_animated {
            ((get_time() as f32 * 5.0) as usize) % bg_tex.frames.len()
        } else {
            0
        };
        
        if let Some(tex) = bg_tex.frames.get(frame_idx) {
            draw_texture_ex(
                tex,
                tx,
                ty,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(thumb_size, thumb_size)),
                    ..Default::default()
                },
            );
        }
        
        draw_rectangle_lines(tx, ty, thumb_size, thumb_size, border_width, border_color);
        
        if bg_tex.is_animated {
            draw_text(&format!("{}f", bg_tex.frames.len()), tx + 4.0, ty + 16.0, 12.0, Color::from_rgba(255, 200, 100, 255));
        }
        
        if bg_tex.has_wave_effect {
            draw_circle(tx + thumb_size - 8.0, ty + 8.0, 4.0, Color::from_rgba(255, 100, 255, 255));
        }
        
        let name_y = ty + thumb_size + 14.0;
        let short_name = if bg_tex.name.len() > 9 {
            format!("{}...", &bg_tex.name[..6])
        } else {
            bg_tex.name.clone()
        };
        draw_text(&short_name, tx + 2.0, name_y, 11.0, Color::from_rgba(200, 200, 200, 255));
    }
    
    fn render_shader_picker(&mut self) {
        let picker_w = 500.0;
        let picker_h = 400.0;
        let picker_x = screen_width() / 2.0 - picker_w / 2.0;
        let picker_y = screen_height() / 2.0 - picker_h / 2.0;
        
        draw_rectangle(picker_x, picker_y, picker_w, picker_h, Color::from_rgba(15, 18, 22, 250));
        draw_rectangle_lines(picker_x, picker_y, picker_w, picker_h, 3.0, Color::from_rgba(100, 120, 140, 255));
        
        draw_text("Tile Shaders (Q3 Style)", picker_x + 20.0, picker_y + 35.0, 26.0, WHITE);
        draw_text("Click to select | Ctrl+X or ESC to close", picker_x + 20.0, picker_y + 60.0, 16.0, Color::from_rgba(180, 180, 180, 255));
        
        let item_height = 50.0;
        let start_y = picker_y + 90.0;
        let (mx, my) = mouse_position();
        
        let none_y = start_y;
        let none_hover = mx >= picker_x + 20.0 && mx <= picker_x + picker_w - 20.0 &&
                         my >= none_y && my <= none_y + item_height;
        let none_selected = self.current_shader.is_none();
        
        let bg_color = if none_selected {
            Color::from_rgba(50, 100, 180, 255)
        } else if none_hover {
            Color::from_rgba(45, 50, 60, 255)
        } else {
            Color::from_rgba(30, 35, 42, 255)
        };
        
        draw_rectangle(picker_x + 20.0, none_y, picker_w - 40.0, item_height, bg_color);
        draw_rectangle_lines(picker_x + 20.0, none_y, picker_w - 40.0, item_height, 2.0,
            if none_selected { Color::from_rgba(100, 200, 255, 255) } else { Color::from_rgba(60, 65, 75, 255) });
        
        draw_text("No Shader (Normal Texture)", picker_x + 35.0, none_y + 30.0, 20.0, WHITE);
        
        if none_hover && is_mouse_button_pressed(MouseButton::Left) {
            self.current_shader = None;
            self.show_shader_picker = false;
        }
        
        for (idx, shader_name) in self.available_shaders.iter().enumerate() {
            let y = start_y + (idx + 1) as f32 * (item_height + 10.0);
            
            if y > picker_y + picker_h - 20.0 {
                break;
            }
            
            let is_selected = self.current_shader.as_ref().map_or(false, |s| s == shader_name);
            let is_hover = mx >= picker_x + 20.0 && mx <= picker_x + picker_w - 20.0 &&
                          my >= y && my <= y + item_height;
            
            let bg_color = if is_selected {
                Color::from_rgba(50, 100, 180, 255)
            } else if is_hover {
                Color::from_rgba(45, 50, 60, 255)
            } else {
                Color::from_rgba(30, 35, 42, 255)
            };
            
            draw_rectangle(picker_x + 20.0, y, picker_w - 40.0, item_height, bg_color);
            draw_rectangle_lines(picker_x + 20.0, y, picker_w - 40.0, item_height, 2.0,
                if is_selected { Color::from_rgba(100, 200, 255, 255) } else { Color::from_rgba(60, 65, 75, 255) });
            
            draw_text(shader_name, picker_x + 35.0, y + 22.0, 20.0, WHITE);
            
            let desc = match shader_name.as_str() {
                "border11light" => "Border trim - sin wave glow (Q3)",
                "dooreye_purple" => "Door Eye - triangle wave (Q3)",
                "metal_glow" => "Metal with glowing energy",
                "tech_panel" => "Tech panel with effects",
                "concrete_detail" => "Concrete with detail overlay",
                "metal_trim" => "Metallic trim with reflection",
                _ => "Custom shader",
            };
            draw_text(desc, picker_x + 35.0, y + 40.0, 14.0, Color::from_rgba(180, 180, 180, 255));
            
            if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                self.current_shader = Some(shader_name.clone());
                self.show_shader_picker = false;
            }
        }
    }
    
    fn render_properties_panel(&mut self) {
        if !self.show_properties {
            return;
        }
        
        if let Some(obj) = &self.selected_object.clone() {
            let panel_width = 300.0;
            let panel_x = screen_width() - panel_width - 10.0;
            let panel_y = 45.0;
            let panel_bg = Color::from_rgba(25, 30, 35, 240);
            
            let (title, properties) = match obj {
                SelectedObject::SpawnPoint(idx) => {
                    if let Some(sp) = self.map.spawn_points.get(*idx) {
                        (
                            "Spawn Point Properties".to_string(),
                            vec![
                                format!("Position X: {:.1}", sp.x),
                                format!("Position Y: {:.1}", sp.y),
                                format!("Team: {}", sp.team),
                            ]
                        )
                    } else {
                        return;
                    }
                },
                SelectedObject::Item(idx) => {
                    if let Some(item) = self.map.items.get(*idx) {
                        let type_name = match item.item_type {
                            map::ItemType::Health25 => "Health +25",
                            map::ItemType::Health50 => "Health +50",
                            map::ItemType::Health100 => "Mega Health",
                            map::ItemType::Armor50 => "Armor +50",
                            map::ItemType::Armor100 => "Armor +100",
                            map::ItemType::Shotgun => "Shotgun",
                            map::ItemType::GrenadeLauncher => "Grenade Launcher",
                            map::ItemType::RocketLauncher => "Rocket Launcher",
                            map::ItemType::Railgun => "Railgun",
                            map::ItemType::Plasmagun => "Plasma Gun",
                            map::ItemType::BFG => "BFG",
                            map::ItemType::Quad => "Quad Damage",
                            map::ItemType::Regen => "Regeneration",
                            map::ItemType::Battle => "Battle Suit",
                            map::ItemType::Flight => "Flight",
                            map::ItemType::Haste => "Haste",
                            map::ItemType::Invis => "Invisibility",
                        };
                        (
                            "Item Properties".to_string(),
                            vec![
                                format!("Type: {}", type_name),
                                format!("Position X: {:.1}", item.x),
                                format!("Position Y: {:.1}", item.y),
                                format!("Active: {}", item.active),
                                format!("Respawn: {}s", item.respawn_time),
                            ]
                        )
                    } else {
                        return;
                    }
                },
                SelectedObject::JumpPad(idx) => {
                    if let Some(jp) = self.map.jumppads.get(*idx) {
                        (
                            "Jump Pad Properties".to_string(),
                            vec![
                                format!("Position X: {:.1}", jp.x),
                                format!("Position Y: {:.1}", jp.y),
                                format!("Width: {:.1}", jp.width),
                                format!("Force X: {:.1}", jp.force_x),
                                format!("Force Y: {:.1}", jp.force_y),
                                String::new(),
                                "+/-: Force Y (вверх/вниз)".to_string(),
                                "[/]: Force X (влево/вправо)".to_string(),
                            ]
                        )
                    } else {
                        return;
                    }
                },
                SelectedObject::Teleporter(idx) => {
                    if let Some(tp) = self.map.teleporters.get(*idx) {
                        (
                            "Teleporter Properties".to_string(),
                            vec![
                                format!("Position X: {:.1}", tp.x),
                                format!("Position Y: {:.1}", tp.y),
                                format!("Width: {:.1}", tp.width),
                                format!("Height: {:.1}", tp.height),
                                format!("Dest X: {:.1}", tp.dest_x),
                                format!("Dest Y: {:.1}", tp.dest_y),
                                String::new(),
                                "R: Set destination (then click)".to_string(),
                                "Shift+Arrows: Resize".to_string(),
                            ]
                        )
                    } else {
                        return;
                    }
                },
                SelectedObject::Light(idx) => {
                    if let Some(light) = self.map.lights.get(*idx) {
                        (
                            "Light Source Properties".to_string(),
                            vec![
                                format!("Position X: {:.1}", light.x),
                                format!("Position Y: {:.1}", light.y),
                                format!("Radius: {:.1}", light.radius),
                                format!("Intensity: {:.2}", light.intensity),
                                format!("Color: RGB({}, {}, {})", light.r, light.g, light.b),
                                format!("Flicker: {}", if light.flicker { "Yes" } else { "No" }),
                                String::new(),
                                "[/]: Радиус (±20)".to_string(),
                                "+/-: Интенсивность (±0.1)".to_string(),
                                "F: Flicker вкл/выкл".to_string(),
                            ]
                        )
                    } else {
                        return;
                    }
                },
                SelectedObject::BackgroundElement(idx) => {
                    if let Some(bg) = self.map.background_elements.get(*idx) {
                        (
                            "Background Element Properties".to_string(),
                            vec![
                                format!("Position X: {:.1}", bg.x),
                                format!("Position Y: {:.1}", bg.y),
                                format!("Width: {:.1}", bg.width),
                                format!("Height: {:.1}", bg.height),
                                format!("Alpha: {:.2}", bg.alpha),
                                format!("Scale: {:.2}", bg.scale),
                                format!("Additive: {}", if bg.additive { "Yes" } else { "No" }),
                                format!("Texture: {}", bg.texture_name.split('/').last().unwrap_or("unknown")),
                            ]
                        )
                    } else {
                        return;
                    }
                },
            };
            
            let panel_height = 60.0 + properties.len() as f32 * 22.0;
            
            draw_rectangle(panel_x, panel_y, panel_width, panel_height, panel_bg);
            draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 2.0, Color::from_rgba(70, 80, 90, 255));
            
            draw_text(&title, panel_x + 10.0, panel_y + 25.0, 20.0, YELLOW);
            
            let mut y = panel_y + 50.0;
            for prop in &properties {
                if prop.is_empty() {
                    y += 10.0;
                } else {
                    draw_text(prop, panel_x + 15.0, y, 16.0, WHITE);
                    y += 22.0;
                }
            }
        } else if self.show_properties {
            let panel_width = 300.0;
            let panel_x = screen_width() - panel_width - 10.0;
            let panel_y = 45.0;
            let panel_height = 80.0;
            let panel_bg = Color::from_rgba(25, 30, 35, 240);
            
            draw_rectangle(panel_x, panel_y, panel_width, panel_height, panel_bg);
            draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 2.0, Color::from_rgba(70, 80, 90, 255));
            
            draw_text("Properties Panel", panel_x + 10.0, panel_y + 25.0, 20.0, YELLOW);
            draw_text("No object selected", panel_x + 15.0, panel_y + 50.0, 16.0, Color::from_rgba(150, 150, 150, 255));
        }
    }
    
    fn render_ui(&mut self) {
        let panel_bg = Color::from_rgba(20, 25, 30, 230);
        draw_rectangle(0.0, 0.0, screen_width(), 35.0, panel_bg);
        
        let tex_name = if (self.current_texture as usize) < self.wall_textures.len() {
            &self.wall_textures[self.current_texture as usize].name
        } else {
            "unknown"
        };
        
        let tool_name = match self.current_tool {
            EditorTool::Select => "Select & Edit".to_string(),
            EditorTool::Draw => {
                let shader_str = if let Some(ref sh) = self.current_shader {
                    format!(" Shader:{}", sh)
                } else {
                    "".to_string()
                };
                format!("Draw ({}) Scale: {:.2}x Offset: ({:.0},{:.0}) Brush: {}x{}{}", 
                    tex_name, self.texture_scale, self.texture_offset_x, self.texture_offset_y, 
                    self.brush_size, self.brush_size, shader_str)
            }
            EditorTool::Erase => format!("Erase Brush: {}x{}", self.brush_size, self.brush_size),
            EditorTool::SpawnPoint => "Spawn Point".to_string(),
            EditorTool::Item => format!("Item ({})", self.current_item_type.to_string()),
            EditorTool::JumpPad => "Jump Pad".to_string(),
            EditorTool::Teleporter => "Teleporter (Click to place or edit)".to_string(),
            EditorTool::TeleporterDestination => "Set Teleporter Destination".to_string(),
            EditorTool::Light => "Light Source".to_string(),
            EditorTool::Background => {
                let bg_name = if !self.background_textures.is_empty() {
                    &self.background_textures[self.current_bg_texture].name
                } else {
                    "none"
                };
                format!("Background ({}) A:{:.1} S:{:.1} {} {}", 
                    bg_name, self.current_bg_alpha, self.current_bg_scale,
                    if self.current_bg_additive { "ADD" } else { "ALPHA" },
                    if self.bg_snap_to_grid { "SNAP" } else { "FREE" })
            },
        };
        
        super::ui::render_compact_status_bar(&self.map_name, &tool_name, self.show_grid, self.zoom);
        
        self.render_properties_panel();
        
        if self.show_texture_picker {
            self.render_texture_picker();
        }
        
        if self.show_bg_texture_picker {
            self.render_bg_texture_picker();
        }
        
        if self.show_shader_picker {
            self.render_shader_picker();
        }
        
        if self.show_help {
            self.help_panel.render(
                self.current_tool,
                &self.selected_object,
                &self.map_name,
                self.zoom,
                self.ambient_light,
            );
        }
    }
}
