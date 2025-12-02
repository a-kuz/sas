use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;

use wgpu::*;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
    keyboard::{Key, NamedKey},
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use pollster::FutureExt;

const MD3_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    light_pos0: vec3<f32>,
    light_color0: vec3<f32>,
    light_radius0: f32,
    light_pos1: vec3<f32>,
    light_color1: vec3<f32>,
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var model_texture: texture_2d<f32>;

@group(0) @binding(2)
var model_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.uv = input.uv;
    output.color = input.color;
    output.normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.world_pos = world_pos.xyz;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(model_texture, model_sampler, input.uv);
    
    var lighting = vec3<f32>(uniforms.ambient_light);
    
    if (uniforms.num_lights > 0) {
        let light_dir0 = normalize(uniforms.light_pos0 - input.world_pos);
        let dist0 = distance(input.world_pos, uniforms.light_pos0);
        let attenuation0 = pow(1.0 - min(dist0 / uniforms.light_radius0, 1.0), 1.6);
        let ndotl0 = max(dot(input.normal, light_dir0), 0.0);
        lighting += uniforms.light_color0 * ndotl0 * attenuation0;
    }
    
    if (uniforms.num_lights > 1) {
        let light_dir1 = normalize(uniforms.light_pos1 - input.world_pos);
        let dist1 = distance(input.world_pos, uniforms.light_pos1);
        let attenuation1 = pow(1.0 - min(dist1 / uniforms.light_radius1, 1.0), 1.6);
        let ndotl1 = max(dot(input.normal, light_dir1), 0.0);
        lighting += uniforms.light_color1 * ndotl1 * attenuation1;
    }
    
    let final_color = tex_color.rgb * lighting;
    return vec4<f32>(final_color, tex_color.a * input.color.a);
}
"#;

#[repr(C)]
#[derive(Debug, Clone)]
struct MD3Header {
    id: [u8; 4],
    version: i32,
    filename: [u8; 68],
    num_bone_frames: i32,
    num_tags: i32,
    num_meshes: i32,
    num_max_skins: i32,
    header_length: i32,
    tag_start: i32,
    tag_end: i32,
    file_size: i32,
}

#[derive(Debug, Clone)]
struct Tag {
    name: [u8; 64],
    position: [f32; 3],
    axis: [[f32; 3]; 3],
}

#[derive(Debug, Clone)]
struct Triangle {
    vertex: [i32; 3],
}

#[derive(Debug, Clone)]
struct TexCoord {
    coord: [f32; 2],
}

#[derive(Debug, Clone)]
struct Vertex {
    vertex: [i16; 3],
    normal: u16,
}

#[derive(Debug, Clone, Copy)]
struct MeshHeader {
    name: [u8; 68],
    num_mesh_frames: i32,
    num_vertices: i32,
    num_triangles: i32,
    tri_start: i32,
    tex_vector_start: i32,
    vertex_start: i32,
    mesh_size: i32,
}

#[derive(Debug, Clone)]
struct Mesh {
    header: MeshHeader,
    triangles: Vec<Triangle>,
    tex_coords: Vec<TexCoord>,
    vertices: Vec<Vec<Vertex>>,
}

#[derive(Debug, Clone)]
struct MD3Model {
    header: MD3Header,
    tags: Vec<Vec<Tag>>,
    meshes: Vec<Mesh>,
}

impl MD3Model {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let mut header_bytes = [0u8; 108];
        file.read_exact(&mut header_bytes)
            .map_err(|e| format!("Failed to read header: {}", e))?;

        let header = unsafe { std::ptr::read(header_bytes.as_ptr() as *const MD3Header) };

        if &header.id != b"IDP3" {
            return Err("Invalid MD3 file format".to_string());
        }

        for _ in 0..header.num_bone_frames {
            let mut frame_bytes = [0u8; 56];
            file.read_exact(&mut frame_bytes)
                .map_err(|e| format!("Failed to read bone frame: {}", e))?;
        }

