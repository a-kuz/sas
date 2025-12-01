use macroquad::prelude::*;
use std::sync::OnceLock;

static AWARD_GLOW_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn get_award_shader_material() -> &'static Material {
    AWARD_GLOW_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;

        varying lowp vec2 uv;
        varying lowp vec4 color;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision highp float;

        varying lowp vec4 color;
        varying lowp vec2 uv;

        uniform sampler2D Texture;
        uniform float time;
        uniform float scale;

        void main() {
            vec4 texColor = texture2D(Texture, uv);
            
            float pulse = 0.85 + 0.15 * sin(time * 3.0);
            
            vec2 center = vec2(0.5, 0.5);
            float dist = length(uv - center);
            
            float glow = 1.0 / (1.0 + dist * 3.0);
            glow = pow(glow, 2.0);
            
            float wave = sin(time * 4.0 + dist * 10.0) * 0.5 + 0.5;
            float shimmer = 0.1 * wave * glow;
            
            vec3 glowColor = vec3(1.0, 0.9, 0.5);
            
            vec3 finalColor = texColor.rgb * pulse;
            finalColor += glowColor * shimmer * texColor.a;
            
            float outerGlow = smoothstep(0.7, 0.3, dist) * 0.3 * pulse;
            finalColor += glowColor * outerGlow;
            
            float alpha = texColor.a * color.a * (1.0 + outerGlow);
            
            gl_FragColor = vec4(finalColor * color.rgb, alpha);
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("scale", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(macroquad::miniquad::BlendState::new(
                        macroquad::miniquad::Equation::Add,
                        macroquad::miniquad::BlendFactor::Value(
                            macroquad::miniquad::BlendValue::SourceAlpha,
                        ),
                        macroquad::miniquad::BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap()
    })
}
