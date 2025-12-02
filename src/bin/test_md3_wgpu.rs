use sas::game::md3::MD3Model;
use sas::wgpu_renderer::{WgpuRenderer, WgpuMD3Renderer};
use sas::wgpu_renderer::texture::WgpuTexture;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
    keyboard::{Key, NamedKey},
};
use std::sync::Arc;
use pollster::FutureExt;
use glam::{Mat4, Vec3};
use std::time::Instant;

struct MD3TestApp {
    window: Option<Arc<Window>>,
    wgpu_renderer: Option<WgpuRenderer>,
    md3_renderer: Option<WgpuMD3Renderer>,
    model: Option<MD3Model>,
    texture_path: String,
    yaw: f32,
    pitch: f32,
    roll: f32,
    frame_idx: usize,
    auto_rotate: bool,
    start_time: Instant,
}

impl MD3TestApp {
    fn new() -> Self {
        Self {
            window: None,
            wgpu_renderer: None,
            md3_renderer: None,
            model: None,
            texture_path: "q3-resources/models/weapons2/machinegun/machinegun.jpg".to_string(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            frame_idx: 0,
            auto_rotate: true,
            start_time: Instant::now(),
        }
    }
}

impl ApplicationHandler for MD3TestApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("MD3 Test Renderer - WGPU")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            
            let wgpu_renderer = WgpuRenderer::new(window.clone()).block_on().unwrap();
            let mut md3_renderer = WgpuMD3Renderer::new(
                wgpu_renderer.device.clone(),
                wgpu_renderer.queue.clone(),
            );

            let model_path = "q3-resources/models/weapons2/machinegun/machinegun.md3";
            println!("Loading MD3 model: {}", model_path);
            let model = MD3Model::load(model_path).expect("Failed to load MD3 model");
            println!("Model loaded: {} meshes, {} frames", model.meshes.len(), model.header.num_bone_frames);

            let surface_format = wgpu_renderer.surface_config.format;
            md3_renderer.create_pipeline(surface_format);

            let texture_data = std::fs::read(&self.texture_path).ok();
            let texture = if let Some(data) = texture_data {
                let img = image::load_from_memory(&data).unwrap().to_rgba8();
                let size = wgpu::Extent3d {
                    width: img.width(),
                    height: img.height(),
                    depth_or_array_layers: 1,
                };
                let texture = wgpu_renderer.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("MD3 Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                
                wgpu_renderer.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &img,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * img.width()),
                        rows_per_image: Some(img.height()),
                    },
                    size,
                );

                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let sampler = wgpu_renderer.device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                });

                WgpuTexture {
                    texture,
                    view,
                    sampler,
                    size: (img.width(), img.height()),
                }
            } else {
                let size = wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                };
                let texture = wgpu_renderer.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Default Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                
                wgpu_renderer.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &[255, 255, 255, 255],
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: Some(1),
                    },
                    size,
                );

                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let sampler = wgpu_renderer.device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                });

                WgpuTexture {
                    texture,
                    view,
                    sampler,
                    size: (1, 1),
                }
            };

            md3_renderer.load_texture(&self.texture_path, texture);
            window.request_redraw();

            self.window = Some(window);
            self.wgpu_renderer = Some(wgpu_renderer);
            self.md3_renderer = Some(md3_renderer);
            self.model = Some(model);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(ref window) = self.window {
            if window.id() != window_id {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut wgpu_renderer) = self.wgpu_renderer {
                    wgpu_renderer.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if self.auto_rotate {
                    let elapsed = self.start_time.elapsed().as_secs_f32();
                    self.yaw = elapsed * 0.5;
                }

                if let (Some(ref mut wgpu_renderer), Some(ref mut md3_renderer), Some(ref model)) = 
                    (self.wgpu_renderer.as_mut(), self.md3_renderer.as_mut(), self.model.as_ref()) {
                    
                    let frame = match wgpu_renderer.begin_frame() {
                        Some(f) => f,
                        None => {
                            if let Some(ref window) = self.window {
                                window.request_redraw();
                            }
                            return;
                        }
                    };
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = wgpu_renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("MD3 Test Encoder"),
                    });

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Clear Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.1,
                                        b: 0.15,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });
                    }

                    let (width, height) = wgpu_renderer.get_viewport_size();
                    let aspect = width as f32 / height as f32;

                    let view_matrix = Mat4::look_at_rh(
                        Vec3::new(0.0, 0.0, 5.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    );

                    let proj_matrix = Mat4::perspective_rh(
                        std::f32::consts::PI / 4.0,
                        aspect,
                        0.1,
                        100.0,
                    );

                    let view_proj = proj_matrix * view_matrix;

                    let rotation_y = Mat4::from_rotation_y(self.yaw);
                    let rotation_x = Mat4::from_rotation_x(self.pitch);
                    let rotation_z = Mat4::from_rotation_z(self.roll);
                    let rotation = rotation_y * rotation_x * rotation_z;

                    let scale = Mat4::from_scale(Vec3::splat(0.01));
                    let model_matrix = rotation * scale;

                    let camera_pos = Vec3::new(0.0, 0.0, 5.0);
                    let light_pos0 = Vec3::new(2.0, 2.0, 2.0);
                    let light_color0 = Vec3::new(1.0, 1.0, 1.0);
                    let light_radius0 = 10.0;
                    let light_pos1 = Vec3::new(-2.0, -2.0, 2.0);
                    let light_color1 = Vec3::new(0.5, 0.5, 0.8);
                    let light_radius1 = 10.0;

                    let surface_format = wgpu_renderer.surface_config.format;
                    md3_renderer.render_model(
                        &mut encoder,
                        &view,
                        surface_format,
                        model,
                        self.frame_idx,
                        &[Some(self.texture_path.clone())],
                        model_matrix,
                        view_proj,
                        camera_pos,
                        light_pos0,
                        light_color0,
                        light_radius0,
                        light_pos1,
                        light_color1,
                        light_radius1,
                        2,
                        0.3,
                    );

                    wgpu_renderer.queue.submit(Some(encoder.finish()));
                    wgpu_renderer.end_frame(frame);
                }

                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    match &event.logical_key {
                        Key::Named(NamedKey::ArrowLeft) => {
                            self.yaw -= 0.1;
                            self.auto_rotate = false;
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            self.yaw += 0.1;
                            self.auto_rotate = false;
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            self.pitch += 0.1;
                            self.auto_rotate = false;
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            self.pitch -= 0.1;
                            self.auto_rotate = false;
                        }
                        Key::Character(c) if c == "q" || c == "Q" => {
                            self.roll -= 0.1;
                            self.auto_rotate = false;
                        }
                        Key::Character(c) if c == "e" || c == "E" => {
                            self.roll += 0.1;
                            self.auto_rotate = false;
                        }
                        Key::Named(NamedKey::Space) => {
                            self.auto_rotate = !self.auto_rotate;
                        }
                        Key::Character(c) if c == "r" || c == "R" => {
                            self.yaw = 0.0;
                            self.pitch = 0.0;
                            self.roll = 0.0;
                        }
                        _ => {}
                    }
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = MD3TestApp::new();
    event_loop.run_app(&mut app).unwrap();
}

