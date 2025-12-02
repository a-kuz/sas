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
use glam::{Mat4, Vec3, Vec4};
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
    camera_pos: vec4<f32>,
    light_pos0: vec4<f32>,
    light_color0: vec4<f32>,
    light_radius0: f32,
    _padding0_0: f32,
    _padding0_1: f32,
    _padding0_2: f32,
    light_pos1: vec4<f32>,
    light_color1: vec4<f32>,
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding1: f32,
    _padding2_0: f32,
    _padding2_1: f32,
    _padding2_2: f32,
    _padding2_3: f32,
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
fn fs_main(input: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    let tex_color = textureSample(model_texture, model_sampler, input.uv);

    var lighting = vec3<f32>(uniforms.ambient_light);

    if (uniforms.num_lights > 0) {
        let light_vec0 = uniforms.light_pos0.xyz - input.world_pos;
        let light_dir0 = normalize(light_vec0);
        let dist0 = length(light_vec0);
        let attenuation0 = pow(1.0 - min(dist0 / uniforms.light_radius0, 1.0), 1.6);
        let ndotl0 = max(dot(input.normal, light_dir0), 0.0);
        
        let view_dir = normalize(uniforms.camera_pos.xyz - input.world_pos);
        let half_dir0 = normalize(light_dir0 + view_dir);
        let spec0 = pow(max(dot(input.normal, half_dir0), 0.0), 32.0);
        
        lighting += uniforms.light_color0.xyz * (ndotl0 + spec0 * 0.3) * attenuation0;
    }

    if (uniforms.num_lights > 1) {
        let light_vec1 = uniforms.light_pos1.xyz - input.world_pos;
        let light_dir1 = normalize(light_vec1);
        let dist1 = length(light_vec1);
        let attenuation1 = pow(1.0 - min(dist1 / uniforms.light_radius1, 1.0), 1.6);
        let ndotl1 = max(dot(input.normal, light_dir1), 0.0);
        
        let view_dir = normalize(uniforms.camera_pos.xyz - input.world_pos);
        let half_dir1 = normalize(light_dir1 + view_dir);
        let spec1 = pow(max(dot(input.normal, half_dir1), 0.0), 32.0);
        
        lighting += uniforms.light_color1.xyz * (ndotl1 + spec1 * 0.3) * attenuation1;
    }

    let final_color = tex_color.rgb * lighting;
    
    if (!is_front) {
        return vec4<f32>(final_color * 0.5, tex_color.a * input.color.a);
    }
    
    return vec4<f32>(final_color, tex_color.a * input.color.a);
}
"#;

const GROUND_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_pos0: vec4<f32>,
    light_color0: vec4<f32>,
    light_radius0: f32,
    light_pos1: vec4<f32>,
    light_color1: vec4<f32>,
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    output.normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let checker = floor(input.world_pos.x * 1.0) + floor(input.world_pos.z * 1.0);
    let base_color = select(vec3<f32>(0.25, 0.25, 0.28), vec3<f32>(0.18, 0.18, 0.2), checker % 2.0 == 0.0);
    
    var lighting = vec3<f32>(uniforms.ambient_light);
    
    if (uniforms.num_lights > 0) {
        let light_vec0 = uniforms.light_pos0.xyz - input.world_pos;
        let light_dir0 = normalize(light_vec0);
        let dist0 = length(light_vec0);
        let attenuation0 = pow(1.0 - min(dist0 / uniforms.light_radius0, 1.0), 1.6);
        let ndotl0 = max(dot(input.normal, light_dir0), 0.0);
        lighting += uniforms.light_color0.xyz * ndotl0 * attenuation0;
    }
    
    if (uniforms.num_lights > 1) {
        let light_vec1 = uniforms.light_pos1.xyz - input.world_pos;
        let light_dir1 = normalize(light_vec1);
        let dist1 = length(light_vec1);
        let attenuation1 = pow(1.0 - min(dist1 / uniforms.light_radius1, 1.0), 1.6);
        let ndotl1 = max(dot(input.normal, light_dir1), 0.0);
        lighting += uniforms.light_color1.xyz * ndotl1 * attenuation1;
    }
    
    return vec4<f32>(base_color * lighting, 1.0);
}
"#;