        let mut tags = vec![Vec::new(); header.num_bone_frames as usize];
        for frame_idx in 0..header.num_bone_frames as usize {
            for _ in 0..header.num_tags {
                let mut tag_bytes = [0u8; 112];
                file.read_exact(&mut tag_bytes)
                    .map_err(|e| format!("Failed to read tag: {}", e))?;

                let mut name = [0u8; 64];
                name.copy_from_slice(&tag_bytes[0..64]);

                let mut position = [0f32; 3];
                for i in 0..3 {
                    let start = 64 + i * 4;
                    position[i] = f32::from_le_bytes([
                        tag_bytes[start],
                        tag_bytes[start + 1],
                        tag_bytes[start + 2],
                        tag_bytes[start + 3],
                    ]);
                }

                let mut axis = [[0f32; 3]; 3];
                for row in 0..3 {
                    for col in 0..3 {
                        let start = 76 + (row * 3 + col) * 4;
                        axis[row][col] = f32::from_le_bytes([
                            tag_bytes[start],
                            tag_bytes[start + 1],
                            tag_bytes[start + 2],
                            tag_bytes[start + 3],
                        ]);
                    }
                }

                tags[frame_idx].push(Tag {
                    name,
                    position,
                    axis,
                });
            }
        }

        let mut meshes = Vec::with_capacity(header.num_meshes as usize);
        for _ in 0..header.num_meshes {
            let mesh_start =
                file.stream_position()
                    .map_err(|e| format!("Failed to get position: {}", e))? as i64;

            let mut mesh_header_bytes = [0u8; 108];
            file.read_exact(&mut mesh_header_bytes)
                .map_err(|e| format!("Failed to read mesh header: {}", e))?;

            let mut name = [0u8; 68];
            name.copy_from_slice(&mesh_header_bytes[4..72]);
            let num_mesh_frames = i32::from_le_bytes([
                mesh_header_bytes[72],
                mesh_header_bytes[73],
                mesh_header_bytes[74],
                mesh_header_bytes[75],
            ]);
            let num_vertices = i32::from_le_bytes([
                mesh_header_bytes[80],
                mesh_header_bytes[81],
                mesh_header_bytes[82],
                mesh_header_bytes[83],
            ]);
            let num_triangles = i32::from_le_bytes([
                mesh_header_bytes[84],
                mesh_header_bytes[85],
                mesh_header_bytes[86],
                mesh_header_bytes[87],
            ]);
            let tri_start = i32::from_le_bytes([
                mesh_header_bytes[88],
                mesh_header_bytes[89],
                mesh_header_bytes[90],
                mesh_header_bytes[91],
            ]);
            let tex_vector_start = i32::from_le_bytes([
                mesh_header_bytes[96],
                mesh_header_bytes[97],
                mesh_header_bytes[98],
                mesh_header_bytes[99],
            ]);
            let vertex_start = i32::from_le_bytes([
                mesh_header_bytes[100],
                mesh_header_bytes[101],
                mesh_header_bytes[102],
                mesh_header_bytes[103],
            ]);
            let mesh_size = i32::from_le_bytes([
                mesh_header_bytes[104],
                mesh_header_bytes[105],
                mesh_header_bytes[106],
                mesh_header_bytes[107],
            ]);

            let mesh_header = MeshHeader {
                name,
                num_mesh_frames,
                num_vertices,
                num_triangles,
                tri_start,
                tex_vector_start,
                vertex_start,
                mesh_size,
            };

            file.seek(SeekFrom::Start(
                (mesh_start + mesh_header.tri_start as i64) as u64,
            ))
            .map_err(|e| format!("Failed to seek: {}", e))?;

            let mut triangles = Vec::with_capacity(mesh_header.num_triangles as usize);
            for _ in 0..mesh_header.num_triangles {
                let mut tri_bytes = [0u8; 12];
                file.read_exact(&mut tri_bytes)
                    .map_err(|e| format!("Failed to read triangle: {}", e))?;
                let tri = unsafe { std::ptr::read(tri_bytes.as_ptr() as *const Triangle) };
                triangles.push(tri);
            }

            file.seek(SeekFrom::Start(
                (mesh_start + mesh_header.tex_vector_start as i64) as u64,
            ))
            .map_err(|e| format!("Failed to seek: {}", e))?;

            let mut tex_coords = Vec::with_capacity(mesh_header.num_vertices as usize);
            for _ in 0..mesh_header.num_vertices {
                let mut tc_bytes = [0u8; 8];
                file.read_exact(&mut tc_bytes)
                    .map_err(|e| format!("Failed to read tex coord: {}", e))?;
                let tc = unsafe { std::ptr::read(tc_bytes.as_ptr() as *const TexCoord) };
                tex_coords.push(tc);
            }

            file.seek(SeekFrom::Start(
                (mesh_start + mesh_header.vertex_start as i64) as u64,
            ))
            .map_err(|e| format!("Failed to seek: {}", e))?;

            let mut vertices = Vec::with_capacity(mesh_header.num_mesh_frames as usize);
            for _ in 0..mesh_header.num_mesh_frames {
                let mut frame_verts = Vec::with_capacity(mesh_header.num_vertices as usize);
                for _ in 0..mesh_header.num_vertices {
                    let mut vert_bytes = [0u8; 8];
                    file.read_exact(&mut vert_bytes)
                        .map_err(|e| format!("Failed to read vertex: {}", e))?;
                    let vertex = [
                        i16::from_le_bytes([vert_bytes[0], vert_bytes[1]]),
                        i16::from_le_bytes([vert_bytes[2], vert_bytes[3]]),
                        i16::from_le_bytes([vert_bytes[4], vert_bytes[5]]),
                    ];
                    let normal = u16::from_le_bytes([vert_bytes[6], vert_bytes[7]]);
                    frame_verts.push(Vertex { vertex, normal });
                }
                vertices.push(frame_verts);
            }

            meshes.push(Mesh {
                header: mesh_header,
                triangles,
                tex_coords,
                vertices,
            });

            file.seek(SeekFrom::Start(
                (mesh_start + mesh_header.mesh_size as i64) as u64,
            ))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        }

