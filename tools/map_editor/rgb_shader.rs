use macroquad::prelude::*;
use std::sync::OnceLock;

static RGB_SHIFT_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn get_rgb_shift_material() -> &'static Material {
    RGB_SHIFT_MATERIAL.get_or_init(|| {
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
        
        void main() {
            vec2 center = vec2(0.5, 0.5);
            vec2 toCenter = center - uv;
            float dist = length(toCenter);
            vec2 direction = normalize(toCenter);
            
            float aberration = 0.003 + dist * 0.008;
            
            vec2 uvR = uv - direction * aberration * 1.0;
            vec2 uvG = uv;
            vec2 uvB = uv + direction * aberration * 1.0;
            
            float r = texture2D(Texture, uvR).r;
            float g = texture2D(Texture, uvG).g;
            float b = texture2D(Texture, uvB).b;
            float a = texture2D(Texture, uvG).a;
            
            float edge = smoothstep(0.0, 0.2, dist);
            float glow = (1.0 - dist) * 0.3;
            
            vec3 rgb = vec3(r, g, b);
            rgb += vec3(0.1, 0.3, 0.5) * edge * (0.5 + 0.5 * sin(time * 2.0));
            
            gl_FragColor = vec4(rgb * color.rgb, a * color.a);
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("time", UniformType::Float1)],
                ..Default::default()
            },
        )
        .unwrap()
    })
}

pub fn render_rgb_texture(texture: &Texture2D, x: f32, y: f32, width: f32, height: f32, time: f32) {
    let material = get_rgb_shift_material();
    gl_use_material(material);
    material.set_uniform("time", time);

    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );

    gl_use_default_material();
}
