use crate::game::q3_shader_parser::Q3ShaderParser;
use crate::game::tile_shader::*;
use macroquad::miniquad;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

static MODEL_DIFFUSE_SPECULAR_MATERIAL: OnceLock<Material> = OnceLock::new();
static MODEL_ALPHA_TEST_MATERIAL: OnceLock<Material> = OnceLock::new();
static MODEL_ENVMAP_MATERIAL: OnceLock<Material> = OnceLock::new();
static MODEL_FIRE_MATERIAL: OnceLock<Material> = OnceLock::new();

fn get_model_diffuse_specular_material() -> &'static Material {
    MODEL_DIFFUSE_SPECULAR_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        attribute vec4 normal;

        varying lowp vec2 uv;
        varying lowp vec4 color;
        varying lowp vec3 vNormal;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            color = color0 / 255.0;
            uv = texcoord;
            vNormal = normalize(normal.xyz);
        }"#;

        let fragment_shader = r#"#version 100
        precision lowp float;

        varying lowp vec4 color;
        varying lowp vec2 uv;
        varying lowp vec3 vNormal;

        uniform sampler2D Texture;

        void main() {
            vec4 baseColor = texture2D(Texture, uv);
            
            float NdotL = max(dot(vNormal, vec3(0.0, 0.0, 1.0)), 0.0);
            vec3 diffuse = vec3(1.0) * NdotL;
            
            vec3 viewDir = normalize(vec3(0.5, 0.5, 1.0));
            float specular = max(dot(vNormal, viewDir), 0.0);
            specular = pow(specular, 16.0);
            
            vec3 final = baseColor.rgb * diffuse + baseColor.rgb * specular * baseColor.a;
            
            gl_FragColor = vec4(final, 1.0) * color;
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
                        miniquad::BlendFactor::Value(miniquad::BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}

fn get_model_alpha_test_material() -> &'static Material {
    MODEL_ALPHA_TEST_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        attribute vec4 normal;

        varying lowp vec2 uv;
        varying lowp vec4 color;
        varying lowp vec3 vNormal;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            color = color0 / 255.0;
            uv = texcoord;
            vNormal = normalize(normal.xyz);
        }"#;

        let fragment_shader = r#"#version 100
        precision lowp float;

        varying lowp vec4 color;
        varying lowp vec2 uv;
        varying lowp vec3 vNormal;

        uniform sampler2D Texture;

        void main() {
            vec4 texColor = texture2D(Texture, uv);
            
            if (texColor.a < 0.5) {
                discard;
            }
            
            float lightDir = max(dot(vNormal, vec3(0.0, 0.0, 1.0)), 0.0);
            vec3 diffuse = texColor.rgb * (0.6 + 0.4 * lightDir);
            
            gl_FragColor = vec4(diffuse, 1.0) * color;
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    cull_face: miniquad::CullFace::Nothing,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}

fn get_model_envmap_material() -> &'static Material {
    MODEL_ENVMAP_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        attribute vec4 normal;

        varying lowp vec2 uv;
        varying lowp vec4 color;
        varying lowp vec3 vNormal;
        varying lowp vec2 envUV;

        uniform mat4 Model;
        uniform mat4 Projection;
        uniform float time;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            color = color0 / 255.0;
            uv = texcoord;
            vNormal = normalize(normal.xyz);
            
            float angle = time * 6.1;
            float s = sin(angle);
            float c = cos(angle);
            vec2 rotated = vec2(
                vNormal.x * c - vNormal.y * s,
                vNormal.x * s + vNormal.y * c
            );
            envUV = rotated * 0.5 + 0.5;
        }"#;

        let fragment_shader = r#"#version 100
        precision lowp float;

        varying lowp vec4 color;
        varying lowp vec2 uv;
        varying lowp vec2 envUV;

        uniform sampler2D Texture;
        uniform sampler2D _env_map;

        void main() {
            vec4 envColor = texture2D(_env_map, envUV);
            vec4 baseColor = texture2D(Texture, uv);
            
            vec3 final = mix(envColor.rgb, baseColor.rgb, baseColor.a);
            
            gl_FragColor = vec4(final, 1.0) * color;
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("time", UniformType::Float1)],
                textures: vec!["_env_map".to_string()],
                ..Default::default()
            },
        )
        .unwrap()
    })
}