const SHADOW_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_pos0: vec4<f32>,
    light_color0: vec4<f32>,
    light_radius0: f32,
    _padding0_0: f32,
    _padding0_1: f32,
    _padding0_2: f32,
    light_pos1: vec4<f32>,
    light_color1: vec4<f32>,
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding1: f32,
    _padding2_0: f32,
    _padding2_1: f32,
    _padding2_2: f32,
    _padding2_3: f32,
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
    
    let ground_y = -1.5;
    let light_pos = uniforms.light_pos0.xyz;
    let light_to_vertex = world_pos.xyz - light_pos;
    let t = (ground_y - light_pos.y) / light_to_vertex.y;
    let shadow_pos = light_pos + light_to_vertex * t;
    
    output.clip_position = uniforms.view_proj * vec4<f32>(shadow_pos.x, ground_y + 0.005, shadow_pos.z, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.4);
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
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct MD3Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    light_pos0: [f32; 4],
    light_color0: [f32; 4],
    light_radius0: f32,
    _padding0: [f32; 3],
    light_pos1: [f32; 4],
    light_color1: [f32; 4],
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding1: f32,
    _padding2: [f32; 4],
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
    ground_pipeline: Option<RenderPipeline>,
    shadow_pipeline: Option<RenderPipeline>,
    uniform_buffer: Option<Buffer>,
    bind_group_layout: BindGroupLayout,
    ground_bind_group_layout: BindGroupLayout,
    model_textures: HashMap<String, WgpuTexture>,
    ground_vertex_buffer: Option<Buffer>,
    ground_index_buffer: Option<Buffer>,
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
                        min_binding_size: std::num::NonZeroU64::new(256),
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

        let ground_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Ground Bind Group Layout"),
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
            ],
        });

        Self {
            device,
            queue,
            pipeline: None,
            ground_pipeline: None,
            shadow_pipeline: None,
            uniform_buffer: None,
            bind_group_layout,
            ground_bind_group_layout,
            model_textures: HashMap::new(),
            ground_vertex_buffer: None,
            ground_index_buffer: None,
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
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.pipeline = Some(pipeline);

        let ground_shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Ground Shader"),
            source: ShaderSource::Wgsl(GROUND_SHADER.into()),
        });

        let ground_pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Ground Pipeline Layout"),
            bind_group_layouts: &[&self.ground_bind_group_layout],
            push_constant_ranges: &[],
        });

        let ground_pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Ground Pipeline"),
            layout: Some(&ground_pipeline_layout),
            vertex: VertexState {
                module: &ground_shader,
                entry_point: "vs_main",
                buffers: &[VertexData::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &ground_shader,
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
                front_face: FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.ground_pipeline = Some(ground_pipeline);

        let shadow_shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: ShaderSource::Wgsl(SHADOW_SHADER.into()),
        });

        let shadow_pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[&self.bind_group_layout],
            push_constant_ranges: &[],
        });

        let shadow_pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: VertexState {
                module: &shadow_shader,
                entry_point: "vs_main",
                buffers: &[VertexData::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shadow_shader,
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
                front_face: FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.shadow_pipeline = Some(shadow_pipeline);

        let ground_size = 5.0;
        let ground_y = -1.5;
        let ground_vertices = vec![
            VertexData {
                position: [-ground_size, ground_y, -ground_size],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            VertexData {
                position: [ground_size, ground_y, -ground_size],
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            VertexData {
                position: [ground_size, ground_y, ground_size],
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            VertexData {
                position: [-ground_size, ground_y, ground_size],
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
        ];
        let ground_indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let ground_vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ground Vertex Buffer"),
            contents: bytemuck::cast_slice(&ground_vertices),
            usage: BufferUsages::VERTEX,
        });

        let ground_index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ground Index Buffer"),
            contents: bytemuck::cast_slice(&ground_indices),
            usage: BufferUsages::INDEX,
        });

        self.ground_vertex_buffer = Some(ground_vertex_buffer);
        self.ground_index_buffer = Some(ground_index_buffer);
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
        
        let num_indices = indices.len() as u32;
        
        Some((vertex_buffer, index_buffer, num_indices))
    }

    fn render_ground(
        &mut self,
        encoder: &mut CommandEncoder,
        output_view: &TextureView,
        depth_view: &TextureView,
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
            let uniforms = MD3Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
                light_pos0: [light_pos0.x, light_pos0.y, light_pos0.z, 0.0],
                light_color0: [light_color0.x, light_color0.y, light_color0.z, 0.0],
                light_radius0,
                _padding0: [0.0; 3],
                light_pos1: [light_pos1.x, light_pos1.y, light_pos1.z, 0.0],
                light_color1: [light_color1.x, light_color1.y, light_color1.z, 0.0],
                light_radius1,
                num_lights,
                ambient_light,
                _padding1: 0.0,
            _padding2: [0.0, 0.0, 0.0, 0.0],
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

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Ground Bind Group"),
            layout: &self.ground_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        });

        let pipeline = self.ground_pipeline.as_ref().unwrap();
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Ground Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.ground_vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(self.ground_index_buffer.as_ref().unwrap().slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    fn render_model(
        &mut self,
        encoder: &mut CommandEncoder,
        output_view: &TextureView,
        depth_view: &TextureView,
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
        render_shadow: bool,
    ) {
        if self.pipeline.is_none() {
            self.create_pipeline(surface_format);
        }

        let uniforms = MD3Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: model_matrix.to_cols_array_2d(),
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 0.0],
            light_pos0: [light_pos0.x, light_pos0.y, light_pos0.z, 0.0],
            light_color0: [light_color0.x, light_color0.y, light_color0.z, 0.0],
            light_radius0,
            _padding0: [0.0; 3],
            light_pos1: [light_pos1.x, light_pos1.y, light_pos1.z, 0.0],
            light_color1: [light_color1.x, light_color1.y, light_color1.z, 0.0],
            light_radius1,
            num_lights,
            ambient_light,
            _padding1: 0.0,
            _padding2: [0.0, 0.0, 0.0, 0.0],
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

        for (mesh_idx, _mesh) in model.meshes.iter().enumerate() {
            let (vertex_buffer, index_buffer, num_indices) = match self.create_buffers(model, mesh_idx, frame_idx) {
                Some(buffers) => buffers,
                None => continue,
            };
            
            let texture_path = texture_paths.get(mesh_idx).and_then(|p| p.as_ref().map(|s| s.as_str()));

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
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }

            if render_shadow && texture_to_use.is_some() {
                let shadow_pipeline = self.shadow_pipeline.as_ref().unwrap();
                let texture = texture_to_use.unwrap();
                
                let shadow_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                    label: Some("Shadow Bind Group"),
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

                let mut shadow_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Shadow Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: output_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                shadow_pass.set_pipeline(shadow_pipeline);
                shadow_pass.set_bind_group(0, &shadow_bind_group, &[]);
                shadow_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                shadow_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                shadow_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        }
    }
}

