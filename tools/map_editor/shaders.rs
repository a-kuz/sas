use macroquad::miniquad;
use macroquad::prelude::*;
use std::sync::OnceLock;

static BG_ADDITIVE_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn get_bg_additive_material() -> &'static Material {
    BG_ADDITIVE_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        
        varying lowp vec2 uv;
        varying lowp vec4 color;
        
        uniform mat4 Model;
        uniform mat4 Projection;
        
        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision mediump float;
        
        varying lowp vec4 color;
        varying lowp vec2 uv;
        
        uniform sampler2D Texture;
        
        void main() {
            vec4 texColor = texture2D(Texture, uv);
            vec3 finalColor = texColor.rgb * texColor.a * color.rgb * color.a;
            gl_FragColor = vec4(finalColor, 1.0);
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::One,
                        miniquad::BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}

pub fn parse_shader_file(path: &str) -> Vec<(String, Vec<String>, bool, bool)> {
    let mut results = Vec::new();

    if let Ok(content) = std::fs::read_to_string(path) {
        let mut current_shader: Option<String> = None;
        let mut in_shader = false;
        let mut brace_count = 0;
        let mut textures = Vec::new();
        let mut is_additive = false;
        let mut has_wave = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }

            if !in_shader && !trimmed.starts_with("{") && !trimmed.contains("//") {
                current_shader = Some(trimmed.to_string());
                textures.clear();
                is_additive = false;
                has_wave = false;
                in_shader = true;
                continue;
            }

            if trimmed.contains("{") {
                brace_count += trimmed.matches("{").count();
            }

            if trimmed.contains("}") {
                brace_count -= trimmed.matches("}").count();
                if brace_count == 0 && in_shader {
                    if let Some(shader_name) = current_shader.take() {
                        if !textures.is_empty() {
                            results.push((shader_name, textures.clone(), is_additive, has_wave));
                        }
                    }
                    in_shader = false;
                    textures.clear();
                }
            }

            if in_shader {
                if trimmed.starts_with("map ")
                    || trimmed.starts_with("clampmap ")
                    || trimmed.starts_with("animmap ")
                {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();

                    if parts[0] == "animmap" && parts.len() > 2 {
                        for part in &parts[2..] {
                            textures.push(part.replace(".tga", ".png"));
                        }
                    } else if parts.len() > 1 {
                        let tex_path = parts[1];
                        if !tex_path.starts_with("$") && !tex_path.starts_with("*") {
                            textures.push(tex_path.replace(".tga", ".png"));
                        }
                    }
                }

                if trimmed.contains("blendFunc ADD") || trimmed.contains("blendFunc GL_ONE GL_ONE")
                {
                    is_additive = true;
                }

                if trimmed.contains("wave ")
                    || trimmed.contains("tcMod turb")
                    || trimmed.contains("tcMod scroll")
                {
                    has_wave = true;
                }
            }
        }
    }

    results
}
