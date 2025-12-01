use macroquad::prelude::*;
use std::collections::HashMap;

pub struct TileTextureCache {
    textures: HashMap<u8, Texture2D>,
}

impl TileTextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub async fn load_default_textures(&mut self) {
        let texture_paths = vec![
            (1, "q3-resources/textures/base_wall/basewall01.png"),
            (2, "q3-resources/textures/base_wall/basewall02.png"),
            (3, "q3-resources/textures/base_floor/metalbridge02.png"),
            (4, "q3-resources/textures/base_wall/atechengine_a.png"),
            (5, "q3-resources/textures/base_wall/atech1_a.png"),
            (6, "q3-resources/textures/base_wall/atech1_b.png"),
            (
                7,
                "q3-resources/textures/gothic_floor/metalfloor_wall_10a.png",
            ),
            (8, "q3-resources/textures/base_wall/atech2_c.png"),
            (9, "q3-resources/textures/gothic_block/blocks15c.png"),
        ];

        for (id, path) in texture_paths {
            if let Some(texture) = load_texture_file(path).await {
                println!("[Tiles] ✓ Loaded texture {}: {}", id, path);
                texture.set_filter(FilterMode::Linear);
                self.textures.insert(id, texture);
            } else {
                println!("[Tiles] ✗ Failed to load texture {}: {}", id, path);
            }
        }
    }
}

async fn load_texture_file(path: &str) -> Option<Texture2D> {
    let mut candidates: Vec<String> = Vec::new();
    let path_lower = path.to_lowercase();
    if let Some(dot) = path_lower.rfind('.') {
        let base = &path[..dot];
        let ext = &path_lower[dot + 1..];
        candidates.push(path.to_string());
        if ext != "png" {
            candidates.push(format!("{}.png", base));
        }
        if ext != "tga" {
            candidates.push(format!("{}.tga", base));
        }
        if ext != "jpg" {
            candidates.push(format!("{}.jpg", base));
        }
        if ext != "jpeg" {
            candidates.push(format!("{}.jpeg", base));
        }
    } else {
        candidates.push(format!("{}.png", path));
        candidates.push(format!("{}.tga", path));
        candidates.push(format!("{}.jpg", path));
        candidates.push(format!("{}.jpeg", path));
    }

    for candidate in candidates {
        if let Ok(image) = load_image(&candidate).await {
            let texture = Texture2D::from_image(&image);
            return Some(texture);
        }
    }

    None
}
