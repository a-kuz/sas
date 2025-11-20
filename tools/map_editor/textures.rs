use macroquad::prelude::*;

pub struct WallTexture {
    pub texture: Texture2D,
    pub name: String,
    pub path: String,
}

pub struct BackgroundTexture {
    pub frames: Vec<Texture2D>,
    pub name: String,
    pub base_path: String,
    pub is_animated: bool,
    pub has_wave_effect: bool,
}

fn scan_directory_recursive(dir: &str, texture_paths: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    scan_directory_recursive(path_str, texture_paths);
                }
            } else if let Some(ext) = path.extension() {
                if ext == "png" || ext == "jpg" || ext == "tga" {
                    if let Some(path_str) = path.to_str() {
                        texture_paths.push(path_str.to_string());
                    }
                }
            }
        }
    }
}

pub async fn scan_all_textures() -> Vec<String> {
    let mut texture_paths = Vec::new();
    
    scan_directory_recursive("q3-resources/textures", &mut texture_paths);
    
    texture_paths.sort();
    println!("[Textures] Found {} texture files total", texture_paths.len());
    texture_paths
}

