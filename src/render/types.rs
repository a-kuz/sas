use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct RenderTriangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub n0: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    pub uv0: Vec2,
    pub uv1: Vec2,
    pub uv2: Vec2,
}

#[derive(Clone, Debug)]
pub struct RenderMesh {
    pub triangles: Vec<RenderTriangle>,
    pub texture_id: u64,
}

#[derive(Clone, Debug)]
pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub radius: f32,
    pub intensity: f32,
}

#[derive(Clone, Debug)]
pub struct LinearLight {
    pub start: Vec2,
    pub end: Vec2,
    pub width: f32,
    pub color: Vec3,
}

#[derive(Clone, Debug)]
pub struct ShadowCaster {
    pub min: Vec2,
    pub max: Vec2,
}

pub struct RenderBatch {
    pub meshes: Vec<RenderMesh>,
    pub lights: Vec<Light>,
    pub linear_lights: Vec<LinearLight>,
    pub shadow_casters: Vec<ShadowCaster>,
    pub camera: Vec2,
    pub ambient: f32,
    pub map_size: Vec2,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            lights: Vec::new(),
            linear_lights: Vec::new(),
            shadow_casters: Vec::new(),
            camera: Vec2::ZERO,
            ambient: 0.06,
            map_size: Vec2::new(1024.0, 1024.0),
        }
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}


