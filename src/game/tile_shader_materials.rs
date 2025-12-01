use macroquad::miniquad;
use macroquad::prelude::*;
use std::sync::OnceLock;

static GLOW_WAVE_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn get_glow_wave_material() -> &'static Material {
    GLOW_WAVE_MATERIAL.get_or_init(|| {
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
        uniform float time;
        uniform float waveBase;
        uniform float waveAmplitude;
        uniform float waveFrequency;
        uniform int waveFunc;
        
        float computeWave(float t) {
            if (waveFunc == 0) {
                return sin(t * 6.28318);
            } else if (waveFunc == 1) {
                float fract_t = fract(t);
                return 1.0 - 2.0 * abs(fract_t - 0.5);
            } else if (waveFunc == 2) {
                return fract(t) < 0.5 ? 1.0 : -1.0;
            } else if (waveFunc == 3) {
                return fract(t);
            }
            return sin(t * 6.28318);
        }
        
        void main() {
            vec4 texColor = texture2D(Texture, uv);
            
            float t = time * waveFrequency;
            float wave = computeWave(t);
            float intensity = waveBase + waveAmplitude * wave;
            
            vec3 finalColor = texColor.rgb * intensity;
            gl_FragColor = vec4(finalColor, texColor.a) * color;
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("waveBase", UniformType::Float1),
                    UniformDesc::new("waveAmplitude", UniformType::Float1),
                    UniformDesc::new("waveFrequency", UniformType::Float1),
                    UniformDesc::new("waveFunc", UniformType::Int1),
                ],
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
