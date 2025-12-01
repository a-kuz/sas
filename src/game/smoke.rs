use macroquad::prelude::*;
use std::sync::OnceLock;

static VOLUMETRIC_SMOKE_MATERIAL: OnceLock<Material> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Smoke {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub life: u32,
    pub max_life: u32,
    pub alpha: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub rotation: f32,
    pub turbulence_offset: f32,
    pub density_multiplier: f32,
}

impl Smoke {
    pub fn new(x: f32, y: f32, size: f32) -> Self {
        use crate::compat_rand::*;
        Self {
            x,
            y,
            radius: size,
            life: 0,
            max_life: 240, // 4 раза длиннее
            alpha: 0.9,
            vel_x: gen_range_f32(-0.5, 0.5),
            vel_y: gen_range_f32(-0.1, 0.05),
            rotation: gen_range_f32(0.0, 6.28),
            turbulence_offset: gen_range_f32(0.0, 100.0),
            density_multiplier: 2.5, // темнее
        }
    }

    pub fn update(&mut self) -> bool {
        self.life += 1;

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.vel_x *= 0.98;
        self.vel_y *= 0.98;

        self.radius += 0.1;
        self.rotation += 0.02;

        let t = self.life as f32 / self.max_life as f32;
        self.alpha = 0.9 * (1.0 - t * t);

        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let material = get_volumetric_smoke_material();

        let t = self.life as f32 / self.max_life as f32;
        let time = get_time() as f32;

        material.set_uniform("iTime", time + self.turbulence_offset);
        material.set_uniform("iResolution", (self.radius * 2.0, self.radius * 2.0));
        material.set_uniform("smokeAge", t);
        material.set_uniform("smokeAlpha", self.alpha);
        material.set_uniform("rotation", self.rotation);
        material.set_uniform("turbulenceOffset", self.turbulence_offset);
        material.set_uniform("densityMultiplier", self.density_multiplier);

        gl_use_material(material);

        let size = self.radius * 2.0;
        draw_rectangle(
            screen_x - self.radius,
            screen_y - self.radius,
            size,
            size,
            WHITE,
        );

        gl_use_default_material();
    }
}

fn get_volumetric_smoke_material() -> &'static Material {
    VOLUMETRIC_SMOKE_MATERIAL.get_or_init(|| {
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
        precision mediump float;
        
        varying mediump vec2 uv;
        
        uniform float iTime;
        uniform vec2 iResolution;
        uniform float smokeAge;
        uniform float smokeAlpha;
        uniform float rotation;
        uniform float turbulenceOffset;
        uniform float densityMultiplier;
        
        float hash(vec2 p) {
            vec3 p3 = fract(vec3(p.xyx) * 0.1031);
            p3 += dot(p3, p3.yzx + 33.33);
            return fract((p3.x + p3.y) * p3.z);
        }
        
        float noise(vec2 p) {
            vec2 i = floor(p);
            vec2 f = fract(p);
            f = f * f * (3.0 - 2.0 * f);
            
            float a = hash(i);
            float b = hash(i + vec2(1.0, 0.0));
            float c = hash(i + vec2(0.0, 1.0));
            float d = hash(i + vec2(1.0, 1.0));
            
            return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
        }
        
        float fbm(vec2 p) {
            float value = 0.0;
            float amplitude = 0.5;
            float frequency = 1.0;
            
            for (int i = 0; i < 3; i++) {
                value += amplitude * noise(p * frequency);
                frequency *= 2.0;
                amplitude *= 0.5;
            }
            
            return value;
        }
        
        vec3 volumetricSmoke(vec2 p, float time) {
            vec2 center = vec2(0.5);
            vec2 toCenter = p - center;
            float dist = length(toCenter);
            
            float angle = atan(toCenter.y, toCenter.x) + rotation;
            float swirlPhase = angle + time * 0.5 + dist * 3.0;
            
            vec2 swirl = vec2(cos(swirlPhase), sin(swirlPhase));
            
            float ageFactor = 1.0 - smokeAge;
            vec2 turbulentP = p + swirl * 0.1 * ageFactor;
            
            float timeX = time * 0.3;
            float timeY = time * 0.5;
            float n1 = fbm(turbulentP * 3.0 + vec2(timeX, timeY));
            float n2 = fbm(turbulentP * 5.0 - vec2(time * 0.2, time * 0.4));
            
            float wispy = n1 * 0.6 + n2 * 0.4;
            
            float radialFade = 1.0 - smoothstep(0.2, 0.8, dist);
            
            float density = wispy * radialFade;
            density = pow(density, 1.5 - smokeAge * 0.5);
            density *= densityMultiplier;

            float depthGradient = (1.0 - abs(p.x - 0.5) * 0.5) * (1.0 - abs(p.y - 0.5) * 0.3);
            density *= depthGradient;

            float edgeFade = smoothstep(0.0, 0.1, dist) * smoothstep(1.0, 0.7, dist);
            density *= edgeFade;
            
            vec3 darkSmoke = vec3(0.15, 0.14, 0.13);
            vec3 lightSmoke = vec3(0.5, 0.48, 0.45);
            vec3 hotSmoke = vec3(0.8, 0.4, 0.2);
            
            vec3 smokeColor = mix(darkSmoke, lightSmoke, wispy);
            
            float heat = (1.0 - smokeAge) * (1.0 - dist * 2.0);
            heat = max(0.0, heat);
            smokeColor = mix(smokeColor, hotSmoke, heat * 0.3);
            
            return smokeColor * density;
        }
        
        void main() {
            vec2 p = uv;
            
            float time = iTime + turbulenceOffset;
            
            vec3 smoke = volumetricSmoke(p, time);
            
            float layerScale = 1.0 - smokeAge * 0.5;
            if (layerScale > 0.3) {
                float layerOffset1 = 0.05 * sin(time * 2.0);
                float layerOffset2 = 0.03 * cos(time * 3.0);
                vec3 layer1 = volumetricSmoke(p + vec2(layerOffset1, layerOffset2) * layerScale, time + 0.5);
                smoke = smoke + layer1 * 0.5 * layerScale;
            }
            
            float totalDensity = (smoke.r + smoke.g + smoke.b) * 0.333333;
            float alpha = totalDensity * smokeAlpha;
            
            alpha = clamp(alpha, 0.0, 0.95);
            
            vec2 center = p - vec2(0.5);
            float vignette = 1.0 - length(center) * 0.8;
            alpha *= vignette;
            
            gl_FragColor = vec4(smoke, alpha);
        }"#;
        
        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("iTime", UniformType::Float1),
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("smokeAge", UniformType::Float1),
                    UniformDesc::new("smokeAlpha", UniformType::Float1),
                    UniformDesc::new("rotation", UniformType::Float1),
                    UniformDesc::new("turbulenceOffset", UniformType::Float1),
                    UniformDesc::new("densityMultiplier", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::Value(miniquad::BlendValue::SourceAlpha),
                        miniquad::BlendFactor::OneMinusValue(miniquad::BlendValue::SourceAlpha),
                    )),
                    depth_test: miniquad::Comparison::Always,
                    depth_write: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap()
    })
}
