use glam::{Mat4, Vec2, Vec3};

pub struct WgpuCamera {
    pub position: Vec2,
    pub zoom: f32,
    pub target: Vec2,
    pub offset: Vec2,
}

impl WgpuCamera {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            target: Vec2::ZERO,
            offset: Vec2::ZERO,
        }
    }

    pub fn build_view_projection_matrix(&self, screen_width: f32, screen_height: f32) -> Mat4 {
        let zoom_x = (2.0 * self.zoom) / screen_width;
        let zoom_y = (2.0 * self.zoom) / screen_height;
        
        let center_x = self.target.x + self.offset.x;
        let center_y = self.target.y + self.offset.y;
        
        Mat4::orthographic_rh(
            center_x - screen_width / (2.0 * zoom_x),
            center_x + screen_width / (2.0 * zoom_x),
            center_y - screen_height / (2.0 * zoom_y),
            center_y + screen_height / (2.0 * zoom_y),
            -1.0,
            1.0,
        )
    }
}

