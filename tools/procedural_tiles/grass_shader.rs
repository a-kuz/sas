use macroquad::prelude::*;
use std::sync::OnceLock;

static GRASS_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn get_grass_material() -> &'static Material {
    GRASS_MATERIAL.get_or_init(|| create_grass_material())
}

fn create_grass_material() -> Material {
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
    
    float hash(float n) {
        return fract(sin(n) * 43758.5453123);
    }
    
    float noise(vec2 st) {
        vec2 i = floor(st);
        vec2 f = fract(st);
        float a = hash(i.x + i.y * 57.0);
        float b = hash(i.x + 1.0 + i.y * 57.0);
        float c = hash(i.x + (i.y + 1.0) * 57.0);
        float d = hash(i.x + 1.0 + (i.y + 1.0) * 57.0);
        vec2 u = f * f * (3.0 - 2.0 * f);
        return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
    }
    
    void main() {
        float y = 1.0 - uv.y;
        float blade_id = floor(uv.x * 15.0);
        float blade_x = fract(uv.x * 15.0);
        
        float seed = hash(blade_id + color.r);
        if (seed < 0.5) discard;
        
        float max_h = 0.6 + hash(blade_id * 1.3 + color.g) * 0.4;
        if (y > max_h) discard;
        
        float t = y / max_h;
        
        float wind = sin(time * 1.8 + blade_id * 0.7) * 0.25;
        float bend = (wind + (hash(blade_id * 2.1) - 0.5) * 0.6) * t * t;
        
        float dist = abs(blade_x - 0.5 - bend);
        float width = (0.32 + hash(blade_id * 1.9) * 0.12) * (1.0 - t * 0.75);
        
        if (t > 0.7) {
            width *= 1.0 - (t - 0.7) * 3.0;
        }
        
        if (dist > width) discard;
        
        float alpha = (1.0 - dist / width) * (1.0 - t * 0.3);
        
        vec3 col = mix(vec3(0.25, 0.45, 0.2), vec3(0.5, 0.75, 0.35), t);
        col = mix(col, vec3(0.7, 0.85, 0.5), t * t);
        
        gl_FragColor = vec4(col, alpha);
    }"#;

    load_material(
        ShaderSource::Glsl {
            vertex: vertex_shader,
            fragment: fragment_shader,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("time", UniformType::Float1),
            ],
            ..Default::default()
        },
    ).unwrap()
}

pub fn render_grass(x: f32, y: f32, width: f32, tile_x: f32, tile_y: f32) {
    let material = get_grass_material();
    gl_use_material(material);
    
    let time = macroquad::time::get_time() as f32;
    material.set_uniform("time", time);
    
    let grass_height = 8.0;
    
    let seed_r = ((tile_x * 73.0 + tile_y * 179.0) % 255.0) as u8;
    let seed_g = ((tile_x * 157.0 + tile_y * 211.0) % 255.0) as u8;
    let seed_b = ((tile_x * 97.0 + tile_y * 131.0) % 255.0) as u8;
    
    let color = Color::from_rgba(seed_r, seed_g, seed_b, 255);
    
    draw_rectangle(x, y - grass_height, width, grass_height, color);
    
    gl_use_default_material();
}

