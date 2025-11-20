pub async fn load_file_bytes(path: &str) -> Result<Vec<u8>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use macroquad::prelude::*;
        load_file(path).await
            .map_err(|e| format!("Failed to load file {}: {:?}", path, e))
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read(path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))
    }
}

pub async fn load_file_string(path: &str) -> Result<String, String> {
    let bytes = load_file_bytes(path).await?;
    String::from_utf8(bytes)
        .map_err(|e| format!("Failed to decode UTF-8 from {}: {}", path, e))
}

pub fn read_dir(path: &str) -> Result<Vec<String>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        if path.contains("models/players/") {
            if let Some(model_name) = path.strip_prefix("q3-resources/models/players/") {
                return Ok(get_model_skin_files(model_name));
            }
        }
        Ok(Vec::new())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory {}: {}", path, e))?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name().to_str().map(|s| s.to_string())
                })
            })
            .collect::<Vec<_>>())
    }
}

#[cfg(target_arch = "wasm32")]
fn get_model_skin_files(_model_name: &str) -> Vec<String> {
    vec![
        "head_default.skin".to_string(),
        "upper_default.skin".to_string(),
        "lower_default.skin".to_string(),
        "head_red.skin".to_string(),
        "upper_red.skin".to_string(),
        "lower_red.skin".to_string(),
        "head_blue.skin".to_string(),
        "upper_blue.skin".to_string(),
        "lower_blue.skin".to_string(),
    ]
}

