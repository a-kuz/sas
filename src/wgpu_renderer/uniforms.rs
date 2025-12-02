use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LightingUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub screen_to_world: [f32; 2],
    pub camera_pos: [f32; 2],
    pub map_size: [f32; 2],
    pub tile_size: [f32; 2],
    pub time: f32,
    pub num_dynamic_lights: i32,
    pub num_linear_lights: i32,
    pub disable_shadows: i32,
    pub ambient_light: f32,
    pub _padding0: f32,
    pub _padding1: f32,
    pub _padding2: f32,
    pub _padding3: f32,
    pub _padding4: f32,
    pub _padding5: f32,
    pub _padding6: f32,
}

impl LightingUniforms {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            screen_to_world: [0.0, 0.0],
            camera_pos: [0.0, 0.0],
            map_size: [0.0, 0.0],
            tile_size: [32.0, 16.0],
            time: 0.0,
            num_dynamic_lights: 0,
            num_linear_lights: 0,
            disable_shadows: 0,
            ambient_light: 0.3,
            _padding0: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            _padding4: 0.0,
            _padding5: 0.0,
            _padding6: 0.0,
        }
    }
}

