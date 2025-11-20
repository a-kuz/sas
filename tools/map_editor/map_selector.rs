use macroquad::prelude::*;
use std::fs;

use super::map;
use super::state::{TILE_WIDTH, TILE_HEIGHT};

struct MapCard {
    name: String,
    preview: Option<RenderTarget>,
    bounds: (f32, f32, f32, f32),
}

pub struct MapSelector {
    cards: Vec<MapCard>,
    create_button_bounds: (f32, f32, f32, f32),
    scroll_offset: f32,
}

impl MapSelector {
    pub async fn new() -> Self {
        let mut cards = Vec::new();
        
        if let Ok(entries) = fs::read_dir("maps") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if !name.ends_with("_navgraph") {
                            cards.push(MapCard {
                                name: name.to_string(),
                                preview: None,
                                bounds: (0.0, 0.0, 0.0, 0.0),
                            });
                        }
                    }
                }
            }
        }
        
        cards.sort_by(|a, b| a.name.cmp(&b.name));
        
        MapSelector {
            cards,
            create_button_bounds: (0.0, 0.0, 0.0, 0.0),
            scroll_offset: 0.0,
        }
    }
    
    pub async fn ensure_previews_loaded(&mut self) {
        for card in &mut self.cards {
            if card.preview.is_none() {
                card.preview = Self::generate_preview(&card.name).await;
            }
        }
    }
    
    async fn generate_preview(map_name: &str) -> Option<RenderTarget> {
        println!("[Preview] Generating preview for map: {}", map_name);
        
        let map = match map::Map::load_from_file(map_name) {
            Ok(m) => {
                println!("[Preview] Map loaded: {}x{}, {} solid tiles", m.width, m.height, 
                    m.tiles.iter().flatten().filter(|t| t.solid).count());
                m
            },
            Err(e) => {
                println!("[Preview] Failed to load map: {}", e);
                return None;
            }
        };
        
        let preview_width = 280;
        let preview_height = 200;
        
        println!("[Preview] Creating render target {}x{}", preview_width, preview_height);
        let render_target = render_target(preview_width * 2, preview_height * 2);
        render_target.texture.set_filter(FilterMode::Linear);
        
        let map_width_pixels = map.width as f32 * TILE_WIDTH;
        let map_height_pixels = map.height as f32 * TILE_HEIGHT;
        
        let scale_x = preview_width as f32 / map_width_pixels;
        let scale_y = preview_height as f32 / map_height_pixels;
        let scale = scale_x.min(scale_y) * 0.85;
        
        println!("[Preview] Map size: {}x{} pixels, scale: {}", map_width_pixels, map_height_pixels, scale);
        
        let scaled_width = map_width_pixels * scale;
        let scaled_height = map_height_pixels * scale;
        
        let offset_x = (preview_width as f32 - scaled_width) / 2.0;
        let offset_y = (preview_height as f32 - scaled_height) / 2.0;
        
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            zoom: vec2(2.0 / preview_width as f32, -2.0 / preview_height as f32),
            target: vec2(preview_width as f32 / 2.0, preview_height as f32 / 2.0),
            offset: vec2(0.0, 0.0),
            rotation: 0.0,
            viewport: None,
        });
        
        clear_background(Color::from_rgba(25, 28, 32, 255));
        
        let mut tiles_drawn = 0;
        for x in 0..map.width {
            for y in 0..map.height {
                let tile = &map.tiles[x][y];
                if tile.solid {
                    let px = offset_x + x as f32 * TILE_WIDTH * scale;
                    let py = offset_y + y as f32 * TILE_HEIGHT * scale;
                    let w = TILE_WIDTH * scale;
                    let h = TILE_HEIGHT * scale;
                    
                    let color = match tile.texture_id {
                        1 => Color::from_rgba(95, 105, 120, 255),
                        2 => Color::from_rgba(120, 95, 80, 255),
                        3 => Color::from_rgba(85, 95, 110, 255),
                        4 => Color::from_rgba(75, 85, 95, 255),
                        5 => Color::from_rgba(100, 90, 80, 255),
                        6 => Color::from_rgba(90, 100, 110, 255),
                        7 => Color::from_rgba(105, 95, 85, 255),
                        8 => Color::from_rgba(80, 90, 100, 255),
                        9 => Color::from_rgba(110, 100, 90, 255),
                        _ => Color::from_rgba(70, 75, 80, 255),
                    };
                    draw_rectangle(px, py, w, h, color);
                    tiles_drawn += 1;
                    
                    if w > 1.5 && h > 1.5 {
                        draw_rectangle_lines(px, py, w, h, 0.5, Color::from_rgba(40, 45, 50, 150));
                    }
                }
            }
        }
        
        println!("[Preview] Drew {} tiles", tiles_drawn);
        
        for sp in &map.spawn_points {
            let px = offset_x + sp.x * scale;
            let py = offset_y + sp.y * scale;
            let radius = (5.0 * scale).max(2.0);
            draw_circle(px, py, radius, Color::from_rgba(100, 255, 100, 255));
            draw_circle(px, py, radius * 0.6, Color::from_rgba(50, 200, 50, 255));
        }
        
        for item in &map.items {
            let px = offset_x + item.x * scale;
            let py = offset_y + item.y * scale;
            let radius = (4.0 * scale).max(1.5);
            let color = match item.item_type {
                map::ItemType::Health25 | map::ItemType::Health50 | map::ItemType::Health100 => 
                    Color::from_rgba(255, 80, 80, 255),
                map::ItemType::Armor50 | map::ItemType::Armor100 => 
                    Color::from_rgba(255, 220, 80, 255),
                map::ItemType::Quad | map::ItemType::Regen | map::ItemType::Battle | 
                map::ItemType::Flight | map::ItemType::Haste | map::ItemType::Invis => 
                    Color::from_rgba(200, 100, 255, 255),
                _ => Color::from_rgba(100, 150, 255, 255),
            };
            draw_circle(px, py, radius, color);
        }
        
        for light in &map.lights {
            let px = offset_x + light.x * scale;
            let py = offset_y + light.y * scale;
            let radius = (3.0 * scale).max(1.0);
            draw_circle(px, py, radius, Color::from_rgba(light.r, light.g, light.b, 220));
        }
        
        set_default_camera();
        
        println!("[Preview] Preview generated successfully");
        Some(render_target)
    }
    
    pub fn handle_input_and_render(&mut self) -> Option<String> {
        clear_background(Color::from_rgba(18, 20, 24, 255));
        
        let title_height = 80.0;
        let padding = 20.0;
        let card_width = 300.0;
        let card_height = 240.0;
        let cards_per_row = ((screen_width() - padding * 2.0) / (card_width + padding)).floor() as usize;
        let cards_per_row = cards_per_row.max(1);
        
        draw_text("SAS Map Editor", padding, 50.0, 40.0, WHITE);
        
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            self.scroll_offset += scroll * 20.0;
            self.scroll_offset = self.scroll_offset.max(0.0);
        }
        
        let mut selected_map: Option<String> = None;
        let y_pos = title_height - self.scroll_offset;
        
        let create_btn_w = 300.0;
        let create_btn_h = 240.0;
        let create_btn_x = padding;
        let create_btn_y = y_pos;
        
        let (mx, my) = mouse_position();
        let is_hovering_create = mx >= create_btn_x && mx <= create_btn_x + create_btn_w &&
                                  my >= create_btn_y && my <= create_btn_y + create_btn_h;
        
        let create_color = if is_hovering_create {
            Color::from_rgba(50, 120, 200, 255)
        } else {
            Color::from_rgba(35, 90, 160, 255)
        };
        
        draw_rectangle(create_btn_x, create_btn_y, create_btn_w, create_btn_h, create_color);
        draw_rectangle_lines(create_btn_x, create_btn_y, create_btn_w, create_btn_h, 2.0, 
            if is_hovering_create { WHITE } else { Color::from_rgba(180, 180, 180, 255) });
        
        let plus_size = 80.0;
        let plus_x = create_btn_x + create_btn_w / 2.0;
        let plus_y = create_btn_y + create_btn_h / 2.0 - 20.0;
        let plus_color = if is_hovering_create { WHITE } else { Color::from_rgba(200, 200, 200, 255) };
        
        draw_rectangle(plus_x - plus_size / 2.0, plus_y - 4.0, plus_size, 8.0, plus_color);
        draw_rectangle(plus_x - 4.0, plus_y - plus_size / 2.0, 8.0, plus_size, plus_color);
        
        let text = "Create New Map";
        let text_dims = measure_text(text, None, 20, 1.0);
        draw_text(text, plus_x - text_dims.width / 2.0, create_btn_y + create_btn_h - 30.0, 20.0, plus_color);
        
        self.create_button_bounds = (create_btn_x, create_btn_y, create_btn_w, create_btn_h);
        
        if is_hovering_create && is_mouse_button_pressed(MouseButton::Left) {
            selected_map = Some("new_map".to_string());
        }
        
        let mut current_idx = 1;
        for card in &mut self.cards {
            let row = current_idx / cards_per_row;
            let col = current_idx % cards_per_row;
            
            let x = padding + col as f32 * (card_width + padding);
            let y = y_pos + row as f32 * (card_height + padding);
            
            card.bounds = (x, y, card_width, card_height);
            
            if y + card_height < 0.0 || y > screen_height() {
                current_idx += 1;
                continue;
            }
            
            let is_hovering = mx >= x && mx <= x + card_width && my >= y && my <= y + card_height;
            
            let bg_color = if is_hovering {
                Color::from_rgba(45, 50, 60, 255)
            } else {
                Color::from_rgba(30, 35, 42, 255)
            };
            
            draw_rectangle(x, y, card_width, card_height, bg_color);
            draw_rectangle_lines(x, y, card_width, card_height, 2.0, 
                if is_hovering { Color::from_rgba(100, 150, 220, 255) } else { Color::from_rgba(60, 65, 75, 255) });
            
            if let Some(preview) = &card.preview {
                draw_texture_ex(
                    &preview.texture,
                    x + 10.0,
                    y + 10.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(280.0, 200.0)),
                        flip_y: true,
                        ..Default::default()
                    },
                );
            } else {
                let no_preview_text = "No preview";
                let dims = measure_text(no_preview_text, None, 18, 1.0);
                draw_text(no_preview_text, 
                    x + card_width / 2.0 - dims.width / 2.0, 
                    y + card_height / 2.0, 
                    18.0, 
                    Color::from_rgba(100, 100, 100, 255));
            }
            
            let text_y = y + card_height - 15.0;
            let text_dims = measure_text(&card.name, None, 18, 1.0);
            draw_text(&card.name, x + card_width / 2.0 - text_dims.width / 2.0, text_y, 18.0, WHITE);
            
            if is_hovering && is_mouse_button_pressed(MouseButton::Left) {
                selected_map = Some(card.name.clone());
            }
            
            current_idx += 1;
        }
        
        draw_text("Press ESC to exit", padding, screen_height() - 20.0, 16.0, Color::from_rgba(150, 150, 150, 255));
        
        selected_map
    }
}