struct MD3TestApp {
    window: Option<Arc<Window>>,
    wgpu_renderer: Option<WgpuRenderer>,
    md3_renderer: Option<MD3Renderer>,
    model: Option<MD3Model>,
    player_lower: Option<MD3Model>,
    player_upper: Option<MD3Model>,
    player_head: Option<MD3Model>,
    weapon: Option<MD3Model>,
    texture_path: String,
    mesh_texture_paths: Vec<Option<String>>,
    depth_texture: Option<Texture>,
    depth_view: Option<TextureView>,
    yaw: f32,
    pitch: f32,
    roll: f32,
    frame_idx: usize,
    auto_rotate: bool,
    start_time: Instant,
    last_fps_update: Instant,
    frame_count: u32,
    fps: f32,
    light0_pos: Vec3,
    light1_pos: Vec3,
    ambient_light: f32,
    num_lights: i32,
}

impl MD3TestApp {
    fn new() -> Self {
        Self {
            window: None,
            wgpu_renderer: None,
            md3_renderer: None,
            model: None,
            player_lower: None,
            player_upper: None,
            player_head: None,
            weapon: None,
            texture_path: String::new(),
            mesh_texture_paths: Vec::new(),
            depth_texture: None,
            depth_view: None,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            frame_idx: 0,
            auto_rotate: true,
            start_time: Instant::now(),
            last_fps_update: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            light0_pos: Vec3::new(2.0, 1.0, 3.0),
            light1_pos: Vec3::new(-2.0, -1.0, 2.0),
            ambient_light: 0.15,
            num_lights: 1,
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
            let mut md3_renderer = MD3Renderer::new(
                wgpu_renderer.device.clone(),
                wgpu_renderer.queue.clone(),
            );

            let lower_paths = vec![
                "q3-resources/models/players/sarge/lower.md3",
                "../q3-resources/models/players/sarge/lower.md3",
            ];
            let upper_paths = vec![
                "q3-resources/models/players/sarge/upper.md3",
                "../q3-resources/models/players/sarge/upper.md3",
            ];
            let head_paths = vec![
                "q3-resources/models/players/sarge/head.md3",
                "../q3-resources/models/players/sarge/head.md3",
            ];
            let weapon_paths = vec![
                "q3-resources/models/weapons2/rocketl/rocketl.md3",
                "../q3-resources/models/weapons2/rocketl/rocketl.md3",
            ];
            
            let lower_path = lower_paths.iter().find(|p| std::path::Path::new(p).exists()).copied();
            let upper_path = upper_paths.iter().find(|p| std::path::Path::new(p).exists()).copied();
            let head_path = head_paths.iter().find(|p| std::path::Path::new(p).exists()).copied();
            let weapon_path = weapon_paths.iter().find(|p| std::path::Path::new(p).exists()).copied();
            
            if let Some(path) = lower_path {
                println!("Loading lower: {}", path);
                let model = MD3Model::load(path).unwrap();
                println!("  Lower: {} meshes, {} frames, {} tags", model.meshes.len(), model.header.num_bone_frames, model.tags.len());
                if !model.tags.is_empty() && !model.tags[0].is_empty() {
                    for tag in &model.tags[0] {
                        let name = std::str::from_utf8(&tag.name).unwrap_or("");
                        println!("    Tag: {}", name.trim_end_matches('\0'));
                    }
                }
                self.player_lower = Some(model);
            }
            if let Some(path) = upper_path {
                println!("Loading upper: {}", path);
                let model = MD3Model::load(path).unwrap();
                println!("  Upper: {} meshes, {} frames, {} tags", model.meshes.len(), model.header.num_bone_frames, model.tags.len());
                if !model.tags.is_empty() && !model.tags[0].is_empty() {
                    for tag in &model.tags[0] {
                        let name = std::str::from_utf8(&tag.name).unwrap_or("");
                        println!("    Tag: {}", name.trim_end_matches('\0'));
                    }
                }
                self.player_upper = Some(model);
            }
            if let Some(path) = head_path {
                println!("Loading head: {}", path);
                let model = MD3Model::load(path).unwrap();
                println!("  Head: {} meshes, {} frames, {} tags", model.meshes.len(), model.header.num_bone_frames, model.tags.len());
                self.player_head = Some(model);
            }
            if let Some(path) = weapon_path {
                println!("Loading weapon: {}", path);
                let model = MD3Model::load(path).unwrap();
                println!("  Weapon: {} meshes, {} frames, {} tags", model.meshes.len(), model.header.num_bone_frames, model.tags.len());
                self.weapon = Some(model);
            }
            
            let model = self.weapon.as_ref().unwrap();
            println!("Model loaded: {} meshes, {} frames", model.meshes.len(), model.header.num_bone_frames);

            let surface_format = wgpu_renderer.surface_config.format;
            md3_renderer.create_pipeline(surface_format);

            let mesh_texture_candidates: Vec<Vec<&str>> = vec![
                vec![
                    "q3-resources/models/weapons2/rocketl/rocketl.jpg",
                    "q3-resources/models/weapons2/rocketl/rocketl.png",
                    "../q3-resources/models/weapons2/rocketl/rocketl.jpg",
                    "../q3-resources/models/weapons2/rocketl/rocketl.png",
                ],
                vec![
                    "q3-resources/models/weapons2/rocketl/rocketl2.jpg",
                    "q3-resources/models/weapons2/rocketl/rocketl2.png",
                    "../q3-resources/models/weapons2/rocketl/rocketl2.jpg",
                    "../q3-resources/models/weapons2/rocketl/rocketl2.png",
                ],
            ];

            let mut mesh_texture_paths = Vec::new();

            for candidates in mesh_texture_candidates.iter().take(model.meshes.len()) {
                let texture_path = candidates
                    .iter()
                    .find(|p| std::path::Path::new(p).exists())
                    .map(|s| s.to_string());

                if let Some(ref path) = texture_path {
                    let texture_data = std::fs::read(path).ok();
                    if let Some(data) = texture_data {
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
                            address_mode_u: wgpu::AddressMode::Repeat,
                            address_mode_v: wgpu::AddressMode::Repeat,
                            address_mode_w: wgpu::AddressMode::Repeat,
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            mipmap_filter: wgpu::FilterMode::Linear,
                            ..Default::default()
                        });

                        let wgpu_tex = WgpuTexture {
                            texture,
                            view,
                            sampler,
                        };

                        md3_renderer.load_texture(path, wgpu_tex);
                    }
                }

                mesh_texture_paths.push(texture_path);
            }