        Ok(MD3Model {
            header,
            tags,
            meshes,
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct VertexData {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
    normal: [f32; 3],
}

impl VertexData {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexData>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>()) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>()) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct MD3Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    light_pos0: [f32; 3],
    light_color0: [f32; 3],
    light_radius0: f32,
    light_pos1: [f32; 3],
    light_color1: [f32; 3],
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding: [f32; 2],
}

struct WgpuTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

struct WgpuRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl WgpuRenderer {
    async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();
        
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())
            .map_err(|e| format!("Failed to create surface: {:?}", e))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface,
            surface_config,
            size,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    fn begin_frame(&mut self) -> Option<SurfaceTexture> {
        self.surface.get_current_texture().ok()
    }

    fn end_frame(&mut self, frame: SurfaceTexture) {
        frame.present();
    }

    fn get_viewport_size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
    }
}

struct MD3Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Option<RenderPipeline>,
    uniform_buffer: Option<Buffer>,
    bind_group_layout: BindGroupLayout,
    model_textures: HashMap<String, WgpuTexture>,
}

impl MD3Renderer {
    fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("MD3 Bind Group Layout"),
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

        Self {
            device,
            queue,
            pipeline: None,
            uniform_buffer: None,
            bind_group_layout,
            model_textures: HashMap::new(),
        }
    }

    fn load_texture(&mut self, path: &str, texture: WgpuTexture) {
        self.model_textures.insert(path.to_string(), texture);
    }

    fn create_pipeline(&mut self, surface_format: TextureFormat) {
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("MD3 Shader"),
            source: ShaderSource::Wgsl(MD3_SHADER.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("MD3 Pipeline Layout"),
            bind_group_layouts: &[&self.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("MD3 Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexData::desc()],
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
                cull_mode: Some(Face::Back),
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

    fn create_buffers(&self, model: &MD3Model, mesh_idx: usize, frame_idx: usize) -> Option<(Buffer, Buffer, u32)> {
        if mesh_idx >= model.meshes.len() {
            return None;
        }
        
        let mesh = &model.meshes[mesh_idx];
        if frame_idx >= mesh.vertices.len() {
            return None;
        }
        
        let frame_vertices = &mesh.vertices[frame_idx];
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for (i, vertex) in frame_vertices.iter().enumerate() {
            let vertex_data = vertex.vertex;
            let x = vertex_data[0] as f32 * (1.0 / 64.0);
            let y = vertex_data[1] as f32 * (1.0 / 64.0);
            let z = vertex_data[2] as f32 * (1.0 / 64.0);
            
            let normal_encoded = vertex.normal;
            let lat = ((normal_encoded >> 8) & 0xFF) as f32 * 2.0 * std::f32::consts::PI / 255.0;
            let lng = (normal_encoded & 0xFF) as f32 * 2.0 * std::f32::consts::PI / 255.0;
            let nx = lat.cos() * lng.sin();
            let ny = lat.sin() * lng.sin();
            let nz = lng.cos();
            
            let tex_coord = if i < mesh.tex_coords.len() {
                mesh.tex_coords[i].coord
            } else {
                [0.0, 0.0]
            };
            
            vertices.push(VertexData {
                position: [x, y, z],
                uv: [tex_coord[0], tex_coord[1]],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [nx, ny, nz],
            });
        }
        
        for triangle in &mesh.triangles {
            indices.push(triangle.vertex[0] as u16);
            indices.push(triangle.vertex[1] as u16);
            indices.push(triangle.vertex[2] as u16);
        }
        
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("MD3 Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("MD3 Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        
        let num_indices = mesh.triangles.len() as u32 * 3;
        
        Some((vertex_buffer, index_buffer, num_indices))
    }

    fn render_model(
        &mut self,
        encoder: &mut CommandEncoder,
        output_view: &TextureView,
        surface_format: TextureFormat,
        model: &MD3Model,
        frame_idx: usize,
        texture_paths: &[Option<String>],
        model_matrix: Mat4,
        view_proj: Mat4,
        camera_pos: Vec3,
        light_pos0: Vec3,
        light_color0: Vec3,
        light_radius0: f32,
        light_pos1: Vec3,
        light_color1: Vec3,
        light_radius1: f32,
        num_lights: i32,
        ambient_light: f32,
    ) {
        if self.pipeline.is_none() {
            self.create_pipeline(surface_format);
        }

        for (mesh_idx, _mesh) in model.meshes.iter().enumerate() {
            let (vertex_buffer, index_buffer, num_indices) = match self.create_buffers(model, mesh_idx, frame_idx) {
                Some(buffers) => buffers,
                None => continue,
            };
            
            let texture_path = texture_paths.get(mesh_idx).and_then(|p| p.as_ref().map(|s| s.as_str()));

            let uniforms = MD3Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: model_matrix.to_cols_array_2d(),
                camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z],
                light_pos0: [light_pos0.x, light_pos0.y, light_pos0.z],
                light_color0: [light_color0.x, light_color0.y, light_color0.z],
                light_radius0,
                light_pos1: [light_pos1.x, light_pos1.y, light_pos1.z],
                light_color1: [light_color1.x, light_color1.y, light_color1.z],
                light_radius1,
                num_lights,
                ambient_light,
                _padding: [0.0; 2],
            };

            if self.uniform_buffer.is_none() {
                self.uniform_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("MD3 Uniform Buffer"),
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

            let texture = texture_path.and_then(|path| self.model_textures.get(path));
            let default_texture = self.model_textures.values().next();
            let texture_to_use = texture.or(default_texture);

            if let Some(texture) = texture_to_use {
                let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                    label: Some("MD3 Bind Group"),
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
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("MD3 Render Pass"),
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

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        }
    }
}

struct MD3TestApp {
    window: Option<Arc<Window>>,
    wgpu_renderer: Option<WgpuRenderer>,
    md3_renderer: Option<MD3Renderer>,
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
            let md3_renderer = MD3Renderer::new(
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

