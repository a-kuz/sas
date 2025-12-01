use crate::compat_rand::*;
use crate::game::map::Map;
use crate::game::particle::Particle;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

const LIGHTNING_BEAM_DURATION: f32 = 0.15;
const LIGHTNING_CORE_WIDTH: f32 = 6.0;
const LIGHTNING_MAX_RANGE: f32 = 768.0;
const LIGHTNING_SEGMENTS: usize = 16;
const LIGHTNING_JITTER: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct LightningBeam {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub segments: Vec<(f32, f32)>,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub player_id: u16,
    pub material: Material,
}

#[derive(Clone, Debug)]
pub struct LightningImpact {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct LightningEffects {
    pub beams: Vec<LightningBeam>,
    pub impacts: Vec<LightningImpact>,
    pub material: Option<Material>,
}

impl LightningBeam {
    pub fn new(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        player_color: Color,
        material: Material,
    ) -> Self {
        let mut segments = Vec::new();

        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();

        let perp_x = -dy / length;
        let perp_y = dx / length;

        segments.push((start_x, start_y));

        for i in 1..LIGHTNING_SEGMENTS {
            let t = i as f32 / LIGHTNING_SEGMENTS as f32;
            let base_x = start_x + dx * t;
            let base_y = start_y + dy * t;

            let jitter_amount = LIGHTNING_JITTER * (1.0 - (t - 0.5).abs() * 2.0);
            let offset = gen_range_f32(-jitter_amount, jitter_amount);

            let jittered_x = base_x + perp_x * offset;
            let jittered_y = base_y + perp_y * offset;

            segments.push((jittered_x, jittered_y));
        }

        segments.push((end_x, end_y));

        Self {
            start_x,
            start_y,
            end_x,
            end_y,
            segments,
            life: 0.0,
            max_life: LIGHTNING_BEAM_DURATION,
            color: player_color,
            player_id: 0,
            material,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life += dt;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let t = self.life / self.max_life;
        let fade = 1.0 - (t * t * t);

        if fade < 0.01 {
            return;
        }

        let time = get_time() as f32;

        self.material.set_uniform("time", time);
        self.material.set_uniform("fade", fade);
        self.material.set_uniform(
            "beamColor",
            (self.color.r, self.color.g, self.color.b, self.color.a),
        );
        self.material.set_uniform("cameraPos", (camera_x, camera_y));

        gl_use_material(&self.material);

        for i in 0..self.segments.len() - 1 {
            let (x1, y1) = self.segments[i];
            let (x2, y2) = self.segments[i + 1];

            let screen_x1 = x1 - camera_x;
            let screen_y1 = y1 - camera_y;
            let screen_x2 = x2 - camera_x;
            let screen_y2 = y2 - camera_y;

            let dx = screen_x2 - screen_x1;
            let dy = screen_y2 - screen_y1;
            let length = (dx * dx + dy * dy).sqrt();

            if length < 0.1 {
                continue;
            }

            let perp_x = -dy / length;
            let perp_y = dx / length;

            let width = LIGHTNING_CORE_WIDTH * fade;

            let core_color = Color::from_rgba(255, 255, 255, (fade * 255.0) as u8);

            let outer_color = Color::from_rgba(
                (self.color.r * 255.0) as u8,
                (self.color.g * 255.0) as u8,
                (self.color.b * 255.0) as u8,
                (fade * 180.0) as u8,
            );

            let outer_color_bytes = [
                (outer_color.r * 255.0) as u8,
                (outer_color.g * 255.0) as u8,
                (outer_color.b * 255.0) as u8,
                (outer_color.a * 255.0) as u8,
            ];

            let vertices = [
                Vertex {
                    position: Vec3::new(
                        screen_x1 + perp_x * width * 1.5,
                        screen_y1 + perp_y * width * 1.5,
                        0.0,
                    ),
                    uv: Vec2::new(0.0, 0.0),
                    color: outer_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(
                        screen_x2 + perp_x * width * 1.5,
                        screen_y2 + perp_y * width * 1.5,
                        0.0,
                    ),
                    uv: Vec2::new(1.0, 0.0),
                    color: outer_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(
                        screen_x2 - perp_x * width * 1.5,
                        screen_y2 - perp_y * width * 1.5,
                        0.0,
                    ),
                    uv: Vec2::new(1.0, 1.0),
                    color: outer_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(
                        screen_x1 - perp_x * width * 1.5,
                        screen_y1 - perp_y * width * 1.5,
                        0.0,
                    ),
                    uv: Vec2::new(0.0, 1.0),
                    color: outer_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
            ];

            let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

            draw_mesh(&Mesh {
                vertices: vertices.to_vec(),
                indices: indices.to_vec(),
                texture: None,
            });

            let core_color_bytes = [
                (core_color.r * 255.0) as u8,
                (core_color.g * 255.0) as u8,
                (core_color.b * 255.0) as u8,
                (core_color.a * 255.0) as u8,
            ];

            let core_vertices = [
                Vertex {
                    position: Vec3::new(
                        screen_x1 + perp_x * width * 0.5,
                        screen_y1 + perp_y * width * 0.5,
                        0.0,
                    ),
                    uv: Vec2::new(0.0, 0.0),
                    color: core_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(
                        screen_x2 + perp_x * width * 0.5,
                        screen_y2 + perp_y * width * 0.5,
                        0.0,
                    ),
                    uv: Vec2::new(1.0, 0.0),
                    color: core_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(
                        screen_x2 - perp_x * width * 0.5,
                        screen_y2 - perp_y * width * 0.5,
                        0.0,
                    ),
                    uv: Vec2::new(1.0, 1.0),
                    color: core_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(
                        screen_x1 - perp_x * width * 0.5,
                        screen_y1 - perp_y * width * 0.5,
                        0.0,
                    ),
                    uv: Vec2::new(0.0, 1.0),
                    color: core_color_bytes,
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
            ];

            draw_mesh(&Mesh {
                vertices: core_vertices.to_vec(),
                indices: indices.to_vec(),
                texture: None,
            });
        }

        gl_use_default_material();
    }
}

impl LightningImpact {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            x,
            y,
            life: 0.0,
            max_life: 0.3,
            color,
            radius: 16.0,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life += dt;
        self.life < self.max_life
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        let t = self.life / self.max_life;
        let fade = 1.0 - (t * t);

        let current_radius = self.radius * (0.8 + t * 0.2);
        let alpha = (fade * 220.0) as u8;

        let impact_color = Color::from_rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            alpha,
        );

        draw_circle(screen_x, screen_y, current_radius, impact_color);

        let inner_color = Color::from_rgba(255, 255, 255, (alpha as f32 * 0.8) as u8);
        draw_circle(screen_x, screen_y, current_radius * 0.5, inner_color);

        for i in 0..6 {
            let angle = (i as f32 * std::f32::consts::PI / 3.0) + t * 2.0;
            let spark_length = current_radius * 1.5 * fade;
            let end_x = screen_x + angle.cos() * spark_length;
            let end_y = screen_y + angle.sin() * spark_length;

            draw_line(screen_x, screen_y, end_x, end_y, 2.0 * fade, inner_color);
        }
    }

    pub fn create_particles(&self) -> Vec<Particle> {
        let mut particles = Vec::new();

        for _ in 0..8 {
            let angle = gen_range_f32(0.0, std::f32::consts::PI * 2.0);
            let speed = gen_range_f32(1.5, 4.0);
            let particle = Particle::new(
                self.x,
                self.y,
                angle.cos() * speed,
                angle.sin() * speed,
                true,
            );
            particles.push(particle);
        }

        particles
    }
}

impl LightningEffects {
    pub fn new() -> Self {
        Self {
            beams: Vec::new(),
            impacts: Vec::new(),
            material: None,
        }
    }

