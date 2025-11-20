use macroquad::prelude::*;
use std::sync::OnceLock;
use std::cell::RefCell;

static MENU_SHADOW_MATERIAL: OnceLock<Material> = OnceLock::new();

thread_local! {
    static MENU_TEXT_RT: RefCell<Option<RenderTarget>> = RefCell::new(None);
}

pub fn init_menu_shader() {
    let _ = MENU_SHADOW_MATERIAL.get_or_init(|| {
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
        
        uniform sampler2D textTexture;
        uniform vec2 iResolution;
        uniform vec2 iMouse;
        uniform float iTime;
        
        const int MAX_SAMPLES = 1;
        const int MAX_STEPS = 100;
        const int LIGHT_SIZE = 1;
        const float LIGHT_ATTENUATION_COEFFICIENT = 0.5;
        
        struct Ray {
            vec2 pos;
            vec2 dir;
        };
        
        float sampleText(vec2 pos) {
            vec2 texUV = pos / iResolution;
            if (texUV.x < 0.0 || texUV.x > 1.0 || texUV.y < 0.0 || texUV.y > 1.0) {
                return 0.0;
            }
            texUV.y = 1.0 - texUV.y;
            return texture2D(textTexture, texUV).a;
        }
        
        bool raySceneIntersect(in Ray ray, in vec2 stopPos) {
            vec2 pos = ray.pos;
            float dist = distance(ray.pos, stopPos);
            float stepSize = dist / float(MAX_STEPS);
            
            for (int stepIndex = 0; stepIndex < MAX_STEPS; stepIndex++) {
                if (distance(pos, stopPos) < .50) {
                    return false;
                }
                
                float d = sampleText(pos);
                if (d > 0.5) {
                    return true;
                }
                
                pos += ray.dir * stepSize;
            }
            
            return false;
        }
        
        vec3 spectral(float x) {
            vec3 col = 4.0 * (x - vec3(0.75, 0.5, 0.25));
            return max(vec3(0.0), vec3(1.0) - col * col);
        }
        
        void main() {
            vec2 fragCoord = uv * iResolution;
            
            float scene = sampleText(fragCoord);
            if (scene > 0.5) {
                gl_FragColor = vec4(vec3(scene), 1.0);
                return;
            }
            
            vec2 lightPos = iMouse;
            if (length(lightPos) < 1.0) {
                lightPos = iResolution * 0.5;
            }
            
            float lightRadius = float(LIGHT_SIZE);
            
            vec3 lightColour = clamp(spectral(mod(iTime / 10.0, 1.0)) + 0.5, 0.0, 1.0);
            
            if (distance(fragCoord, lightPos) < lightRadius) {
                gl_FragColor = vec4(lightColour, 1.0);
                return;
            }
            
            vec2 dir = normalize(lightPos - fragCoord);
            vec2 tangent = vec2(-dir.y, dir.x);
            
            vec2 start = lightPos + tangent * lightRadius;
            vec2 end = lightPos - tangent * lightRadius;
            
            int numSamples = MAX_SAMPLES;
            float intensityStep = 1.0 / float(numSamples + 1);
            
            float intensity = 0.1;
            
            for (int offset = 0; offset <= MAX_SAMPLES; offset++) {
                if (offset > numSamples) break;
                
                vec2 target = mix(start, end, float(offset) / float(numSamples));
                vec2 rayDir = normalize(target - fragCoord);
                
                Ray ray = Ray(fragCoord + rayDir * 2.0, rayDir);
                
                if (!raySceneIntersect(ray, target)) {
                    float dist = distance(target, fragCoord) - lightRadius;
                    intensity += intensityStep * exp(-LIGHT_ATTENUATION_COEFFICIENT * dist / 100.0);
                }
            }
            
            gl_FragColor = vec4(lightColour * clamp(intensity, 0.0, 1.0), 1.0);
        }"#;
        
        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("iMouse", UniformType::Float2),
                    UniformDesc::new("iTime", UniformType::Float1),
                ],
                textures: vec!["textTexture".to_string()],
                ..Default::default()
            },
        ).unwrap()
    });
}

pub fn draw_menu_with_shadows(selected: usize, items: &[&str], time: f32) {
    init_menu_shader();
    
    let w = screen_width();
    let h = screen_height();
    
    let rt = MENU_TEXT_RT.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() || opt.as_ref().unwrap().texture.width() != w {
            *opt = Some(render_target((w) as u32, (h) as u32));
        }
        opt.as_ref().unwrap().clone()
    });
    
    set_camera(&Camera2D {
        render_target: Some(rt.clone()),
        zoom: vec2(1.0 / w, -1.0 / h),
        target: vec2(w * 0.5, h * 0.5),
        ..Default::default()
    });
    
    clear_background(Color::from_rgba(0, 0, 0, 0));
    
    crate::render::draw_q3_banner_string("SAS III", w * 0.5 - 100.0, 80.0, 48.0, WHITE);
   
    
    let item_h = 54.0;
    let item_w = 400.0;
    let start_y = h * 0.5 - (items.len() as f32 * (item_h + 12.0)) * 0.5;
    
    for (i, label) in items.iter().enumerate() {
        let y = start_y + (i as f32) * (item_h + 12.0);
        let x = w * 0.5 - item_w * 0.5;
        let size = if i == selected { 36.0 } else { 30.0 };
        crate::render::draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, size, WHITE);
    }
    
    set_default_camera();
    
    let material = MENU_SHADOW_MATERIAL.get().unwrap();
    let mouse_pos = mouse_position();
    
    material.set_uniform("iResolution", (w, h));
    material.set_uniform("iMouse", mouse_pos);
    material.set_uniform("iTime", time);
    material.set_texture("textTexture", rt.texture.clone());
    
    gl_use_material(material);
    draw_rectangle(0.0, 0.0, w, h, WHITE);
    gl_use_default_material();
    
    set_camera(&Camera2D {
        render_target: Some(rt.clone()),
        zoom: vec2(1.0 / w, -1.0 / h),
        target: vec2(w * 0.5, h * 0.5),
        ..Default::default()
    });
    
    for (i, label) in items.iter().enumerate() {
        let y = start_y + (i as f32) * (item_h + 12.0);
        let x = w * 0.5 - item_w * 0.5;
        
        let text_color = if i == selected { 
            Color::from_rgba(255, 64, 64, 255) 
        } else { 
            Color::from_rgba(210, 220, 230, 255) 
        };
        let size = if i == selected { 36.0 } else { 30.0 };
        crate::render::draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, size, text_color);
    }
    
    set_default_camera();
    
    draw_texture_ex(
        &rt.texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            flip_y: true,
            ..Default::default()
        },
    );
}