fn get_model_fire_material() -> &'static Material {
    MODEL_FIRE_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        attribute vec4 normal;

        varying lowp vec2 uv;
        varying lowp vec4 color;
        varying lowp vec3 vNormal;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            color = color0 / 255.0;
            uv = texcoord;
            vNormal = normalize(normal.xyz);
        }"#;

        let fragment_shader = r#"#version 100
        precision lowp float;

        varying lowp vec4 color;
        varying lowp vec2 uv;
        varying lowp vec3 vNormal;

        uniform sampler2D Texture;
        uniform sampler2D _fire_tex;
        uniform float time;

        void main() {
            vec4 baseColor = texture2D(Texture, uv);
            
            vec2 fireUV = uv + vec2(0.1 * time, time);
            vec4 fireColor = texture2D(_fire_tex, fireUV);
            
            float lightDir = max(dot(vNormal, vec3(0.0, 0.0, 1.0)), 0.0);
            vec3 diffuse = baseColor.rgb * (0.6 + 0.4 * lightDir);
            
            vec3 final = diffuse + fireColor.rgb * 0.8;
            
            gl_FragColor = vec4(final, 1.0) * color;
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("time", UniformType::Float1)],
                textures: vec!["_fire_tex".to_string()],
                ..Default::default()
            },
        )
        .unwrap()
    })
}

pub fn get_fire_shader_material() -> &'static Material {
    get_model_fire_material()
}

pub fn get_envmap_shader_material() -> &'static Material {
    get_model_envmap_material()
}

pub fn get_alpha_test_shader_material() -> &'static Material {
    get_model_alpha_test_material()
}

pub fn get_diffuse_specular_material() -> &'static Material {
    get_model_diffuse_specular_material()
}

pub struct ModelShaderManager {
    shader_parser: Option<Q3ShaderParser>,
    loaded_textures: HashMap<String, Texture2D>,
}

impl ModelShaderManager {
    pub fn new() -> Self {
        Self {
            shader_parser: None,
            loaded_textures: HashMap::new(),
        }
    }

    pub fn set_shader_parser(&mut self, parser: Q3ShaderParser) {
        self.shader_parser = Some(parser);
    }

    pub async fn load_texture(&mut self, path: &str) -> Option<Texture2D> {
        if let Some(tex) = self.loaded_textures.get(path) {
            return Some(tex.clone());
        }

        let tex_path = path.replace(".tga", ".png");
        if let Ok(image) = load_image(&tex_path).await {
            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Linear);
            self.loaded_textures
                .insert(path.to_string(), texture.clone());
            Some(texture)
        } else {
            None
        }
    }

    pub fn get_shader_for_texture(&self, texture_path: &str) -> Option<&TileShader> {
        if let Some(ref parser) = self.shader_parser {
            let shader_name = texture_path
                .replace("q3-resources/", "")
                .replace(".png", "")
                .replace(".tga", "");
            parser.get_shader(&shader_name)
        } else {
            None
        }
    }

    pub fn should_use_shader(&self, texture_path: &str) -> bool {
        self.get_shader_for_texture(texture_path).is_some()
    }
}

pub enum ModelRenderMode {
    Standard,
    DiffuseSpecular,
    AlphaTest,
    EnvMap,
}

pub fn get_render_mode_for_shader(shader: &TileShader) -> ModelRenderMode {
    if shader.cull_disable {
        return ModelRenderMode::AlphaTest;
    }

    if shader.stages.len() >= 2 {
        let first_stage = &shader.stages[0];
        if first_stage.use_white_image && first_stage.rgb_gen == RgbGen::LightingDiffuse {
            if shader.stages.len() > 1 {
                let second_stage = &shader.stages[1];
                if second_stage.alpha_gen == AlphaGen::LightingSpecular {
                    return ModelRenderMode::DiffuseSpecular;
                }
            }
        }

        let has_envmap = shader
            .stages
            .iter()
            .any(|s| s.texture_path.contains("envmap") || s.rotate_speed.abs() > 100.0);
        if has_envmap {
            return ModelRenderMode::EnvMap;
        }
    }

    ModelRenderMode::Standard
}

pub fn apply_model_shader(
    render_mode: ModelRenderMode,
    _base_texture: Option<&Texture2D>,
    second_texture: Option<&Texture2D>,
    time: f32,
) {
    match render_mode {
        ModelRenderMode::DiffuseSpecular => {
            let material = get_model_diffuse_specular_material();
            gl_use_material(material);
            if let Some(tex2) = second_texture {
                material.set_uniform("hasTexture2", 1.0f32);
                material.set_texture("Texture2", tex2.clone());
            } else {
                material.set_uniform("hasTexture2", 0.0f32);
            }
        }
        ModelRenderMode::AlphaTest => {
            let material = get_model_alpha_test_material();
            gl_use_material(material);
        }
        ModelRenderMode::EnvMap => {
            let material = get_model_envmap_material();
            gl_use_material(material);
            material.set_uniform("time", time);
            if let Some(env_tex) = second_texture {
                material.set_texture("EnvMap", env_tex.clone());
            }
        }
        ModelRenderMode::Standard => {}
    }
}