    pub fn init_material(&mut self) {
        if self.material.is_none() {
            self.material = Some(create_lightning_beam_material());
        }
    }

    pub fn fire_lightning(
        &mut self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        player_color: Color,
    ) {
        if self.material.is_none() {
            self.init_material();
        }

        let material = self.material.as_ref().unwrap().clone();
        let beam = LightningBeam::new(start_x, start_y, end_x, end_y, player_color, material);
        self.beams.push(beam);

        let impact = LightningImpact::new(end_x, end_y, player_color);
        self.impacts.push(impact);
    }

    pub fn get_linear_lights(&self) -> Vec<super::map::LinearLight> {
        let mut lights = Vec::new();

        for beam in &self.beams {
            let t = beam.life / beam.max_life;
            let fade = 1.0 - (t * t * t);
            if fade > 0.1 {
                lights.push(super::map::LinearLight {
                    start_x: beam.start_x,
                    start_y: beam.start_y,
                    end_x: beam.end_x,
                    end_y: beam.end_y,
                    width: LIGHTNING_CORE_WIDTH * 20.0 * fade,
                    r: (beam.color.r * 255.0) as u8,
                    g: (beam.color.g * 255.0) as u8,
                    b: (beam.color.b * 255.0) as u8,
                    intensity: fade * 10.0,
                });
            }
        }

        lights
    }

    pub fn get_impact_lights(&self) -> Vec<super::map::LightSource> {
        let mut lights = Vec::new();

        for impact in &self.impacts {
            let t = impact.life / impact.max_life;
            let fade = 1.0 - (t * t);
            if fade > 0.1 {
                lights.push(super::map::LightSource {
                    x: impact.x,
                    y: impact.y,
                    radius: impact.radius * 3.0 * fade,
                    r: (impact.color.r * 255.0) as u8,
                    g: (impact.color.g * 255.0) as u8,
                    b: (impact.color.b * 255.0) as u8,
                    intensity: fade * 6.0,
                    flicker: false,
                });
            }
        }

        lights
    }

