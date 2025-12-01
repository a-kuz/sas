use crate::game::file_loader;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct SkinMapping {
    pub mesh_textures: HashMap<String, String>,
}

impl SkinMapping {
    pub async fn load(model_name: &str, skin_name: &str, part: &str) -> Result<Self, String> {
        let path = format!(
            "q3-resources/models/players/{}/{}_{}.skin",
            model_name, part, skin_name
        );
        let content = file_loader::load_file_string(&path).await?;

        let mut mesh_textures = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let mesh_name = parts[0].trim().to_string();
                let mut texture_path = parts[1].trim().to_string();
                if !texture_path.is_empty() {
                    if !texture_path.starts_with("q3-resources/") {
                        texture_path = format!("q3-resources/{}", texture_path);
                    }
                    mesh_textures.insert(mesh_name, texture_path);
                }
            }
        }

        Ok(SkinMapping { mesh_textures })
    }
}

pub async fn load_texture_file(path: &str) -> Option<Texture2D> {
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
        println!("[Texture] Attempting to load: {}", candidate);
        if let Ok(image) = load_image(&candidate).await {
            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Linear);
            println!(
                "[Texture] ✓ Successfully loaded {} ({}x{})",
                candidate, image.width, image.height
            );
            return Some(texture);
        }
    }
    println!("[Texture] ✗ All attempts failed for {}", path);
    None
}
