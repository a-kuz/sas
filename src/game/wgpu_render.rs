use crate::wgpu_renderer::integration::WgpuRenderContext;
use crate::wgpu_renderer::renderer::WgpuRenderer;
use crate::wgpu_renderer::texture::WgpuTexture;
use crate::game::map::Map;

impl super::GameState {
    pub fn init_wgpu_renderer(&mut self, renderer: &WgpuRenderer) {
        if self.wgpu_render_context.is_none() {
            let mut context = WgpuRenderContext::new(renderer);
            context.init_deferred_renderer(renderer, &self.map);
            context.init_tile_renderer();
            context.init_md3_renderer();
            self.wgpu_render_context = Some(context);
        }
    }

    pub fn load_tile_textures_to_wgpu(&mut self, renderer: &WgpuRenderer) {
        if let Some(ref mut wgpu_context) = self.wgpu_render_context {
            if let Some(ref mut tile_renderer) = wgpu_context.tile_renderer {
                for texture_id in 0u16..=255u16 {
                    if let Some(image) = self.tile_textures.get(texture_id) {
                        let wgpu_texture = WgpuTexture::from_image(
                            &renderer.device,
                            &renderer.queue,
                            image,
                        );
                        tile_renderer.load_tile_texture(texture_id, wgpu_texture);
                    }
                }
            }
        }
    }

    pub fn render_wgpu(
        &mut self,
        renderer: &mut WgpuRenderer,
        camera_x: f32,
        camera_y: f32,
        zoom: f32,
    ) {
        if let Some(ref mut wgpu_context) = self.wgpu_render_context {
            let (screen_w, screen_h) = renderer.get_viewport_size();
            
            let all_lights: Vec<_> = self.lights.iter().map(|l| {
                let life_ratio = l.life as f32 / l.max_life as f32;
                let intensity = (1.0 - life_ratio).max(0.0);
                crate::game::map::LightSource {
                    x: l.x,
                    y: l.y,
                    radius: l.radius,
                    r: (l.color.r * 255.0) as u8,
                    g: (l.color.g * 255.0) as u8,
                    b: (l.color.b * 255.0) as u8,
                    intensity,
                    flicker: false,
                }
            }).collect();
            
            wgpu_context.render_frame(
                renderer,
                &self.map,
                &self.map.lights,
                &all_lights,
                &self.linear_lights,
                camera_x,
                camera_y,
                zoom,
                screen_w,
                screen_h,
                self.time as f32,
                self.ambient_light,
                self.disable_shadows,
                self.disable_dynamic_lights,
            );
        }
    }
}

