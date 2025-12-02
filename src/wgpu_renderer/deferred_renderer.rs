use wgpu::*;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use crate::wgpu_renderer::texture::WgpuTexture;
use crate::wgpu_renderer::renderer::WgpuRenderer;
use crate::wgpu_renderer::pipeline::RenderPipeline;
use crate::wgpu_renderer::uniforms::LightingUniforms;
use crate::wgpu_renderer::mesh::Vertex;
use crate::game::map::{Map, LightSource, LinearLight};
use crate::game::lightmap::Lightmap;
use crate::game::light_grid::LightGrid;

pub struct WgpuDeferredRenderer {
    pub scene_target: Option<WgpuTexture>,
    map_width: usize,
    map_height: usize,
    last_screen_w: u32,
    last_screen_h: u32,
    last_render_scale: f32,
    light_grid: LightGrid,
    lightmap: Option<Lightmap>,
    lightmap_wgpu: Option<WgpuTexture>,
    static_lights_dirty: bool,
    obstacle_texture: Option<WgpuTexture>,
    dynamic_light_texture: Option<WgpuTexture>,
    linear_light_texture: Option<WgpuTexture>,
    light_data_buffer: [u8; 64],
    linear_light_data_buffer: [u8; 32],
    static_uniforms_set: bool,
    lit_scene_target: Option<WgpuTexture>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Option<RenderPipeline>,
    uniform_buffer: Option<Buffer>,
    bind_group: Option<BindGroup>,
    fullscreen_vertices: Option<Buffer>,
    fullscreen_indices: Option<Buffer>,
    pub num_dynamic_lights: i32,
    pub num_linear_lights: i32,
}

impl WgpuDeferredRenderer {
    pub fn new(renderer: &WgpuRenderer, map: &Map) -> Self {
        Self::new_with_scale(renderer, map, 2.0)
    }

    pub fn new_with_scale(renderer: &WgpuRenderer, map: &Map, render_scale: f32) -> Self {
        let (screen_w, screen_h) = renderer.get_viewport_size();
        let target_w = (screen_w as f32 * render_scale) as u32;
        let target_h = (screen_h as f32 * render_scale) as u32;

        println!(
            "[WgpuDeferredRenderer] Creating render target {}x{} (scale: {}x)",
            target_w, target_h, render_scale
        );

        let scene_target = WgpuTexture::create_render_target(
            &renderer.device,
            target_w,
            target_h,
            TextureFormat::Rgba8UnormSrgb,
        );

        let lit_target = WgpuTexture::create_render_target(
            &renderer.device,
            target_w,
            target_h,
            TextureFormat::Rgba8UnormSrgb,
        );

        let dynamic_light_tex = WgpuTexture::create_render_target(
            &renderer.device,
            8,
            2,
            TextureFormat::Rgba8Unorm,
        );

        let linear_light_tex = WgpuTexture::create_render_target(
            &renderer.device,
            4,
            2,
            TextureFormat::Rgba8Unorm,
        );

        let device = renderer.device.clone();
        let queue = renderer.queue.clone();

        let light_grid = LightGrid::new(map);

        Self {
            scene_target: Some(scene_target),
            map_width: map.width,
            map_height: map.height,
            last_screen_w: screen_w,
            last_screen_h: screen_h,
            last_render_scale: render_scale,
            light_grid,
            lightmap: None,
            lightmap_wgpu: None,
            static_lights_dirty: true,
            obstacle_texture: None,
            dynamic_light_texture: Some(dynamic_light_tex),
            linear_light_texture: Some(linear_light_tex),
            light_data_buffer: [0u8; 64],
            linear_light_data_buffer: [0u8; 32],
            static_uniforms_set: false,
            lit_scene_target: Some(lit_target),
            device,
            queue,
            pipeline: None,
            uniform_buffer: None,
            bind_group: None,
            fullscreen_vertices: None,
            fullscreen_indices: None,
            num_dynamic_lights: 0,
            num_linear_lights: 0,
        }
    }
    
