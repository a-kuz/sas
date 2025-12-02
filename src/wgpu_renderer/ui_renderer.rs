use wgpu::*;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct UIVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl UIVertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct UITextVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

impl UITextVertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<UITextVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct UIRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pub pipeline: Option<RenderPipeline>,
    pub text_pipeline: Option<RenderPipeline>,
    pub font_texture: Option<Arc<crate::wgpu_renderer::texture::WgpuTexture>>,
    pub text_bind_group_layout: Option<Arc<BindGroupLayout>>,
}

impl UIRenderer {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, surface_format: TextureFormat) -> Self {
        let mut renderer = Self {
            device,
            queue,
            pipeline: None,
            text_pipeline: None,
            font_texture: None,
            text_bind_group_layout: None,
        };
        renderer.create_pipeline(surface_format);
        renderer
    }

    pub fn set_font_texture(&mut self, texture: Arc<crate::wgpu_renderer::texture::WgpuTexture>) {
        self.font_texture = Some(texture);
    }

    fn create_pipeline(&mut self, surface_format: TextureFormat) {
        let shader_source = include_str!("ui_shader.wgsl");
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[UIVertex::desc()],
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

        let text_shader_source = include_str!("ui_text_shader.wgsl");
        let text_shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("UI Text Shader"),
            source: ShaderSource::Wgsl(text_shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("UI Text Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let text_pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("UI Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let text_pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("UI Text Pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: VertexState {
                module: &text_shader,
                entry_point: "vs_main",
                buffers: &[UITextVertex::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &text_shader,
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

        self.text_pipeline = Some(text_pipeline);
        self.text_bind_group_layout = Some(Arc::new(bind_group_layout));
    }

    pub fn create_rect_buffers(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        screen_width: f32,
        screen_height: f32,
        color: [f32; 4],
    ) -> (Buffer, Buffer) {
        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (width / screen_width) * 2.0;
        let ndc_h = (height / screen_height) * 2.0;

        let vertices = [
            UIVertex { position: [ndc_x, ndc_y], color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y], color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color },
            UIVertex { position: [ndc_x, ndc_y - ndc_h], color },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn create_text_char_buffers(
        &self,
        x: f32,
        y: f32,
        char_size: f32,
        screen_width: f32,
        screen_height: f32,
        ch: u8,
        color: [f32; 4],
    ) -> (Buffer, Buffer) {
        let char_w = char_size;
        let char_h = char_size;

        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (char_w / screen_width) * 2.0;
        let ndc_h = (char_h / screen_height) * 2.0;

        let row = (ch >> 4) as f32;
        let col = (ch & 15) as f32;
        let tex_size = 16.0;
        let u0 = col / tex_size;
        let v0 = row / tex_size;
        let u1 = (col + 1.0) / tex_size;
        let v1 = (row + 1.0) / tex_size;

        let vertices = [
            UITextVertex {
                position: [ndc_x, ndc_y],
                tex_coords: [u0, v0],
                color,
            },
            UITextVertex {
                position: [ndc_x + ndc_w, ndc_y],
                tex_coords: [u1, v0],
                color,
            },
            UITextVertex {
                position: [ndc_x + ndc_w, ndc_y - ndc_h],
                tex_coords: [u1, v1],
                color,
            },
            UITextVertex {
                position: [ndc_x, ndc_y - ndc_h],
                tex_coords: [u0, v1],
                color,
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Text Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Text Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn create_prop_char_buffers(
        &self,
        x: f32,
        y: f32,
        size: f32,
        screen_width: f32,
        screen_height: f32,
        ch: char,
        color: [f32; 4],
    ) -> (Buffer, Buffer, f32) {
        const PROPB_HEIGHT: f32 = 36.0;
        const PROPB_SPACE_WIDTH: f32 = 12.0;
        const PROPB_GAP_WIDTH: f32 = 4.0;
        const PROPB_MAP: [(u16, u16, u16); 26] = [
            (11, 12, 33),
            (49, 12, 31),
            (85, 12, 31),
            (120, 12, 30),
            (156, 12, 21),
            (183, 12, 21),
            (207, 12, 32),
            (13, 55, 30),
            (49, 55, 13),
            (66, 55, 29),
            (101, 55, 31),
            (135, 55, 21),
            (158, 55, 40),
            (204, 55, 32),
            (12, 97, 31),
            (48, 97, 31),
            (82, 97, 30),
            (118, 97, 30),
            (153, 97, 30),
            (185, 97, 25),
            (213, 97, 30),
            (11, 139, 32),
            (42, 139, 51),
            (93, 139, 32),
            (126, 139, 31),
            (158, 139, 25),
        ];

        let upper = ch.to_ascii_uppercase();
        let size_scale = size / PROPB_HEIGHT;
        
        if upper == ' ' {
            let advance = (PROPB_SPACE_WIDTH + PROPB_GAP_WIDTH) * size_scale;
            let empty_vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Vertex Buffer"),
                size: 0,
                usage: BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            let empty_index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Index Buffer"),
                size: 0,
                usage: BufferUsages::INDEX,
                mapped_at_creation: false,
            });
            return (empty_vertex_buffer, empty_index_buffer, advance);
        }
        
        if !('A'..='Z').contains(&upper) {
            let advance = size.round();
            let empty_vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Vertex Buffer"),
                size: 0,
                usage: BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            let empty_index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Index Buffer"),
                size: 0,
                usage: BufferUsages::INDEX,
                mapped_at_creation: false,
            });
            return (empty_vertex_buffer, empty_index_buffer, advance);
        }
        
        let idx = (upper as u8 - b'A') as usize;
        let (sx, sy, w) = PROPB_MAP[idx];
        let aw = (w as f32) * size_scale;
        let ah = PROPB_HEIGHT * size_scale;
        
        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (aw / screen_width) * 2.0;
        let ndc_h = (ah / screen_height) * 2.0;

        let tex_width = 256.0;
        let tex_height = 256.0;
        let inset = 0.5;
        let u0 = ((sx as f32 + inset) / tex_width) as f32;
        let v0 = ((sy as f32 + inset) / tex_height) as f32;
        let u1 = ((sx as f32 + w as f32 - inset) / tex_width) as f32;
        let v1 = ((sy as f32 + PROPB_HEIGHT - inset) / tex_height) as f32;

        let vertices = [
            UITextVertex {
                position: [ndc_x, ndc_y],
                tex_coords: [u0, v0],
                color,
            },
            UITextVertex {
                position: [ndc_x + ndc_w, ndc_y],
                tex_coords: [u1, v0],
                color,
            },
            UITextVertex {
                position: [ndc_x + ndc_w, ndc_y - ndc_h],
                tex_coords: [u1, v1],
                color,
            },
            UITextVertex {
                position: [ndc_x, ndc_y - ndc_h],
                tex_coords: [u0, v1],
                color,
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Prop Text Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Prop Text Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let advance = (aw + PROPB_GAP_WIDTH * size_scale).round();
        (vertex_buffer, index_buffer, advance)
    }
}
