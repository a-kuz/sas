use macroquad::prelude::*;
use crate::game::map::Map;
use std::collections::HashMap;

pub struct TileBorderRenderer {
    border_textures: HashMap<String, Texture2D>,
}

impl TileBorderRenderer {
    pub fn new() -> Self {
        Self {
            border_textures: HashMap::new(),
        }
    }
    
    pub async fn load_border_textures(&mut self) {
        let border_paths = vec![
            ("metal_edge", "q3-resources/textures/base_trim/dark_tin2.png"),
            ("concrete_edge", "q3-resources/textures/base_wall/concrete_dark.png"),
            ("tech_edge", "q3-resources/textures/base_trim/pewter.png"),
        ];
        
        for (name, path) in border_paths {
            if let Ok(image) = load_image(path).await {
                let texture = Texture2D::from_image(&image);
                texture.set_filter(FilterMode::Linear);
                self.border_textures.insert(name.to_string(), texture);
                println!("[Borders] ✓ Loaded border texture: {}", name);
            }
        }
    }
    
    pub fn render_tile_borders(
        &self,
        map: &Map,
        x: i32,
        y: i32,
        screen_x: f32,
        screen_y: f32,
        tile_width: f32,
        tile_height: f32,
    ) {
        let tile = &map.tiles[x as usize][y as usize];
        
        if !tile.solid {
            return;
        }
        
        let has_left = x > 0 && map.tiles[(x - 1) as usize][y as usize].solid 
            && map.tiles[(x - 1) as usize][y as usize].texture_id != tile.texture_id;
        let has_right = x < (map.width as i32 - 1) && map.tiles[(x + 1) as usize][y as usize].solid
            && map.tiles[(x + 1) as usize][y as usize].texture_id != tile.texture_id;
        let has_top = y > 0 && map.tiles[x as usize][(y - 1) as usize].solid
            && map.tiles[x as usize][(y - 1) as usize].texture_id != tile.texture_id;
        let has_bottom = y < (map.height as i32 - 1) && map.tiles[x as usize][(y + 1) as usize].solid
            && map.tiles[x as usize][(y + 1) as usize].texture_id != tile.texture_id;
        
        let border_width = 3.0;
        let border_color = self.get_border_color(tile.texture_id);
        
        if has_left {
            draw_rectangle(screen_x, screen_y, border_width, tile_height, border_color);
        }
        if has_right {
            draw_rectangle(screen_x + tile_width - border_width, screen_y, border_width, tile_height, border_color);
        }
        if has_top {
            draw_rectangle(screen_x, screen_y, tile_width, border_width, border_color);
        }
        if has_bottom {
            draw_rectangle(screen_x, screen_y + tile_height - border_width, tile_width, border_width, border_color);
        }
    }
    
    fn get_border_color(&self, texture_id: u16) -> Color {
        match texture_id % 10 {
            0 | 1 => Color::from_rgba(60, 65, 70, 180),
            2 | 3 => Color::from_rgba(80, 70, 60, 180),
            4 | 5 => Color::from_rgba(50, 60, 75, 180),
            6 | 7 => Color::from_rgba(70, 75, 80, 180),
            _ => Color::from_rgba(65, 70, 75, 180),
        }
    }
}














