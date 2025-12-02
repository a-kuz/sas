use std::sync::Arc;
use wgpu::*;
use crate::wgpu_renderer::renderer::WgpuRenderer;
use crate::wgpu_renderer::deferred_renderer::WgpuDeferredRenderer;
use crate::wgpu_renderer::tile_renderer::WgpuTileRenderer;
use crate::wgpu_renderer::md3_renderer::WgpuMD3Renderer;
use crate::game::map::Map;

pub struct WgpuRenderContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface_config: SurfaceConfiguration,
    pub deferred_renderer: Option<WgpuDeferredRenderer>,
    pub tile_renderer: Option<WgpuTileRenderer>,
    pub md3_renderer: Option<crate::wgpu_renderer::md3_renderer::WgpuMD3Renderer>,
}

impl WgpuRenderContext {
    pub fn new(renderer: &WgpuRenderer) -> Self {
        Self {
            device: renderer.device.clone(),
            queue: renderer.queue.clone(),
            surface_config: renderer.surface_config.clone(),
            deferred_renderer: None,
            tile_renderer: None,
            md3_renderer: None,
        }
    }

    pub fn init_deferred_renderer(&mut self, renderer: &WgpuRenderer, map: &Map) {
        if self.deferred_renderer.is_none() {
            self.deferred_renderer = Some(WgpuDeferredRenderer::new(renderer, map));
        }
    }

    pub fn init_tile_renderer(&mut self) {
        if self.tile_renderer.is_none() {
            self.tile_renderer = Some(WgpuTileRenderer::new(
                self.device.clone(),
                self.queue.clone(),
            ));
        }
    }

    pub fn init_md3_renderer(&mut self) {
        if self.md3_renderer.is_none() {
            self.md3_renderer = Some(WgpuMD3Renderer::new(
                self.device.clone(),
                self.queue.clone(),
            ));
        }
    }

    pub fn render_frame(
        &mut self,
        renderer: &mut WgpuRenderer,
        map: &Map,
        static_lights: &[crate::game::map::LightSource],
        all_lights: &[crate::game::map::LightSource],
        linear_lights: &[crate::game::map::LinearLight],
        camera_x: f32,
        camera_y: f32,
        zoom: f32,
        screen_w: u32,
        screen_h: u32,
        time: f32,
        ambient: f32,
        disable_shadows: bool,
        disable_dynamic_lights: bool,
    ) -> Option<()> {
        let frame = renderer.begin_frame()?;
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        {
            let _clear_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        let surface_format = self.surface_config.format;

        if let Some(ref mut deferred_renderer) = self.deferred_renderer {
            deferred_renderer.begin_scene_with_scale(1.0, zoom, screen_w, screen_h);
            
            if let Some(ref scene_target) = deferred_renderer.scene_target {
                let scene_view = scene_target.texture.create_view(&TextureViewDescriptor::default());
                let scene_format = TextureFormat::Rgba8UnormSrgb;
                
                if let Some(ref mut tile_renderer) = self.tile_renderer {
                    tile_renderer.render_tiles(
                        &mut encoder,
                        &scene_view,
                        scene_format,
                        map,
                        camera_x,
                        camera_y,
                        zoom,
                        screen_w as f32,
                        screen_h as f32,
                        time,
                        true,
                    );
                }
            }
        } else {
            if let Some(ref mut tile_renderer) = self.tile_renderer {
                tile_renderer.render_tiles(
                    &mut encoder,
                    &view,
                    surface_format,
                    map,
                    camera_x,
                    camera_y,
                    zoom,
                    screen_w as f32,
                    screen_h as f32,
                    time,
                    false,
                );
            }
        }

        if let Some(ref mut deferred_renderer) = self.deferred_renderer {
            deferred_renderer.apply_lighting(
                map,
                static_lights,
                all_lights,
                linear_lights,
                camera_x,
                camera_y,
                zoom,
                ambient,
                disable_shadows,
                disable_dynamic_lights,
                false,
            );
            
            let num_dyn = deferred_renderer.num_dynamic_lights;
            let num_lin = deferred_renderer.num_linear_lights;
            deferred_renderer.render_lighting(
                &mut encoder,
                &view,
                surface_format,
                screen_w,
                screen_h,
                camera_x,
                camera_y,
                zoom,
                time,
                num_dyn,
                num_lin,
                disable_shadows,
                ambient,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        renderer.end_frame(frame);

        Some(())
    }
}

