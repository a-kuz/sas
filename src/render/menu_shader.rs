use macroquad::prelude::*;
use std::cell::RefCell;
use std::sync::OnceLock;

static MENU_FIRE_MATERIAL: OnceLock<Material> = OnceLock::new();

thread_local! {
    static MENU_TEXT_RT: RefCell<Option<RenderTarget>> = RefCell::new(None);
}

pub fn init_menu_shader() {
    let _ = MENU_FIRE_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        
        varying lowp vec2 uv;
        
        uniform mat4 Model;
        uniform mat4 Projection;
        
        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision highp float;
        
        varying highp vec2 uv;
        
        uniform vec2 iResolution;
        uniform float iTime;
        uniform vec2 iMouse;
        
        // Tanh approximation for GLSL 100
        vec4 tanh_approx(vec4 x) {
            vec4 e2x = exp(2.0 * x);
            return (e2x - 1.0) / (e2x + 1.0);
        }

        void main() {
            vec2 I = uv * iResolution;
            vec4 O = vec4(0.0);
            
            // Time for animation
            float t = iTime / 2.0;
            
            // Raymarch loop iterator
            float i = 0.0;
            // Raymarched depth
            float z = 0.0;
            // Raymarch step size and "Turbulence" frequency
            float d = 0.0;

            // Raymarching loop with 50 iterations
            for (int j = 0; j < 50; j++) {
                i += 1.0;
                
                // Add color and glow attenuation
                O += (sin(z / 3.0 + vec4(7.0, 2.0, 3.0, 0.0)) + 1.1) / d;
                
                // Compute raymarch sample point
                vec3 p = z * normalize(vec3(I + I, 0.0) - vec3(iResolution, 0.0));
                // Shift back and animate
                p.z += 5.0 + cos(t);
                // Twist and rotate
                float c = cos(p.y * 0.5 + 0.0); // vec4(0, 33, 11, 0) -> 0 for x
                float s = sin(p.y * 0.5 + 0.0);
                mat2 m = mat2(c, -s, s, c);
                p.xz = m * p.xz;
                
                // Expand upward
                p /= max(p.y * 0.1 + 1.0, 0.1);
                
                // Turbulence loop (increase frequency)
                d = 2.0;
                for (int k = 0; k < 5; k++) { // approximate loop
                    if (d >= 15.0) break;
                    
                    // Add a turbulence wave
                    // p.yzx swizzle
                    vec3 pyzx = vec3(p.y, p.z, p.x);
                    p += cos((pyzx - vec3(t / 0.1, t, d)) * d) / d;
                    
                    d /= 0.6;
                }
                
                // Sample approximate distance to hollow cone
                float dist = 0.01 + abs(length(p.xz) + p.y * 0.3 - 0.5) / 7.0;
                d = dist;
                z += dist;
                
                if (i >= 50.0) break;
            }
            
            // Tanh tonemapping
            O = tanh_approx(O / 1000.0);
            
            // Mouse shadow effect
            vec2 mousePos = iMouse;
            float distToMouse = length(I - mousePos);
            float shadowRadius = 150.0;
            float shadowStrength = smoothstep(shadowRadius, 0.0, distToMouse);
            O.rgb *= (1.0 - shadowStrength * 0.7);
            
            O.a = 1.0;
            
            gl_FragColor = O;
        }"#;

        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("iTime", UniformType::Float1),
                    UniformDesc::new("iMouse", UniformType::Float2),
                ],
                ..Default::default()
            },
        )
        .unwrap()
    });
}

pub fn draw_menu_background(time: f32) {
    init_menu_shader();

    let w = screen_width();
    let h = screen_height();
    let mouse_pos = mouse_position();

    let material = MENU_FIRE_MATERIAL.get().unwrap();

    material.set_uniform("iResolution", (w, h));
    material.set_uniform("iTime", time);
    material.set_uniform("iMouse", (mouse_pos.0, mouse_pos.1));

    gl_use_material(material);
    draw_rectangle(0.0, 0.0, w, h, WHITE);
    gl_use_default_material();
}

pub fn draw_menu_items(selected: usize, items: &[&str], logo_texture: Option<&Texture2D>) {
    let w = screen_width();
    let h = screen_height();

    // Draw Logo
    if let Some(texture) = logo_texture {
        let scale = 0.5; // Adjust scale as needed
        let logo_w = texture.width() * scale;
        let logo_h = texture.height() * scale;
        let logo_x = w * 0.5 - logo_w * 0.5;
        let logo_y = h * 0.1; // Position near top

        draw_texture_ex(
            texture,
            logo_x,
            logo_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(logo_w, logo_h)),
                ..Default::default()
            },
        );
    } else {
        crate::render::draw_q3_banner_string("SAS III", w * 0.5 - 100.0, 80.0, 48.0, WHITE);
    }

    let item_h = 54.0;
    let start_y = h * 0.5 - (items.len() as f32 * (item_h + 12.0)) * 0.5;
    let right_margin = 100.0;

    for (i, label) in items.iter().enumerate() {
        let y = start_y + (i as f32) * (item_h + 12.0);

        let text_color = if i == selected {
            Color::from_rgba(255, 64, 64, 255)
        } else {
            Color::from_rgba(210, 220, 230, 255)
        };
        let size = if i == selected { 36.0 } else { 30.0 };

        let text_width = crate::render::measure_q3_banner_string(&label.to_uppercase(), size);
        let x = w - text_width - right_margin;

        crate::render::draw_q3_banner_string(&label.to_uppercase(), x, y + 10.0, size, text_color);
    }
}
