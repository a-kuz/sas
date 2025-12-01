use macroquad::prelude::*;

pub struct BatchedEffectsRenderer {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl BatchedEffectsRenderer {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(2048),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add_trail(&mut self, x: f32, y: f32, size: f32, color: Color) {
        self.add_circle(x, y, size, color);
    }

    pub fn add_smoke(&mut self, x: f32, y: f32, size: f32, color: Color) {
        self.add_circle(x, y, size, color);
    }

    pub fn add_plasma(&mut self, x: f32, y: f32, size: f32, color: Color) {
        self.add_circle(x, y, size, color);
    }

    fn add_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        const SEGMENTS: u16 = 16;
        let base_index = self.vertices.len() as u16;

        self.vertices.push(Vertex {
            position: Vec3::new(x, y, 0.0),
            uv: Vec2::new(0.5, 0.5),
            color: [
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                (color.a * 255.0) as u8,
            ],
            normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
        });

        for i in 0..=SEGMENTS {
            let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::PI * 2.0;
            let px = x + angle.cos() * radius;
            let py = y + angle.sin() * radius;

            self.vertices.push(Vertex {
                position: Vec3::new(px, py, 0.0),
                uv: Vec2::new(0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5),
                color: [
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    (color.a * 255.0) as u8,
                ],
                normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
            });
        }

        for i in 0..SEGMENTS {
            self.indices.push(base_index);
            self.indices.push(base_index + i + 1);
            self.indices.push(base_index + i + 2);
        }
    }

    pub fn render(&self) {
        if self.vertices.is_empty() {
            return;
        }

        let mesh = Mesh {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            texture: None,
        };

        draw_mesh(&mesh);
    }
}
