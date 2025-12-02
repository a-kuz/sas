use wgpu::*;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use std::collections::HashMap;
use crate::wgpu_renderer::texture::WgpuTexture;
use crate::wgpu_renderer::mesh::Vertex;
use crate::game::map::Map;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TileUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 2],
    time: f32,
    _padding: f32,
}

pub struct WgpuTileRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Option<wgpu::RenderPipeline>,
    uniform_buffer: Option<Buffer>,
    bind_group_layout: BindGroupLayout,
    tile_textures: HashMap<u16, WgpuTexture>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    fallback_texture: Option<WgpuTexture>,
}

impl WgpuTileRenderer {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Tile Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let fallback_img = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_pixel(64, 64, image::Rgba([200, 200, 200, 255]))
        );
        let fallback_texture = WgpuTexture::from_image(&device, &queue, &fallback_img);
        
        Self {
            device,
            queue,
            pipeline: None,
            uniform_buffer: None,
            bind_group_layout,
            tile_textures: HashMap::new(),
            vertex_buffer: None,
            index_buffer: None,
            fallback_texture: Some(fallback_texture),
        }
    }

    pub fn load_tile_texture(&mut self, texture_id: u16, texture: WgpuTexture) {
        self.tile_textures.insert(texture_id, texture);
    }

    pub fn create_pipeline(&mut self, surface_format: TextureFormat) {
        let shader_source = include_str!("tile_shader.wgsl");
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Tile Shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Tile Pipeline Layout"),
            bind_group_layouts: &[&self.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Tile Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.pipeline = Some(pipeline);
    }

    pub fn render_tiles(
        &mut self,
        encoder: &mut CommandEncoder,
        output_view: &TextureView,
        surface_format: TextureFormat,
        map: &Map,
        camera_x: f32,
        camera_y: f32,
        zoom: f32,
        screen_w: f32,
        screen_h: f32,
        time: f32,
        clear: bool,
    ) {
        if self.pipeline.is_none() {
            self.create_pipeline(surface_format);
        }

        const TILE_WIDTH: f32 = 32.0;
        const TILE_HEIGHT: f32 = 16.0;

        let start_x = ((camera_x / TILE_WIDTH).floor() as i32).max(0);
        let end_x = (((camera_x + screen_w) / TILE_WIDTH).ceil() as i32).min(map.width as i32);
        let start_y = ((camera_y / TILE_HEIGHT).floor() as i32).max(0);
        let end_y = (((camera_y + screen_h) / TILE_HEIGHT).ceil() as i32).min(map.height as i32);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut current_texture_id = None;
        let mut current_index_offset = 0u16;
        
        let mut solid_count = 0;
        let mut missing_texture_count = 0;

        for x in start_x..end_x {
            for y in start_y..end_y {
                let tile = &map.tiles[x as usize][y as usize];
                if !tile.solid {
                    continue;
                }
                
                solid_count += 1;

                if !self.tile_textures.contains_key(&tile.texture_id) {
                    missing_texture_count += 1;
                }

                if current_texture_id != Some(tile.texture_id) {
                    current_texture_id = Some(tile.texture_id);
                }

                let screen_x = x as f32 * TILE_WIDTH;
                let screen_y = y as f32 * TILE_HEIGHT;

                let base_idx = vertices.len() as u16;

                vertices.push(Vertex {
                    position: [screen_x, screen_y, 0.0],
                    uv: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    normal: [0.0, 0.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [screen_x + TILE_WIDTH, screen_y, 0.0],
                    uv: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    normal: [0.0, 0.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [screen_x + TILE_WIDTH, screen_y + TILE_HEIGHT, 0.0],
                    uv: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    normal: [0.0, 0.0, 1.0],
                });
                vertices.push(Vertex {
                    position: [screen_x, screen_y + TILE_HEIGHT, 0.0],
                    uv: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    normal: [0.0, 0.0, 1.0],
                });

                indices.push(base_idx);
                indices.push(base_idx + 1);
                indices.push(base_idx + 2);
                indices.push(base_idx);
                indices.push(base_idx + 2);
                indices.push(base_idx + 3);
            }
        }

        if vertices.is_empty() {
            return;
        }

        let view_proj = glam::Mat4::orthographic_rh(
            camera_x,
            camera_x + screen_w / zoom,
            camera_y + screen_h / zoom,
            camera_y,
            -1.0,
            1.0,
        );

        let uniforms = TileUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [camera_x, camera_y],
            time,
            _padding: 0.0,
        };

        if self.uniform_buffer.is_none() {
            self.uniform_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Tile Uniform Buffer"),
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

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let texture = self.tile_textures.values().next()
            .or(self.fallback_texture.as_ref())
            .expect("No texture available for tile rendering");
            
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tile Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let pipeline = self.pipeline.as_ref().unwrap();
            let load_op = if clear {
                LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                })
            } else {
                LoadOp::Load
        };
        
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Tile Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: load_op,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