    fn create_fullscreen_quad(&mut self) {
        let vertices = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
        ];

        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fullscreen Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fullscreen Quad Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        self.fullscreen_vertices = Some(vertex_buffer);
        self.fullscreen_indices = Some(index_buffer);
    }

    pub fn mark_static_lights_dirty(&mut self) {
        self.static_lights_dirty = true;
        self.lightmap = None;
        self.lightmap_wgpu = None;
        self.static_uniforms_set = false;
    }

    pub fn create_obstacle_texture(&self, map: &Map) -> WgpuTexture {
        let scale = 2;
        let tile_w = 32 * scale;
        let tile_h = 16 * scale;
        let width = map.width * tile_w;
        let height = map.height * tile_h;
        let mut pixels = vec![0u8; width * height * 4];

        for ty in 0..map.height {
            for tx in 0..map.width {
                if map.tiles[tx][ty].solid {
                    for py in 0..tile_h {
                        for px in 0..tile_w {
                            let x = tx * tile_w + px;
                            let y = ty * tile_h + py;
                            let idx = (y * width + x) * 4;
                            pixels[idx] = 255;
                            pixels[idx + 1] = 255;
                            pixels[idx + 2] = 255;
                            pixels[idx + 3] = 255;
                        }
                    }
                }
            }
        }

        let img = image::RgbaImage::from_raw(width as u32, height as u32, pixels)
            .expect("Failed to create obstacle image");
        let dyn_img = image::DynamicImage::ImageRgba8(img);
        
        WgpuTexture::from_image(&self.device, &self.queue, &dyn_img)
    }

    fn update_light_data_buffer(&mut self, lights: &[LightSource], map_w: f32, map_h: f32) {
        self.light_data_buffer.fill(0);

        let num_lights = 8.min(lights.len());
        let map_size_inv = 1.0 / map_w.max(map_h);

        for i in 0..num_lights {
            let light = &lights[i];
            let idx1 = i * 4;
            let idx2 = (8 + i) * 4;

            self.light_data_buffer[idx1] = (light.x / map_w * 255.0).min(255.0) as u8;
            self.light_data_buffer[idx1 + 1] = (light.y / map_h * 255.0).min(255.0) as u8;
            self.light_data_buffer[idx1 + 2] =
                (light.radius * map_size_inv * 100.0).min(255.0) as u8;
            self.light_data_buffer[idx1 + 3] = (light.intensity * 255.0).min(255.0) as u8;

            self.light_data_buffer[idx2] = light.r;
            self.light_data_buffer[idx2 + 1] = light.g;
            self.light_data_buffer[idx2 + 2] = light.b;
            self.light_data_buffer[idx2 + 3] = if light.flicker { 255 } else { 0 };
        }

        if let Some(ref tex) = self.dynamic_light_texture {
            tex.update(&self.device, &self.queue, &self.light_data_buffer, 8, 2);
        }
    }

    fn update_linear_light_data_buffer(
        &mut self,
        linear_lights: &[LinearLight],
        map_w: f32,
        map_h: f32,
    ) {
        self.linear_light_data_buffer.fill(0);

        let map_w_inv = 1.0 / map_w;
        let map_h_inv = 1.0 / map_h;
        let map_size_inv = 1.0 / map_w.max(map_h);

        for (i, light) in linear_lights.iter().enumerate().take(4) {
            let col = i;
            let row1_idx = col * 4;
            let row2_idx = col * 4 + 16;

            let start_u = (light.start_x * map_w_inv).clamp(0.0, 1.0);
            let start_v = (light.start_y * map_h_inv).clamp(0.0, 1.0);
            let end_u = (light.end_x * map_w_inv).clamp(0.0, 1.0);
            let end_v = (light.end_y * map_h_inv).clamp(0.0, 1.0);
            let width_norm = (light.width * map_size_inv * 2.0).clamp(0.0, 1.0);

            self.linear_light_data_buffer[row1_idx] = (start_u * 255.0) as u8;
            self.linear_light_data_buffer[row1_idx + 1] = (start_v * 255.0) as u8;
            self.linear_light_data_buffer[row1_idx + 2] = (end_u * 255.0) as u8;
            self.linear_light_data_buffer[row1_idx + 3] = (end_v * 255.0) as u8;

            self.linear_light_data_buffer[row2_idx] = light.r;
            self.linear_light_data_buffer[row2_idx + 1] = light.g;
            self.linear_light_data_buffer[row2_idx + 2] = light.b;
            self.linear_light_data_buffer[row2_idx + 3] = (width_norm * 255.0) as u8;
        }

        if let Some(ref tex) = self.linear_light_texture {
            tex.update(&self.device, &self.queue, &self.linear_light_data_buffer, 4, 2);
        }
    }

    pub fn begin_scene(&mut self, screen_w: u32, screen_h: u32) {
        self.begin_scene_with_scale(self.last_render_scale, 1.0, screen_w, screen_h);
    }

    pub fn begin_scene_with_scale(&mut self, render_scale: f32, _zoom: f32, screen_w: u32, screen_h: u32) {
        let current_w = screen_w;
        let current_h = screen_h;

        if current_w != self.last_screen_w
            || current_h != self.last_screen_h
            || render_scale != self.last_render_scale
        {
            let target_w = (current_w as f32 * render_scale) as u32;
            let target_h = (current_h as f32 * render_scale) as u32;

            let scene_target = WgpuTexture::create_render_target(
                &self.device,
                target_w,
                target_h,
                TextureFormat::Rgba8UnormSrgb,
            );
            self.scene_target = Some(scene_target);

            let lit_target = WgpuTexture::create_render_target(
                &self.device,
                target_w,
                target_h,
                TextureFormat::Rgba8UnormSrgb,
            );
            self.lit_scene_target = Some(lit_target);

            self.last_screen_w = current_w;
            self.last_screen_h = current_h;
            self.last_render_scale = render_scale;
        }
    }

    pub fn end_scene(&self) {
    }

    pub fn apply_lighting(
        &mut self,
        map: &Map,
        static_lights: &[LightSource],
        all_lights: &[LightSource],
        linear_lights: &[LinearLight],
        camera_x: f32,
        camera_y: f32,
        _zoom: f32,
        ambient: f32,
        _disable_shadows: bool,
        _disable_dynamic_lights: bool,
        _cartoon_shader: bool,
    ) {
        if self.lightmap.is_none() || self.static_lights_dirty {
            println!(
                "[Lightmap] Building lightmap for {} static lights...",
                static_lights.len()
            );
            let lightmap = Lightmap::new(map, static_lights, ambient);
            
            let width = lightmap.width;
            let height = lightmap.height;
            let dyn_img = image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width as u32, height as u32, lightmap.pixels.clone())
                    .expect("Failed to create lightmap image")
            );
            
            self.lightmap_wgpu = Some(WgpuTexture::from_image(&self.device, &self.queue, &dyn_img));
            self.lightmap = Some(lightmap);
            self.light_grid.rebuild(static_lights, map);
            self.static_lights_dirty = false;
            println!("[Lightmap] Done!");
        }

        let dynamic_lights: Vec<LightSource> = all_lights
            .iter()
            .filter(|light| {
                !static_lights
                    .iter()
                    .any(|sl| (sl.x - light.x).abs() < 1.0 && (sl.y - light.y).abs() < 1.0)
            })
            .cloned()
            .collect();

        let map_w = self.map_width as f32 * 32.0;
        let map_h = self.map_height as f32 * 16.0;

        let (screen_w, screen_h) = (0.0, 0.0);
        let active_dynamic_lights: Vec<LightSource> = {
            let mut lights = Vec::with_capacity(8);
            for light in &dynamic_lights {
                lights.push(light.clone());
                if lights.len() >= 8 {
                    break;
                }
            }
            lights
        };

        self.update_light_data_buffer(&active_dynamic_lights, map_w, map_h);
        self.update_linear_light_data_buffer(linear_lights, map_w, map_h);
        
        self.num_dynamic_lights = active_dynamic_lights.len() as i32;
        self.num_linear_lights = linear_lights.len().min(4) as i32;

        if self.obstacle_texture.is_none() {
            self.obstacle_texture = Some(self.create_obstacle_texture(map));
        }
    }
    
    pub fn render_lighting(
        &mut self,
        encoder: &mut CommandEncoder,
        output_view: &TextureView,
        surface_format: TextureFormat,
        screen_w: u32,
        screen_h: u32,
        camera_x: f32,
        camera_y: f32,
        zoom: f32,
        time: f32,
        num_dynamic_lights: i32,
        num_linear_lights: i32,
        disable_shadows: bool,
        ambient_light: f32,
    ) {
        if self.pipeline.is_none() {
            self.pipeline = Some(RenderPipeline::new(&self.device, surface_format));
            self.create_fullscreen_quad();
        }

        let (screen_w_f, screen_h_f) = (screen_w as f32, screen_h as f32);
        let view_proj = glam::Mat4::orthographic_rh(
            -screen_w_f / (2.0 * zoom),
            screen_w_f / (2.0 * zoom),
            -screen_h_f / (2.0 * zoom),
            screen_h_f / (2.0 * zoom),
            -1.0,
            1.0,
        );
        
        let map_w = self.map_width as f32 * 32.0;
        let map_h = self.map_height as f32 * 16.0;
        let center_x = camera_x + screen_w_f * 0.5;
        let center_y = camera_y + screen_h_f * 0.5;
        let adjusted_x = center_x - (screen_w_f / zoom) * 0.5;
        let adjusted_y = center_y - (screen_h_f / zoom) * 0.5;
        
        let uniforms = crate::wgpu_renderer::uniforms::LightingUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            screen_to_world: [screen_w_f / zoom, screen_h_f / zoom],
            camera_pos: [adjusted_x, adjusted_y],
            map_size: [map_w, map_h],
            tile_size: [32.0, 16.0],
            time,
            num_dynamic_lights,
            num_linear_lights,
            disable_shadows: if disable_shadows { 1 } else { 0 },
            ambient_light,
            _padding0: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            _padding4: 0.0,
            _padding5: 0.0,
            _padding6: 0.0,
        };
        
        if self.uniform_buffer.is_none() {
            self.uniform_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lighting Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }));
        } else {
            self.queue.write_buffer(
                self.uniform_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        let scene_tex = self.scene_target.as_ref().unwrap();
        let lightmap_tex = self.lightmap_wgpu.as_ref().unwrap();
        let dynamic_light_tex = self.dynamic_light_texture.as_ref().unwrap();
        let linear_light_tex = self.linear_light_texture.as_ref().unwrap();
        let obstacle_tex = self.obstacle_texture.as_ref().unwrap();

        let pipeline = self.pipeline.as_ref().unwrap();
        
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Lighting Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&scene_tex.view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&scene_tex.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&lightmap_tex.view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&lightmap_tex.sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(&dynamic_light_tex.view),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(&dynamic_light_tex.sampler),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::TextureView(&linear_light_tex.view),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Sampler(&linear_light_tex.sampler),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::TextureView(&obstacle_tex.view),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::Sampler(&obstacle_tex.sampler),
                },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lighting Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.fullscreen_vertices.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            self.fullscreen_indices.as_ref().unwrap().slice(..),
            IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}