    pub fn update(&mut self, dt: f32) {
        self.beams.retain_mut(|beam| beam.update(dt));
        self.impacts.retain_mut(|impact| impact.update(dt));
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        for beam in &self.beams {
            beam.render(camera_x, camera_y);
        }

        for impact in &self.impacts {
            impact.render(camera_x, camera_y);
        }
    }

    pub fn create_impact_particles(&self) -> Vec<Particle> {
        let mut all_particles = Vec::new();

        for impact in &self.impacts {
            if impact.life < 0.05 {
                let particles = impact.create_particles();
                all_particles.extend(particles);
            }
        }

        all_particles
    }
}

pub fn fire_lightning_hitscan(
    start_x: f32,
    start_y: f32,
    angle: f32,
    _owner_id: u16,
    map: &Map,
) -> (f32, f32, Vec<(usize, i32, f32, f32)>) {
    let mut current_x = start_x;
    let mut current_y = start_y;
    let hits = Vec::new();

    let dx = angle.cos();
    let dy = angle.sin();

    let step_size = 4.0;
    let max_steps = (LIGHTNING_MAX_RANGE / step_size) as i32;

    for _ in 0..max_steps {
        current_x += dx * step_size;
        current_y += dy * step_size;

        let tile_x = (current_x / 32.0) as i32;
        let tile_y = (current_y / 16.0) as i32;

        if map.is_solid(tile_x, tile_y) {
            break;
        }
    }

    (current_x, current_y, hits)
}

pub fn get_player_lightning_color(player_id: u16) -> Color {
    match player_id {
        1 => Color::from_rgba(100, 200, 255, 255),
        2 => Color::from_rgba(255, 100, 255, 255),
        3 => Color::from_rgba(100, 255, 255, 255),
        4 => Color::from_rgba(255, 255, 100, 255),
        _ => Color::from_rgba(150, 200, 255, 255),
    }
}

pub fn create_lightning_beam_material() -> Material {
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
    
    uniform float time;
    uniform float fade;
    uniform vec4 beamColor;
    uniform vec2 cameraPos;
    
    float random(vec2 st) {
        return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
    }
    
    float noise(vec2 st) {
        vec2 i = floor(st);
        vec2 f = fract(st);
        float a = random(i);
        float b = random(i + vec2(1.0, 0.0));
        float c = random(i + vec2(0.0, 1.0));
        float d = random(i + vec2(1.0, 1.0));
        vec2 u = f * f * (3.0 - 2.0 * f);
        return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
    }
    
    float fbm(vec2 st) {
        float value = 0.0;
        float amplitude = 0.5;
        float frequency = 2.0;
        for (int i = 0; i < 4; i++) {
            value += amplitude * noise(st * frequency);
            frequency *= 2.0;
            amplitude *= 0.5;
        }
        return value;
    }
    
    void main() {
        vec2 st = uv;
        
        float dist_from_center = abs(st.y - 0.5) * 2.0;
        
        float electric_noise = fbm(vec2(st.x * 10.0 + time * 8.0, st.y * 5.0 + time * 3.0));
        electric_noise += fbm(vec2(st.x * 20.0 - time * 12.0, st.y * 10.0 - time * 5.0)) * 0.5;
        
        float pulse = sin(time * 15.0 + st.x * 20.0) * 0.5 + 0.5;
        electric_noise = electric_noise * 0.7 + pulse * 0.3;
        
        float edge = smoothstep(0.3, 0.7, electric_noise);
        float core_intensity = 1.0 - dist_from_center;
        core_intensity = pow(core_intensity, 2.0);
        
        float glow = exp(-dist_from_center * 3.0);
        
        vec3 electric_color = mix(beamColor.rgb, vec3(1.0, 1.0, 1.0), core_intensity * 0.8);
        electric_color += vec3(0.3, 0.5, 1.0) * edge * 0.4;
        
        float alpha = (core_intensity * 0.7 + glow * 0.3) * fade * edge;
        alpha = clamp(alpha, 0.0, 1.0);
        
        gl_FragColor = vec4(electric_color * color.rgb, alpha * color.a);
    }"#;

    load_material(
        ShaderSource::Glsl {
            vertex: vertex_shader,
            fragment: fragment_shader,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("fade", UniformType::Float1),
                UniformDesc::new("beamColor", UniformType::Float4),
                UniformDesc::new("cameraPos", UniformType::Float2),
            ],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                depth_write: false,
                depth_test: Comparison::Always,
                cull_face: miniquad::CullFace::Nothing,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
