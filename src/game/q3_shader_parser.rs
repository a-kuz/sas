use super::tile_shader::*;
use std::collections::HashMap;

pub struct Q3ShaderParser {
    shaders: HashMap<String, TileShader>,
}

impl Q3ShaderParser {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }
    
    pub fn parse_shader_file(&mut self, path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read shader file: {}", e))?;
        
        let mut current_shader: Option<TileShader> = None;
        let mut current_stage: Option<ShaderStage> = None;
        let mut brace_level = 0;
        let mut shader_name = String::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            if trimmed.contains("{") {
                brace_level += trimmed.matches("{").count();
                
                if brace_level == 1 && current_shader.is_none() {
                    current_shader = Some(TileShader {
                        name: shader_name.clone(),
                        ..Default::default()
                    });
                } else if brace_level == 2 {
                    current_stage = Some(ShaderStage::default());
                }
            }
            
            if trimmed.contains("}") {
                let close_count = trimmed.matches("}").count();
                
                if brace_level == 2 && close_count > 0 {
                    if let (Some(stage), Some(ref mut shader)) = (current_stage.take(), &mut current_shader) {
                        shader.stages.push(stage);
                    }
                }
                
                brace_level -= close_count;
                
                if brace_level == 0 {
                    if let Some(shader) = current_shader.take() {
                        self.shaders.insert(shader.name.clone(), shader);
                    }
                }
            }
            
            if !trimmed.starts_with("{") && !trimmed.starts_with("}") && brace_level == 0 {
                shader_name = trimmed.to_string();
            }
            
            if brace_level == 1 && current_shader.is_some() {
                if trimmed.starts_with("q3map_surfacelight ") {
                    if let Some(value) = trimmed.split_whitespace().nth(1) {
                        if let Ok(light) = value.parse::<f32>() {
                            if let Some(ref mut shader) = current_shader {
                                shader.surface_light = light;
                            }
                        }
                    }
                }
            }
            
            if brace_level == 2 && current_stage.is_some() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                
                if parts.is_empty() {
                    continue;
                }
                
                match parts[0] {
                    "map" if parts.len() > 1 => {
                        let tex_path = parts[1].replace(".tga", ".png");
                        if !tex_path.starts_with("$") && !tex_path.starts_with("*") {
                            if let Some(ref mut stage) = current_stage {
                                if stage.texture_path.is_empty() {
                                    stage.texture_path = if tex_path.starts_with("textures/") {
                                        format!("q3-resources/{}", tex_path)
                                    } else {
                                        tex_path
                                    };
                                }
                            }
                        }
                    }
                    "clampmap" if parts.len() > 1 => {
                        let tex_path = parts[1].replace(".tga", ".png");
                        if let Some(ref mut stage) = current_stage {
                            stage.texture_path = if tex_path.starts_with("textures/") {
                                format!("q3-resources/{}", tex_path)
                            } else {
                                tex_path
                            };
                        }
                    }
                    "blendFunc" => {
                        if let Some(ref mut stage) = current_stage {
                            if parts.len() > 1 {
                                match parts[1].to_uppercase().as_str() {
                                    "ADD" => stage.blend_mode = BlendMode::Add,
                                    "BLEND" => stage.blend_mode = BlendMode::Blend,
                                    "FILTER" => stage.blend_mode = BlendMode::Filter,
                                    "GL_ONE" if parts.get(2).map_or(false, |&p| p == "GL_ONE") => {
                                        stage.blend_mode = BlendMode::Add;
                                        stage.glow = true;
                                        stage.glow_intensity = 0.8;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    "tcMod" if parts.len() > 2 => {
                        if let Some(ref mut stage) = current_stage {
                            match parts[1] {
                                "scroll" if parts.len() > 3 => {
                                    if let (Ok(x), Ok(y)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                                        stage.scroll_x = x;
                                        stage.scroll_y = y;
                                    }
                                }
                                "scale" if parts.len() > 3 => {
                                    if let (Ok(x), Ok(y)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                                        stage.scale_x = x;
                                        stage.scale_y = y;
                                    }
                                }
                                "rotate" if parts.len() > 2 => {
                                    if let Ok(speed) = parts[2].parse::<f32>() {
                                        stage.rotate_speed = speed;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_shader(&self, name: &str) -> Option<&TileShader> {
        self.shaders.get(name)
    }
    
    pub fn get_all_shaders(&self) -> &HashMap<String, TileShader> {
        &self.shaders
    }
    
    pub fn load_all_shader_files(&mut self) {
        let shader_files = vec![
            "q3-resources/scripts/base_wall.shader",
            "q3-resources/scripts/base_floor.shader",
            "q3-resources/scripts/base_trim.shader",
            "q3-resources/scripts/base_light.shader",
            "q3-resources/scripts/gothic_wall.shader",
            "q3-resources/scripts/gothic_floor.shader",
            "q3-resources/scripts/gothic_trim.shader",
            "q3-resources/scripts/gothic_light.shader",
            "q3-resources/scripts/sfx.shader",
        ];
        
        for file in shader_files {
            if let Err(e) = self.parse_shader_file(file) {
                println!("[Shader Parser] Failed to parse {}: {}", file, e);
            } else {
                println!("[Shader Parser] ✓ Parsed {}", file);
            }
        }
        
        println!("[Shader Parser] Loaded {} shaders total", self.shaders.len());
    }
}