            self.mesh_texture_paths = mesh_texture_paths;
            self.texture_path.clear();
            window.request_redraw();

            self.window = Some(window);
            self.wgpu_renderer = Some(wgpu_renderer);
            self.md3_renderer = Some(md3_renderer);
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
                    let (width, height) = wgpu_renderer.get_viewport_size();
                    let depth_texture = wgpu_renderer.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Depth Texture"),
                        size: wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });
                    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    self.depth_texture = Some(depth_texture);
                    self.depth_view = Some(depth_view);
                }
            }
            WindowEvent::RedrawRequested => {
                self.frame_count += 1;
                let now = Instant::now();
                let elapsed_since_fps_update = now.duration_since(self.last_fps_update).as_secs_f32();
                
                if elapsed_since_fps_update >= 0.5 {
                    self.fps = self.frame_count as f32 / elapsed_since_fps_update;
                    self.frame_count = 0;
                    self.last_fps_update = now;
                    
                    if let Some(ref window) = self.window {
                        window.set_title(&format!("MD3 Test Renderer - WGPU | FPS: {:.1}", self.fps));
                    }
                }
                
                if self.auto_rotate {
                    let elapsed = self.start_time.elapsed().as_secs_f32();
                    self.yaw = elapsed * 0.5;
                }

                if let (Some(ref mut wgpu_renderer), Some(ref mut md3_renderer)) = 
                    (self.wgpu_renderer.as_mut(), self.md3_renderer.as_mut()) {
                    
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
                        let depth_view = self.depth_view.as_ref().unwrap();
                        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: depth_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
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

                    let scale = Mat4::from_scale(Vec3::splat(0.05));
                    let model_matrix = rotation * scale;

                    let camera_pos = Vec3::new(0.0, 0.0, 5.0);
                    let light_pos0 = self.light0_pos;
                    let light_color0 = Vec3::new(3.0, 2.8, 2.6);
                    let light_radius0 = 10.0;
                    let light_pos1 = self.light1_pos;
                    let light_color1 = Vec3::new(1.2, 1.2, 1.8);
                    let light_radius1 = 10.0;

                    let surface_format = wgpu_renderer.surface_config.format;
                    let depth_view = self.depth_view.as_ref().unwrap();

                    let lower_frame = self.frame_idx;
                    let upper_frame = self.frame_idx;
                    
                    let mut upper_matrix = model_matrix;
                    let mut head_matrix = model_matrix;
                    let mut weapon_matrix = model_matrix;
                    
                    if let Some(ref lower) = self.player_lower {
                        let lower_matrix = model_matrix;
                        let frame_idx = lower_frame % lower.header.num_bone_frames as usize;
                        
                        md3_renderer.render_model(
                            &mut encoder,
                            &view,
                            depth_view,
                            surface_format,
                            lower,
                            frame_idx,
                            &vec![None],
                            lower_matrix,
                            view_proj,
                            camera_pos,
                            light_pos0,
                            light_color0,
                            light_radius0,
                            light_pos1,
                            light_color1,
                            light_radius1,
                            self.num_lights,
                            self.ambient_light,
                            false,
                        );
                        
                        if let Some(tags) = lower.tags.get(frame_idx) {
                            if let Some(torso_tag) = tags.iter().find(|t| {
                                let name = std::str::from_utf8(&t.name).unwrap_or("");
                                name.trim_end_matches('\0') == "tag_torso"
                            }) {
                                let tag_pos = Vec3::new(
                                    torso_tag.position[0],
                                    torso_tag.position[1],
                                    torso_tag.position[2],
                                );
                                let tag_mat = Mat4::from_cols(
                                    Vec4::new(torso_tag.axis[0][0], torso_tag.axis[0][1], torso_tag.axis[0][2], 0.0),
                                    Vec4::new(torso_tag.axis[1][0], torso_tag.axis[1][1], torso_tag.axis[1][2], 0.0),
                                    Vec4::new(torso_tag.axis[2][0], torso_tag.axis[2][1], torso_tag.axis[2][2], 0.0),
                                    Vec4::new(tag_pos.x, tag_pos.y, tag_pos.z, 1.0),
                                );
                                upper_matrix = lower_matrix * tag_mat;
                            }
                        }
                    }
                    
                    if let Some(ref upper) = self.player_upper {
                        let frame_idx = upper_frame % upper.header.num_bone_frames as usize;
                        
                        md3_renderer.render_model(
                            &mut encoder,
                            &view,
                            depth_view,
                            surface_format,
                            upper,
                            frame_idx,
                            &vec![None],
                            upper_matrix,
                            view_proj,
                            camera_pos,
                            light_pos0,
                            light_color0,
                            light_radius0,
                            light_pos1,
                            light_color1,
                            light_radius1,
                            self.num_lights,
                            self.ambient_light,
                            false,
                        );
                        
                        if let Some(tags) = upper.tags.get(frame_idx) {
                            if let Some(head_tag) = tags.iter().find(|t| {
                                let name = std::str::from_utf8(&t.name).unwrap_or("");
                                name.trim_end_matches('\0') == "tag_head"
                            }) {
                                let tag_pos = Vec3::new(
                                    head_tag.position[0],
                                    head_tag.position[1],
                                    head_tag.position[2],
                                );
                                let tag_mat = Mat4::from_cols(
                                    Vec4::new(head_tag.axis[0][0], head_tag.axis[0][1], head_tag.axis[0][2], 0.0),
                                    Vec4::new(head_tag.axis[1][0], head_tag.axis[1][1], head_tag.axis[1][2], 0.0),
                                    Vec4::new(head_tag.axis[2][0], head_tag.axis[2][1], head_tag.axis[2][2], 0.0),
                                    Vec4::new(tag_pos.x, tag_pos.y, tag_pos.z, 1.0),
                                );
                                head_matrix = upper_matrix * tag_mat;
                            }
                            
                            if let Some(weapon_tag) = tags.iter().find(|t| {
                                let name = std::str::from_utf8(&t.name).unwrap_or("");
                                name.trim_end_matches('\0') == "tag_weapon"
                            }) {
                                let tag_pos = Vec3::new(
                                    weapon_tag.position[0],
                                    weapon_tag.position[1],
                                    weapon_tag.position[2],
                                );
                                let tag_mat = Mat4::from_cols(
                                    Vec4::new(weapon_tag.axis[0][0], weapon_tag.axis[0][1], weapon_tag.axis[0][2], 0.0),
                                    Vec4::new(weapon_tag.axis[1][0], weapon_tag.axis[1][1], weapon_tag.axis[1][2], 0.0),
                                    Vec4::new(weapon_tag.axis[2][0], weapon_tag.axis[2][1], weapon_tag.axis[2][2], 0.0),
                                    Vec4::new(tag_pos.x, tag_pos.y, tag_pos.z, 1.0),
                                );
                                weapon_matrix = upper_matrix * tag_mat;
                            }
                        }
                    }
                    
                    if let Some(ref head) = self.player_head {
                        md3_renderer.render_model(
                            &mut encoder,
                            &view,
                            depth_view,
                            surface_format,
                            head,
                            0,
                            &vec![None],
                            head_matrix,
                            view_proj,
                            camera_pos,
                            light_pos0,
                            light_color0,
                            light_radius0,
                            light_pos1,
                            light_color1,
                            light_radius1,
                            self.num_lights,
                            self.ambient_light,
                            false,
                        );
                    }
                    
                    if let Some(ref weapon) = self.weapon {
                        let texture_paths_for_render: Vec<Option<String>> =
                            if !self.mesh_texture_paths.is_empty() {
                                self.mesh_texture_paths.clone()
                            } else if !self.texture_path.is_empty() {
                                vec![Some(self.texture_path.clone())]
                            } else {
                                vec![None]
                            };
                        md3_renderer.render_model(
                            &mut encoder,
                            &view,
                            depth_view,
                            surface_format,
                            weapon,
                            self.frame_idx % weapon.header.num_bone_frames as usize,
                            &texture_paths_for_render,
                            weapon_matrix,
                            view_proj,
                            camera_pos,
                            light_pos0,
                            light_color0,
                            light_radius0,
                            light_pos1,
                            light_color1,
                            light_radius1,
                            self.num_lights,
                            self.ambient_light,
                            false,
                        );
                    }

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
                            self.light0_pos = Vec3::new(2.0, 1.0, 3.0);
                            self.light1_pos = Vec3::new(-2.0, -1.0, 2.0);
                            self.ambient_light = 0.15;
                            self.num_lights = 1;
                            println!("Reset: light0={:?}, ambient={}", self.light0_pos, self.ambient_light);
                        }
                        Key::Character(c) if c == "i" || c == "I" => {
                            self.light0_pos.z += 0.2;
                            println!("Light0 pos: {:?}", self.light0_pos);
                        }
                        Key::Character(c) if c == "k" || c == "K" => {
                            self.light0_pos.z -= 0.2;
                            println!("Light0 pos: {:?}", self.light0_pos);
                        }
                        Key::Character(c) if c == "j" || c == "J" => {
                            self.light0_pos.x -= 0.2;
                            println!("Light0 pos: {:?}", self.light0_pos);
                        }
                        Key::Character(c) if c == "l" || c == "L" => {
                            self.light0_pos.x += 0.2;
                            println!("Light0 pos: {:?}", self.light0_pos);
                        }
                        Key::Character(c) if c == "u" || c == "U" => {
                            self.light0_pos.y += 0.2;
                            println!("Light0 pos: {:?}", self.light0_pos);
                        }
                        Key::Character(c) if c == "o" || c == "O" => {
                            self.light0_pos.y -= 0.2;
                            println!("Light0 pos: {:?}", self.light0_pos);
                        }
                        Key::Character(c) if c == "z" || c == "Z" => {
                            self.ambient_light = (self.ambient_light - 0.05).max(0.0);
                            println!("Ambient: {}", self.ambient_light);
                        }
                        Key::Character(c) if c == "x" || c == "X" => {
                            self.ambient_light = (self.ambient_light + 0.05).min(1.0);
                            println!("Ambient: {}", self.ambient_light);
                        }
                        Key::Character(c) if c == "0" => {
                            self.num_lights = 0;
                            println!("Num lights: 0");
                        }
                        Key::Character(c) if c == "1" => {
                            self.num_lights = 1;
                            println!("Num lights: 1");
                        }
                        Key::Character(c) if c == "2" => {
                            self.num_lights = 2;
                            println!("Num lights: 2");
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
