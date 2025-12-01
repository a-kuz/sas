use super::tile_shader_materials;
use macroquad::miniquad;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static ADDITIVE_MATERIAL: OnceLock<Material> = OnceLock::new();
static FILTER_MATERIAL: OnceLock<Material> = OnceLock::new();

fn get_additive_material() -> &'static Material {
    ADDITIVE_MATERIAL.get_or_init(|| {
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
            gl_FragColor = texColor * color;
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

fn get_filter_material() -> &'static Material {
    FILTER_MATERIAL.get_or_init(|| {
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
            gl_FragColor = vec4(texColor.rgb * color.rgb, texColor.a * color.a);
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
                        miniquad::BlendFactor::Value(miniquad::BlendValue::DestinationColor),
                        miniquad::BlendFactor::Zero,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BlendMode {
    None,
    Blend,
    Add,
    Filter,
    Multiply,
    Modulate,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RgbGen {
    Identity,
    LightingDiffuse,
    Wave,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AlphaGen {
    Identity,
    LightingSpecular,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AlphaTest {
    None,
    GE128,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WaveFunc {
    Sin,
    Triangle,
    Square,
    Sawtooth,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShaderStage {
    pub texture_path: String,
    pub blend_mode: BlendMode,
    pub alpha: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotate_speed: f32,
    pub glow: bool,
    pub glow_intensity: f32,
    #[serde(default)]
    pub wave_func: Option<WaveFunc>,
    #[serde(default)]
    pub wave_base: f32,
    #[serde(default)]
    pub wave_amplitude: f32,
    #[serde(default)]
    pub wave_frequency: f32,
    #[serde(default)]
    pub rgb_gen: RgbGen,
    #[serde(default)]
    pub alpha_gen: AlphaGen,
    #[serde(default)]
    pub alpha_test: AlphaTest,
    #[serde(default)]
    pub depth_write: bool,
    #[serde(default)]
    pub use_white_image: bool,
}

impl Default for RgbGen {
    fn default() -> Self {
        RgbGen::Identity
    }
}

impl Default for AlphaGen {
    fn default() -> Self {
        AlphaGen::Identity
    }
}

impl Default for AlphaTest {
    fn default() -> Self {
        AlphaTest::None
    }
}

impl Default for ShaderStage {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            blend_mode: BlendMode::None,
            alpha: 1.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate_speed: 0.0,
            glow: false,
            glow_intensity: 0.0,
            wave_func: None,
            wave_base: 0.5,
            wave_amplitude: 0.5,
            wave_frequency: 1.0,
            rgb_gen: RgbGen::Identity,
            alpha_gen: AlphaGen::Identity,
            alpha_test: AlphaTest::None,
            depth_write: false,
            use_white_image: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileShader {
    pub name: String,
    pub base_texture: String,
    pub stages: Vec<ShaderStage>,
    pub surface_light: f32,
    #[serde(default)]
    pub cull_disable: bool,
}

impl Default for TileShader {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_texture: String::new(),
            stages: Vec::new(),
            surface_light: 0.0,
            cull_disable: false,
        }
    }
}

pub struct TileShaderRenderer {
    loaded_textures: std::collections::HashMap<String, Texture2D>,
    shaders: std::collections::HashMap<String, TileShader>,
    detail_overlays: std::collections::HashMap<u16, String>,
}

impl TileShaderRenderer {
    pub fn new() -> Self {
        Self {
            loaded_textures: std::collections::HashMap::new(),
            shaders: std::collections::HashMap::new(),
            detail_overlays: std::collections::HashMap::new(),
        }
    }

    pub fn set_detail_texture(&mut self, texture_id: u16, detail_path: String) {
        self.detail_overlays.insert(texture_id, detail_path);
    }

    pub fn render_detail_overlay(
        &self,
        texture_id: u16,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        world_x: f32,
        world_y: f32,
    ) {
        if let Some(detail_path) = self.detail_overlays.get(&texture_id) {
            if let Some(detail_tex) = self.loaded_textures.get(detail_path) {
                let tex_w = detail_tex.width() * 2.0;
                let tex_h = detail_tex.height() * 2.0;

                let src_x = (world_x % tex_w) / tex_w;
                let src_y = (world_y % tex_h) / tex_h;
                let src_w = width.min(tex_w) / tex_w;
                let src_h = height.min(tex_h) / tex_h;

                gl_use_material(get_filter_material());

                draw_texture_ex(
                    detail_tex,
                    x,
                    y,
                    Color::new(1.0, 1.0, 1.0, 0.5),
                    DrawTextureParams {
                        dest_size: Some(vec2(width, height)),
                        source: Some(Rect::new(
                            src_x * detail_tex.width(),
                            src_y * detail_tex.height(),
                            src_w * detail_tex.width(),
                            src_h * detail_tex.height(),
                        )),
                        ..Default::default()
                    },
                );

                gl_use_default_material();
            }
        }
    }

    pub async fn load_texture(&mut self, path: &str) -> Option<Texture2D> {
        if let Some(tex) = self.loaded_textures.get(path) {
            return Some(tex.clone());
        }

        if let Ok(image) = load_image(path).await {
            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Linear);
            self.loaded_textures
                .insert(path.to_string(), texture.clone());
            Some(texture)
        } else {
            None
        }
    }

    pub fn add_shader(&mut self, shader: TileShader) {
        self.shaders.insert(shader.name.clone(), shader);
    }

    pub fn render_tile_with_shader(
        &self,
        shader_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        world_x: f32,
        world_y: f32,
        time: f32,
    ) {
        if let Some(shader) = self.shaders.get(shader_name) {
            if !shader.base_texture.is_empty() {
                if let Some(base_tex) = self.loaded_textures.get(&shader.base_texture) {
                    let tex_w = base_tex.width();
                    let tex_h = base_tex.height();

                    let src_x = (world_x % tex_w) / tex_w;
                    let src_y = (world_y % tex_h) / tex_h;
                    let src_w = (width.min(tex_w)) / tex_w;
                    let src_h = (height.min(tex_h)) / tex_h;

                    draw_texture_ex(
                        base_tex,
                        x,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(width, height)),
                            source: Some(Rect::new(
                                src_x * tex_w,
                                src_y * tex_h,
                                src_w * tex_w,
                                src_h * tex_h,
                            )),
                            ..Default::default()
                        },
                    );
                }
            }

            for stage in &shader.stages {
                if stage.texture_path.is_empty() {
                    continue;
                }

                if let Some(tex) = self.loaded_textures.get(&stage.texture_path) {
                    let u_offset = stage.scroll_x * time;
                    let v_offset = stage.scroll_y * time;

                    let use_wave_shader = stage.glow && stage.wave_func.is_some();

                    let tex_w = tex.width() * stage.scale_x;
                    let tex_h = tex.height() * stage.scale_y;

                    let src_x = ((world_x + u_offset) % tex_w) / tex_w;
                    let src_y = ((world_y + v_offset) % tex_h) / tex_h;

                    if use_wave_shader {
                        let material = tile_shader_materials::get_glow_wave_material();
                        gl_use_material(material);

                        static mut LAST_PRINT: f32 = 0.0;
                        unsafe {
                            if time - LAST_PRINT > 1.0 {
                                println!(
                                    "[Wave Shader] time={:.2} base={} amp={} freq={}",
                                    time,
                                    stage.wave_base,
                                    stage.wave_amplitude,
                                    stage.wave_frequency
                                );
                                LAST_PRINT = time;
                            }
                        }

                        material.set_uniform("time", time);
                        material.set_uniform("waveBase", stage.wave_base);
                        material.set_uniform("waveAmplitude", stage.wave_amplitude);
                        material.set_uniform("waveFrequency", stage.wave_frequency);

                        let wave_func_int = match stage.wave_func.as_ref().unwrap() {
                            WaveFunc::Sin => 0,
                            WaveFunc::Triangle => 1,
                            WaveFunc::Square => 2,
                            WaveFunc::Sawtooth => 3,
                        };
                        material.set_uniform("waveFunc", wave_func_int);

                        draw_texture_ex(
                            tex,
                            x,
                            y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(width, height)),
                                source: Some(Rect::new(
                                    src_x * tex.width(),
                                    src_y * tex.height(),
                                    (width / stage.scale_x).min(tex.width()),
                                    (height / stage.scale_y).min(tex.height()),
                                )),
                                ..Default::default()
                            },
                        );

                        gl_use_default_material();
                    } else {
                        match stage.blend_mode {
                            BlendMode::Add => {
                                gl_use_material(get_additive_material());
                            }
                            BlendMode::Filter => {
                                gl_use_material(get_filter_material());
                            }
                            _ => {}
                        }

                        draw_texture_ex(
                            tex,
                            x,
                            y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(width, height)),
                                source: Some(Rect::new(
                                    src_x * tex.width(),
                                    src_y * tex.height(),
                                    (width / stage.scale_x).min(tex.width()),
                                    (height / stage.scale_y).min(tex.height()),
                                )),
                                ..Default::default()
                            },
                        );

                        gl_use_default_material();
                    }
                }
            }
        }
    }
}
