use macroquad::prelude::*;
use crate::game::md3::Mesh;
use crate::game::shader;
use crate::game::map::LightSource;
use std::sync::OnceLock;
use std::collections::HashMap;
use crate::count_shader;
use std::cell::RefCell;

thread_local! {
    static GLOBAL_BATCH: RefCell<MD3Batch> = RefCell::new(MD3Batch::new());
    static BATCH_ENABLED: RefCell<bool> = RefCell::new(false);
}

pub fn enable_batching() {
    BATCH_ENABLED.with(|enabled| *enabled.borrow_mut() = true);
}

pub fn disable_batching() {
    BATCH_ENABLED.with(|enabled| *enabled.borrow_mut() = false);
}

pub fn flush_batch(lighting_context: Option<&LightingContext>) {
    GLOBAL_BATCH.with(|batch| {
        batch.borrow_mut().flush(lighting_context);
    });
}

pub struct MD3BatchItem {
    pub mesh: *const Mesh,
    pub frame_idx: usize,
    pub screen_x: f32,
    pub screen_y: f32,
    pub scale: f32,
    pub color: Color,
    pub texture: Option<Texture2D>,
    pub flip_x: bool,
    pub pitch_angle: f32,
    pub yaw_angle: f32,
    pub roll_angle: f32,
}

pub struct MD3Batch {
    items: Vec<MD3BatchItem>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl MD3Batch {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            vertices: Vec::with_capacity(10000),
            indices: Vec::with_capacity(5000),
        }
    }

    pub fn add_mesh(
        &mut self,
        mesh: &Mesh,
        frame_idx: usize,
        screen_x: f32,
        screen_y: f32,
        scale: f32,
        color: Color,
        texture: Option<Texture2D>,
        flip_x: bool,
        pitch_angle: f32,
        yaw_angle: f32,
        roll_angle: f32,
    ) {
        self.items.push(MD3BatchItem {
            mesh: mesh as *const Mesh,
            frame_idx,
            screen_x,
            screen_y,
            scale,
            color,
            texture,
            flip_x,
            pitch_angle,
            yaw_angle,
            roll_angle,
        });
    }

    pub fn flush(&mut self, lighting_context: Option<&LightingContext>) {
        if self.items.is_empty() {
            return;
        }

        self.vertices.clear();
        self.indices.clear();

        let mut texture_to_use: Option<Texture2D> = None;
        let mut items_to_process = Vec::new();
        std::mem::swap(&mut items_to_process, &mut self.items);

        for item in &items_to_process {
            let mesh = unsafe { &*item.mesh };
            
            if item.frame_idx >= mesh.vertices.len() {
                continue;
            }

            if texture_to_use.is_none() && item.texture.is_some() {
                texture_to_use = item.texture.clone();
            }

            let frame_verts = &mesh.vertices[item.frame_idx];
            if frame_verts.is_empty() || mesh.triangles.is_empty() {
                continue;
            }

            let x_mult = if item.flip_x { -1.0 } else { 1.0 };

            let pitch = item.pitch_angle.clamp(-0.15, 0.15);
            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let cos_y = item.yaw_angle.cos();
            let sin_y = item.yaw_angle.sin();
            let cos_r = item.roll_angle.cos();
            let sin_r = item.roll_angle.sin();

            for triangle in &mesh.triangles {
                let v0_idx = triangle.vertex[0] as usize;
                let v1_idx = triangle.vertex[1] as usize;
                let v2_idx = triangle.vertex[2] as usize;

                if v0_idx >= frame_verts.len() || 
                   v1_idx >= frame_verts.len() || 
                   v2_idx >= frame_verts.len() ||
                   v0_idx >= mesh.tex_coords.len() ||
                   v1_idx >= mesh.tex_coords.len() ||
                   v2_idx >= mesh.tex_coords.len() {
                    continue;
                }

                let v0 = &frame_verts[v0_idx];
                let v1 = &frame_verts[v1_idx];
                let v2 = &frame_verts[v2_idx];

                let process_vertex = |v: &super::md3::Vertex| -> (Vec3, Vec3) {
                    let vx = v.vertex[0] as f32 * item.scale / 64.0 * x_mult;
                    let vy = v.vertex[1] as f32 * item.scale / 64.0;
                    let vz = v.vertex[2] as f32 * item.scale / 64.0;

                    let ry = vy * cos_p + vz * sin_p;
                    let rz = -vy * sin_p + vz * cos_p;

                    let yx = vx * cos_y - ry * sin_y;
                    let yy = vx * sin_y + ry * cos_y;

                    let rx = yx * cos_r - rz * sin_r;
                    let rr = yx * sin_r + rz * cos_r;

                    let pos = Vec3::new(
                        item.screen_x + rx,
                        item.screen_y - rr,
                        0.0
                    );

                    let n = decode_md3_normal(v.normal);
                    let nx = n.x * x_mult;
                    let ny = n.y;
                    let nz = n.z;

                    let n_ry = ny * cos_p + nz * sin_p;
                    let n_rz = -ny * sin_p + nz * cos_p;

                    let n_yx = nx * cos_y - n_ry * sin_y;
                    let n_yy = nx * sin_y + n_ry * cos_y;
                    let n_yz = n_rz;

                    let normal = Vec3::new(n_yx, n_yy, n_yz).normalize();

                    (pos, normal)
                };

                let (pos0, norm0) = process_vertex(v0);
                let (pos1, norm1) = process_vertex(v1);
                let (pos2, norm2) = process_vertex(v2);

                let tc0 = &mesh.tex_coords[v0_idx];
                let tc1 = &mesh.tex_coords[v1_idx];
                let tc2 = &mesh.tex_coords[v2_idx];

                let base = self.vertices.len();
                
                if base + 3 > 9000 {
                    self.draw_current_batch(&texture_to_use, lighting_context);
                }
                
                let base = self.vertices.len() as u16;
                
                self.vertices.push(Vertex {
                    position: pos0,
                    uv: Vec2::new(tc0.coord[0], tc0.coord[1]),
                    color: [
                        (item.color.r * 255.0) as u8,
                        (item.color.g * 255.0) as u8,
                        (item.color.b * 255.0) as u8,
                        (item.color.a * 255.0) as u8,
                    ],
                    normal: Vec4::new(norm0.x, norm0.y, norm0.z, 0.0),
                });
                
                self.vertices.push(Vertex {
                    position: pos1,
                    uv: Vec2::new(tc1.coord[0], tc1.coord[1]),
                    color: [
                        (item.color.r * 255.0) as u8,
                        (item.color.g * 255.0) as u8,
                        (item.color.b * 255.0) as u8,
                        (item.color.a * 255.0) as u8,
                    ],
                    normal: Vec4::new(norm1.x, norm1.y, norm1.z, 0.0),
                });
                
                self.vertices.push(Vertex {
                    position: pos2,
                    uv: Vec2::new(tc2.coord[0], tc2.coord[1]),
                    color: [
                        (item.color.r * 255.0) as u8,
                        (item.color.g * 255.0) as u8,
                        (item.color.b * 255.0) as u8,
                        (item.color.a * 255.0) as u8,
                    ],
                    normal: Vec4::new(norm2.x, norm2.y, norm2.z, 0.0),
                });

                self.indices.push(base);
                self.indices.push(base + 1);
                self.indices.push(base + 2);
            }
        }

        self.draw_current_batch(&texture_to_use, lighting_context);
        self.items.clear();
    }
    
    fn draw_current_batch(&mut self, texture_to_use: &Option<Texture2D>, lighting_context: Option<&LightingContext>) {
        if self.vertices.is_empty() {
            return;
        }
        
        if let Some(tex) = texture_to_use {
            let vertices = std::mem::take(&mut self.vertices);
            let indices = std::mem::take(&mut self.indices);
            
            let mesh_data = macroquad::models::Mesh {
                vertices,
                indices,
                texture: Some(tex.clone()),
            };

            if let Some(ctx) = lighting_context {
                let material = get_model_lit_material();
                gl_use_material(material);

                let avg_x = if !self.items.is_empty() {
                    self.items.iter().map(|i| i.screen_x).sum::<f32>() / self.items.len() as f32
                } else {
                    0.0
                };
                let avg_y = if !self.items.is_empty() {
                    self.items.iter().map(|i| i.screen_y).sum::<f32>() / self.items.len() as f32
                } else {
                    0.0
                };
                
                apply_lighting_uniforms(material, ctx, avg_x + ctx.camera_x, avg_y + ctx.camera_y);
                draw_mesh(&mesh_data);
                count_shader!("md3_batched_lit");
                gl_use_default_material();
            } else {
                draw_mesh(&mesh_data);
                count_shader!("md3_batched");
            }
        }
    }
}

static MODEL_LIT_MATERIAL: OnceLock<Material> = OnceLock::new();

fn get_model_lit_material() -> &'static Material {
    MODEL_LIT_MATERIAL.get_or_init(|| shader::create_model_lit_material())
}

fn decode_md3_normal(n: u16) -> Vec3 {
    let lat = ((n >> 8) & 0xff) as f32 * (std::f32::consts::TAU / 256.0);
    let lng = (n & 0xff) as f32 * (std::f32::consts::TAU / 256.0);
    let x = lat.cos() * lng.sin();
    let y = lat.sin() * lng.sin();
    let z = lng.cos();
    vec3(x, y, z)
}

pub struct LightingContext {
    pub lights: Vec<LightSource>,
    pub ambient: f32,
    pub camera_x: f32,
    pub camera_y: f32,
}

struct CachedLighting {
    light_indices: [usize; 2],
    num_lights: i32,
    frame: u64,
}

static mut LIGHT_CACHE: OnceLock<HashMap<u64, CachedLighting>> = OnceLock::new();
static mut LIGHT_CACHE_FRAME: u64 = 0;

fn get_light_cache() -> &'static mut HashMap<u64, CachedLighting> {
    unsafe {
        if LIGHT_CACHE.get().is_none() {
            let _ = LIGHT_CACHE.set(HashMap::with_capacity(256));
        }
        LIGHT_CACHE.get_mut().unwrap()
    }
}

pub fn clear_light_cache() {
    unsafe {
        LIGHT_CACHE_FRAME += 1;
        if LIGHT_CACHE_FRAME % 300 == 0 {
            if let Some(cache) = LIGHT_CACHE.get_mut() {
                if cache.len() > 1024 {
                    cache.retain(|_, v| v.frame > LIGHT_CACHE_FRAME - 60);
                }
            }
        }
    }
}

pub fn render_md3_mesh_batched(
    batch: &mut MD3Batch,
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
) {
    batch.add_mesh(
        mesh,
        frame_idx,
        screen_x,
        screen_y,
        scale,
        color,
        texture.cloned(),
        flip_x,
        pitch_angle,
        yaw_angle,
        0.0,
    );
}

pub fn render_md3_mesh_batched_with_roll(
    batch: &mut MD3Batch,
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    roll_angle: f32,
) {
    batch.add_mesh(
        mesh,
        frame_idx,
        screen_x,
        screen_y,
        scale,
        color,
        texture.cloned(),
        flip_x,
        pitch_angle,
        yaw_angle,
        roll_angle,
    );
}

fn find_closest_lights(ctx: &LightingContext, world_x: f32, world_y: f32) -> ([usize; 2], i32) {
    let grid_x = (world_x / 128.0) as i32;
    let grid_y = (world_y / 128.0) as i32;
    let cache_key = ((grid_x as u64) << 32) | (grid_y as u64 & 0xFFFFFFFF);
    
    let current_frame = unsafe { LIGHT_CACHE_FRAME };
    let cache = get_light_cache();
    
    if let Some(cached) = cache.get(&cache_key) {
        if cached.frame == current_frame {
            return (cached.light_indices, cached.num_lights);
        }
    }
    
    let mut closest: [(usize, f32); 2] = [(0, f32::MAX), (0, f32::MAX)];
    
    for (idx, light) in ctx.lights.iter().enumerate() {
        let dx = light.x - world_x;
        let dy = light.y - world_y;
        let dist_sq = dx * dx + dy * dy;
        
        if dist_sq < closest[0].1 {
            closest[1] = closest[0];
            closest[0] = (idx, dist_sq);
        } else if dist_sq < closest[1].1 {
            closest[1] = (idx, dist_sq);
        }
    }
    
    let mut num_lights = 0;
    let mut indices = [0usize, 0usize];
    
    for i in 0..2 {
        if closest[i].1 < f32::MAX {
            indices[i] = closest[i].0;
            num_lights += 1;
        }
    }
    
    cache.insert(cache_key, CachedLighting {
        light_indices: indices,
        num_lights,
        frame: current_frame,
    });
    
    (indices, num_lights)
}

fn apply_lighting_uniforms(material: &Material, ctx: &LightingContext, world_x: f32, world_y: f32) {
    material.set_uniform("cameraPos", (ctx.camera_x, ctx.camera_y));
    material.set_uniform("ambientLight", ctx.ambient);
    
    let (indices, num_lights) = find_closest_lights(ctx, world_x, world_y);
    
    for i in 0..2 {
        if i < num_lights as usize {
            let light = &ctx.lights[indices[i]];
            material.set_uniform(&format!("lightPos{}", i), (light.x, light.y, 0.0f32));
            material.set_uniform(&format!("lightColor{}", i), 
                (light.r as f32 / 255.0, light.g as f32 / 255.0, light.b as f32 / 255.0));
            material.set_uniform(&format!("lightRadius{}", i), light.radius);
        } else {
            material.set_uniform(&format!("lightPos{}", i), (0.0f32, 0.0f32, 0.0f32));
            material.set_uniform(&format!("lightColor{}", i), (0.0f32, 0.0f32, 0.0f32));
            material.set_uniform(&format!("lightRadius{}", i), 0.0f32);
        }
    }
    material.set_uniform("numLights", num_lights);
}

pub fn render_md3_mesh(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    pitch_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_with_yaw(mesh, frame_idx, screen_x, screen_y, scale, color, texture, flip_x, pitch_angle, 0.0, lighting_context);
}

pub fn render_md3_mesh_with_yaw_ex(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_internal(mesh, frame_idx, screen_x, screen_y, scale, color, texture, texture_path, None, flip_x, pitch_angle, yaw_angle, 0.0, lighting_context, false);
}

pub fn render_md3_mesh_with_yaw_ex_shader(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    shader_textures: Option<&[Texture2D]>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_internal(mesh, frame_idx, screen_x, screen_y, scale, color, texture, texture_path, shader_textures, flip_x, pitch_angle, yaw_angle, 0.0, lighting_context, false);
}

pub fn render_md3_mesh_with_yaw_and_roll(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    roll_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_internal(mesh, frame_idx, screen_x, screen_y, scale, color, texture, texture_path, None, flip_x, pitch_angle, yaw_angle, roll_angle, lighting_context, false);
}

pub fn render_md3_mesh_with_yaw_and_roll_shader(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    shader_textures: Option<&[Texture2D]>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    roll_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_internal(mesh, frame_idx, screen_x, screen_y, scale, color, texture, texture_path, shader_textures, flip_x, pitch_angle, yaw_angle, roll_angle, lighting_context, false);
}

pub fn render_md3_mesh_with_yaw_and_roll_quad(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    roll_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_internal(mesh, frame_idx, screen_x, screen_y, scale, color, texture, texture_path, None, flip_x, pitch_angle, yaw_angle, roll_angle, lighting_context, true);
}

pub fn render_md3_mesh_with_yaw(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    lighting_context: Option<&LightingContext>,
) {
    render_md3_mesh_internal(mesh, frame_idx, screen_x, screen_y, scale, color, texture, None, None, flip_x, pitch_angle, yaw_angle, 0.0, lighting_context, false);
}

fn render_md3_mesh_internal(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    shader_textures: Option<&[Texture2D]>,
    flip_x: bool,
    pitch_angle: f32,
    yaw_angle: f32,
    roll_angle: f32,
    lighting_context: Option<&LightingContext>,
    use_quad_shader: bool,
) {
    if mesh.vertices.len() <= frame_idx {
        return;
    }
    
    let frame_verts = &mesh.vertices[frame_idx];
    if frame_verts.is_empty() || mesh.triangles.is_empty() {
        return;
    }
    
    let x_mult = if flip_x { -1.0 } else { 1.0 };
    
    let mut all_vertices: Vec<Vertex> = Vec::with_capacity(mesh.triangles.len() * 3);
    let mut all_indices: Vec<u16> = Vec::with_capacity(mesh.triangles.len() * 3);
    
    for triangle in &mesh.triangles {
        let v0_idx = triangle.vertex[0] as usize;
        let v1_idx = triangle.vertex[1] as usize;
        let v2_idx = triangle.vertex[2] as usize;
        
        if v0_idx >= frame_verts.len() || 
           v1_idx >= frame_verts.len() || 
           v2_idx >= frame_verts.len() ||
           v0_idx >= mesh.tex_coords.len() ||
           v1_idx >= mesh.tex_coords.len() ||
           v2_idx >= mesh.tex_coords.len() {
            continue;
        }
        
        let v0 = &frame_verts[v0_idx];
        let v1 = &frame_verts[v1_idx];
        let v2 = &frame_verts[v2_idx];
        
        let v0_x = v0.vertex[0] as f32 * scale / 64.0 * x_mult;
        let v0_y = v0.vertex[1] as f32 * scale / 64.0;
        let v0_z = v0.vertex[2] as f32 * scale / 64.0;
        
        let v1_x = v1.vertex[0] as f32 * scale / 64.0 * x_mult;
        let v1_y = v1.vertex[1] as f32 * scale / 64.0;
        let v1_z = v1.vertex[2] as f32 * scale / 64.0;
        
        let v2_x = v2.vertex[0] as f32 * scale / 64.0 * x_mult;
        let v2_y = v2.vertex[1] as f32 * scale / 64.0;
        let v2_z = v2.vertex[2] as f32 * scale / 64.0;
        
        let pitch = pitch_angle.clamp(-0.15, 0.15);
        let cos_p = pitch.cos();
        let sin_p = pitch.sin();
        
        let ry0 = v0_y * cos_p + v0_z * sin_p;
        let rz0 = -v0_y * sin_p + v0_z * cos_p;
        
        let ry1 = v1_y * cos_p + v1_z * sin_p;
        let rz1 = -v1_y * sin_p + v1_z * cos_p;
        
        let ry2 = v2_y * cos_p + v2_z * sin_p;
        let rz2 = -v2_y * sin_p + v2_z * cos_p;
        
        let cos_y = yaw_angle.cos();
        let sin_y = yaw_angle.sin();
        
        let yx0 = v0_x * cos_y - ry0 * sin_y;
        let yy0 = v0_x * sin_y + ry0 * cos_y;
        let yx1 = v1_x * cos_y - ry1 * sin_y;
        let yy1 = v1_x * sin_y + ry1 * cos_y;
        let yx2 = v2_x * cos_y - ry2 * sin_y;
        let yy2 = v2_x * sin_y + ry2 * cos_y;
        
        let cos_r = roll_angle.cos();
        let sin_r = roll_angle.sin();
        
        let rx0 = yx0 * cos_r - rz0 * sin_r;
        let rr0 = yx0 * sin_r + rz0 * cos_r;
        let rx1 = yx1 * cos_r - rz1 * sin_r;
        let rr1 = yx1 * sin_r + rz1 * cos_r;
        let rx2 = yx2 * cos_r - rz2 * sin_r;
        let rr2 = yx2 * sin_r + rz2 * cos_r;
        
        let x0 = screen_x + rx0;
        let y0 = screen_y - rr0;
        let x1 = screen_x + rx1;
        let y1 = screen_y - rr1;
        let x2 = screen_x + rx2;
        let y2 = screen_y - rr2;
        
        if let Some(_tex) = texture {
            let tc0 = &mesh.tex_coords[v0_idx];
            let tc1 = &mesh.tex_coords[v1_idx];
            let tc2 = &mesh.tex_coords[v2_idx];
            
            let transform_normal = |n_raw: u16| -> Vec3 {
                let n = decode_md3_normal(n_raw);
                let nx = n.x * x_mult;
                let ny = n.y;
                let nz = n.z;

                let n_ry = ny * cos_p + nz * sin_p;
                let n_rz = -ny * sin_p + nz * cos_p;

                let n_yx = nx * cos_y - n_ry * sin_y;
                let n_yy = nx * sin_y + n_ry * cos_y;
                let n_yz = n_rz;

                Vec3::new(n_yx, n_yy, n_yz).normalize()
            };

            let n0 = transform_normal(v0.normal);
            let n1 = transform_normal(v1.normal);
            let n2 = transform_normal(v2.normal);

            let base = all_vertices.len() as u16;
            all_vertices.push(Vertex { position: Vec3::new(x0, y0, 0.0), uv: Vec2::new(tc0.coord[0], tc0.coord[1]), color: [ (color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8, (color.a * 255.0) as u8 ], normal: Vec4::new(n0.x, n0.y, n0.z, 0.0) });
            all_vertices.push(Vertex { position: Vec3::new(x1, y1, 0.0), uv: Vec2::new(tc1.coord[0], tc1.coord[1]), color: [ (color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8, (color.a * 255.0) as u8 ], normal: Vec4::new(n1.x, n1.y, n1.z, 0.0) });
            all_vertices.push(Vertex { position: Vec3::new(x2, y2, 0.0), uv: Vec2::new(tc2.coord[0], tc2.coord[1]), color: [ (color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8, (color.a * 255.0) as u8 ], normal: Vec4::new(n2.x, n2.y, n2.z, 0.0) });
            all_indices.push(base);
            all_indices.push(base + 1);
            all_indices.push(base + 2);
        } else {
            // no texture path: draw immediate
            draw_triangle(Vec2::new(x0, y0), Vec2::new(x1, y1), Vec2::new(x2, y2), color);
        }
    }

    if let Some(tex) = texture {
        if !all_vertices.is_empty() {
            let mesh_data = macroquad::models::Mesh { vertices: all_vertices, indices: all_indices, texture: Some(tex.clone()) };
            
            if use_quad_shader {
                let material = shader::create_quad_damage_outline_material();
                gl_use_material(material);
                material.set_uniform("time", get_time() as f32);
                material.set_uniform("outlineWidth", 2.5f32);
                draw_mesh(&mesh_data);
                count_shader!("md3_quad_damage");
                gl_use_default_material();
            } else {
                let mut shader_applied = false;
                
                if let Some(path) = texture_path {
                    let path_lower = path.to_lowercase();
                    
                    if path_lower.contains("_h.") || path_lower.contains("_h/") {
                        if let Some(shader_tex) = shader_textures.and_then(|t| t.first()) {
                            let material = super::model_shader::get_fire_shader_material();
                            gl_use_material(material);
                            material.set_uniform("time", get_time() as f32);
                            material.set_texture("_fire_tex", shader_tex.clone());
                            draw_mesh(&mesh_data);
                            count_shader!("md3_fire_shader");
                            gl_use_default_material();
                            shader_applied = true;
                        }
                    } else if path_lower.contains("_a.") || path_lower.contains("_a/") {
                        if let Some(env_tex) = shader_textures.and_then(|t| t.first()) {
                            let material = super::model_shader::get_envmap_shader_material();
                            gl_use_material(material);
                            material.set_uniform("time", get_time() as f32);
                            material.set_texture("_env_map", env_tex.clone());
                            draw_mesh(&mesh_data);
                            count_shader!("md3_envmap_shader");
                            gl_use_default_material();
                            shader_applied = true;
                        }
                    } else if path_lower.contains("_q.") || path_lower.contains("_q/") {
                        let material = super::model_shader::get_alpha_test_shader_material();
                        gl_use_material(material);
                        draw_mesh(&mesh_data);
                        count_shader!("md3_alpha_test");
                        gl_use_default_material();
                        shader_applied = true;
                    } else if (path_lower.contains("/xaero.") || path_lower.contains("xaero.png") || path_lower.contains("xaero.tga")) && 
                              !path_lower.contains("_h") && !path_lower.contains("_a") && !path_lower.contains("_q") {
                        let material = super::model_shader::get_diffuse_specular_material();
                        gl_use_material(material);
                        draw_mesh(&mesh_data);
                        count_shader!("md3_diffuse_specular");
                        gl_use_default_material();
                        shader_applied = true;
                    }
                }
                
                if !shader_applied {
                    let use_additive = if let Some(path) = texture_path {
                        let path_lower = path.to_lowercase();
                        path_lower.contains("skate") || 
                        path_lower.contains("null") ||
                        path_lower.contains("_f.") ||
                        path_lower.contains("/f_")
                    } else {
                        false
                    };
                    
                    if use_additive {
                        let material = shader::create_model_additive_material();
                        gl_use_material(material);
                        if let Some(ctx) = lighting_context {
                            apply_lighting_uniforms(material, ctx, screen_x + ctx.camera_x, screen_y + ctx.camera_y);
                        }
                        draw_mesh(&mesh_data);
                        count_shader!("md3_additive");
                        gl_use_default_material();
                    } else {
                        if let Some(ctx) = lighting_context {
                            let material = get_model_lit_material();
                            gl_use_material(material);
                            apply_lighting_uniforms(material, ctx, screen_x + ctx.camera_x, screen_y + ctx.camera_y);
                            draw_mesh(&mesh_data);
                            count_shader!("md3_lit");
                            gl_use_default_material();
                        } else {
                            draw_mesh(&mesh_data);
                            count_shader!("md3_lit");
                        }
                    }
                }
            }
        }
    }
}

pub fn _render_md3_mesh_with_transform(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    pitch_angle: f32,
    rot: [[f32; 3]; 3],
    trans: [f32; 3],
) {
    if mesh.vertices.len() <= frame_idx {
        return;
    }
    let frame_verts = &mesh.vertices[frame_idx];
    if frame_verts.is_empty() || mesh.triangles.is_empty() {
        return;
    }

    let x_mult = if flip_x { -1.0 } else { 1.0 };

    let pitch = pitch_angle.clamp(-0.15, 0.15);
    let cos_p = pitch.cos();
    let sin_p = pitch.sin();

    let t = [trans[0] / 64.0, trans[1] / 64.0, trans[2] / 64.0];
    let r = rot;

    for triangle in &mesh.triangles {
        let v0_idx = triangle.vertex[0] as usize;
        let v1_idx = triangle.vertex[1] as usize;
        let v2_idx = triangle.vertex[2] as usize;

        if v0_idx >= frame_verts.len() ||
           v1_idx >= frame_verts.len() ||
           v2_idx >= frame_verts.len() ||
           v0_idx >= mesh.tex_coords.len() ||
           v1_idx >= mesh.tex_coords.len() ||
           v2_idx >= mesh.tex_coords.len() {
            continue;
        }

        let v0 = &frame_verts[v0_idx];
        let v1 = &frame_verts[v1_idx];
        let v2 = &frame_verts[v2_idx];

        let mut p0 = [v0.vertex[0] as f32 / 64.0, v0.vertex[1] as f32 / 64.0, v0.vertex[2] as f32 / 64.0];
        let mut p1 = [v1.vertex[0] as f32 / 64.0, v1.vertex[1] as f32 / 64.0, v1.vertex[2] as f32 / 64.0];
        let mut p2 = [v2.vertex[0] as f32 / 64.0, v2.vertex[1] as f32 / 64.0, v2.vertex[2] as f32 / 64.0];

        let tx = |p: [f32; 3]| -> [f32; 3] {[
            r[0][0] * p[0] + r[0][1] * p[1] + r[0][2] * p[2] + t[0],
            r[1][0] * p[0] + r[1][1] * p[1] + r[1][2] * p[2] + t[1],
            r[2][0] * p[0] + r[2][1] * p[1] + r[2][2] * p[2] + t[2],
        ]};

        p0 = tx(p0);
        p1 = tx(p1);
        p2 = tx(p2);

        let rz0 = -(p0[1] * sin_p + p0[2] * cos_p) * scale;
        let rz1 = -(p1[1] * sin_p + p1[2] * cos_p) * scale;
        let rz2 = -(p2[1] * sin_p + p2[2] * cos_p) * scale;

        let x0 = screen_x + p0[0] * scale * x_mult;
        let y0 = screen_y - rz0;
        let x1 = screen_x + p1[0] * scale * x_mult;
        let y1 = screen_y - rz1;
        let x2 = screen_x + p2[0] * scale * x_mult;
        let y2 = screen_y - rz2;

        if let Some(tex) = texture {
            tex.set_filter(FilterMode::Linear);

            let tc0 = &mesh.tex_coords[v0_idx];
            let tc1 = &mesh.tex_coords[v1_idx];
            let tc2 = &mesh.tex_coords[v2_idx];

            let vertices = vec![
                Vertex { position: Vec3::new(x0, y0, 0.0), uv: Vec2::new(tc0.coord[0], tc0.coord[1]), color: [(color.r * 255.0) as u8,(color.g * 255.0) as u8,(color.b * 255.0) as u8,(color.a * 255.0) as u8], normal: Vec4::new(0.0, 0.0, 1.0, 0.0) },
                Vertex { position: Vec3::new(x1, y1, 0.0), uv: Vec2::new(tc1.coord[0], tc1.coord[1]), color: [(color.r * 255.0) as u8,(color.g * 255.0) as u8,(color.b * 255.0) as u8,(color.a * 255.0) as u8], normal: Vec4::new(0.0, 0.0, 1.0, 0.0) },
                Vertex { position: Vec3::new(x2, y2, 0.0), uv: Vec2::new(tc2.coord[0], tc2.coord[1]), color: [(color.r * 255.0) as u8,(color.g * 255.0) as u8,(color.b * 255.0) as u8,(color.a * 255.0) as u8], normal: Vec4::new(0.0, 0.0, 1.0, 0.0) },
            ];

            let indices = vec![0_u16, 1, 2];
            let mesh_data = macroquad::models::Mesh { vertices, indices, texture: Some(tex.clone()) };
            draw_mesh(&mesh_data);
            count_shader!("md3_single");
        } else {
            draw_triangle(Vec2::new(x0, y0), Vec2::new(x1, y1), Vec2::new(x2, y2), color);
        }
    }
}

pub fn render_md3_mesh_rotated(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    rotation: f32,
) {
    render_md3_mesh_rotated_with_additive(mesh, frame_idx, screen_x, screen_y, scale, color, texture, None, rotation, false)
}

pub fn render_md3_mesh_rotated_with_additive(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    rotation: f32,
    force_additive: bool,
) {
    if mesh.vertices.len() <= frame_idx {
        return;
    }
    
    let frame_verts = &mesh.vertices[frame_idx];
    if frame_verts.is_empty() || mesh.triangles.is_empty() {
        return;
    }
    
    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
        .trim_end_matches('\0')
        .to_string();
    
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    
    let mut all_vertices: Vec<Vertex> = Vec::with_capacity(mesh.triangles.len() * 3);
    let mut all_indices: Vec<u16> = Vec::with_capacity(mesh.triangles.len() * 3);
    
    for triangle in &mesh.triangles {
        let v0_idx = triangle.vertex[0] as usize;
        let v1_idx = triangle.vertex[1] as usize;
        let v2_idx = triangle.vertex[2] as usize;
        
        if v0_idx >= frame_verts.len() || 
           v1_idx >= frame_verts.len() || 
           v2_idx >= frame_verts.len() ||
           v0_idx >= mesh.tex_coords.len() ||
           v1_idx >= mesh.tex_coords.len() ||
           v2_idx >= mesh.tex_coords.len() {
            continue;
        }
        
        let v0 = &frame_verts[v0_idx];
        let v1 = &frame_verts[v1_idx];
        let v2 = &frame_verts[v2_idx];
        
        let v0_x = v0.vertex[0] as f32 * scale / 64.0;
        let v0_y = v0.vertex[1] as f32 * scale / 64.0;
        let v0_z = v0.vertex[2] as f32 * scale / 64.0;
        
        let v1_x = v1.vertex[0] as f32 * scale / 64.0;
        let v1_y = v1.vertex[1] as f32 * scale / 64.0;
        let v1_z = v1.vertex[2] as f32 * scale / 64.0;
        
        let v2_x = v2.vertex[0] as f32 * scale / 64.0;
        let v2_y = v2.vertex[1] as f32 * scale / 64.0;
        let v2_z = v2.vertex[2] as f32 * scale / 64.0;
        
        let rx0 = v0_x * cos_r - v0_y * sin_r;
        let _ry0 = v0_x * sin_r + v0_y * cos_r;
        
        let rx1 = v1_x * cos_r - v1_y * sin_r;
        let _ry1 = v1_x * sin_r + v1_y * cos_r;
        
        let rx2 = v2_x * cos_r - v2_y * sin_r;
        let _ry2 = v2_x * sin_r + v2_y * cos_r;
        
        let x0 = screen_x + rx0;
        let y0 = screen_y - v0_z;
        let x1 = screen_x + rx1;
        let y1 = screen_y - v1_z;
        let x2 = screen_x + rx2;
        let y2 = screen_y - v2_z;
        
        if let Some(_tex) = texture {
            let tc0 = &mesh.tex_coords[v0_idx];
            let tc1 = &mesh.tex_coords[v1_idx];
            let tc2 = &mesh.tex_coords[v2_idx];
            
            let transform_normal = |n_raw: u16| -> Vec3 {
                let n = decode_md3_normal(n_raw);
                let n_rx = n.x * cos_r - n.y * sin_r;
                let n_ry = n.x * sin_r + n.y * cos_r;
                Vec3::new(n_rx, n_ry, n.z).normalize()
            };

            let n0 = transform_normal(v0.normal);
            let n1 = transform_normal(v1.normal);
            let n2 = transform_normal(v2.normal);
            
            let base = all_vertices.len() as u16;
            
            all_vertices.push(Vertex {
                position: Vec3::new(x0, y0, 0.0),
                uv: Vec2::new(tc0.coord[0], tc0.coord[1]),
                color: [
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    (color.a * 255.0) as u8,
                ],
                normal: Vec4::new(n0.x, n0.y, n0.z, 0.0),
            });
            all_vertices.push(Vertex {
                position: Vec3::new(x1, y1, 0.0),
                uv: Vec2::new(tc1.coord[0], tc1.coord[1]),
                color: [
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    (color.a * 255.0) as u8,
                ],
                normal: Vec4::new(n1.x, n1.y, n1.z, 0.0),
            });
            all_vertices.push(Vertex {
                position: Vec3::new(x2, y2, 0.0),
                uv: Vec2::new(tc2.coord[0], tc2.coord[1]),
                color: [
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    (color.a * 255.0) as u8,
                ],
                normal: Vec4::new(n2.x, n2.y, n2.z, 0.0),
            });
            
            all_indices.push(base);
            all_indices.push(base + 1);
            all_indices.push(base + 2);
        } else {
            draw_triangle(
                Vec2::new(x0, y0),
                Vec2::new(x1, y1),
                Vec2::new(x2, y2),
                color,
            );
        }
    }
    
    if let Some(tex) = texture {
        if !all_vertices.is_empty() {
            let mesh_data = macroquad::models::Mesh {
                vertices: all_vertices,
                indices: all_indices,
                texture: Some(tex.clone()),
            };
            
            let use_additive = force_additive || if let Some(path) = texture_path {
                let path_lower = path.to_lowercase();
                path_lower.contains("_f.") || path_lower.contains("/f_") || path_lower.contains("flash")
            } else {
                false
            };
            
            if use_additive {
                let material = shader::create_model_additive_material();
                gl_use_material(material);
                draw_mesh(&mesh_data);
                count_shader!(&format!("md3_rotated_additive:{}", mesh_name));
                gl_use_default_material();
            } else {
                draw_mesh(&mesh_data);
                count_shader!(&format!("md3_rotated:{}", mesh_name));
            }
        }
    }
}

pub fn _render_md3_mesh_screen_rotated(
    mesh: &Mesh,
    frame_idx: usize,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    angle: f32,
    pivot_screen_x: f32,
    pivot_screen_y: f32,
) {
    if mesh.vertices.len() <= frame_idx {
        return;
    }
    let frame_verts = &mesh.vertices[frame_idx];
    if frame_verts.is_empty() || mesh.triangles.is_empty() {
        return;
    }

    let x_mult = if flip_x { -1.0 } else { 1.0 };
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    for triangle in &mesh.triangles {
        let v0_idx = triangle.vertex[0] as usize;
        let v1_idx = triangle.vertex[1] as usize;
        let v2_idx = triangle.vertex[2] as usize;

        if v0_idx >= frame_verts.len() ||
           v1_idx >= frame_verts.len() ||
           v2_idx >= frame_verts.len() ||
           v0_idx >= mesh.tex_coords.len() ||
           v1_idx >= mesh.tex_coords.len() ||
           v2_idx >= mesh.tex_coords.len() {
            continue;
        }

        let v0 = &frame_verts[v0_idx];
        let v1 = &frame_verts[v1_idx];
        let v2 = &frame_verts[v2_idx];

        let v0_x = v0.vertex[0] as f32 * scale / 64.0 * x_mult;
        let v0_z = v0.vertex[2] as f32 * scale / 64.0;

        let v1_x = v1.vertex[0] as f32 * scale / 64.0 * x_mult;
        let v1_z = v1.vertex[2] as f32 * scale / 64.0;

        let v2_x = v2.vertex[0] as f32 * scale / 64.0 * x_mult;
        let v2_z = v2.vertex[2] as f32 * scale / 64.0;

        let sx0 = screen_x + v0_x;
        let sy0 = screen_y - v0_z;
        let sx1 = screen_x + v1_x;
        let sy1 = screen_y - v1_z;
        let sx2 = screen_x + v2_x;
        let sy2 = screen_y - v2_z;

        let dx0 = sx0 - pivot_screen_x;
        let dy0 = sy0 - pivot_screen_y;
        let dx1 = sx1 - pivot_screen_x;
        let dy1 = sy1 - pivot_screen_y;
        let dx2 = sx2 - pivot_screen_x;
        let dy2 = sy2 - pivot_screen_y;

        let x0 = pivot_screen_x + dx0 * cos_a - dy0 * sin_a;
        let y0 = pivot_screen_y + dx0 * sin_a + dy0 * cos_a;
        let x1 = pivot_screen_x + dx1 * cos_a - dy1 * sin_a;
        let y1 = pivot_screen_y + dx1 * sin_a + dy1 * cos_a;
        let x2 = pivot_screen_x + dx2 * cos_a - dy2 * sin_a;
        let y2 = pivot_screen_y + dx2 * sin_a + dy2 * cos_a;

        if let Some(tex) = texture {
            tex.set_filter(FilterMode::Linear);

            let tc0 = &mesh.tex_coords[v0_idx];
            let tc1 = &mesh.tex_coords[v1_idx];
            let tc2 = &mesh.tex_coords[v2_idx];

            let vertices = vec![
                Vertex {
                    position: Vec3::new(x0, y0, 0.0),
                    uv: Vec2::new(tc0.coord[0], tc0.coord[1]),
                    color: [
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ],
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(x1, y1, 0.0),
                    uv: Vec2::new(tc1.coord[0], tc1.coord[1]),
                    color: [
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ],
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
                Vertex {
                    position: Vec3::new(x2, y2, 0.0),
                    uv: Vec2::new(tc2.coord[0], tc2.coord[1]),
                    color: [
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        (color.a * 255.0) as u8,
                    ],
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                },
            ];

            let indices = vec![0_u16, 1, 2];

            let mesh = macroquad::models::Mesh {
                vertices,
                indices,
                texture: Some(tex.clone()),
            };

            draw_mesh(&mesh);
            count_shader!("md3_default");
        } else {
            draw_triangle(
                Vec2::new(x0, y0),
                Vec2::new(x1, y1),
                Vec2::new(x2, y2),
                color,
            );
        }
    }
}

pub fn render_md3_mesh_with_pivot(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    angle: f32,
    pivot_x: f32,
    pivot_y: f32,
) {
    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
        .trim_end_matches('\0')
        .to_string();
    count_shader!(&format!("md3_pivot:{}", mesh_name));
    render_md3_mesh_with_pivot_and_yaw(mesh, frame_idx, origin_x, origin_y, scale, color, texture, flip_x, angle, 0.0, pivot_x, pivot_y);
}

pub fn render_md3_mesh_with_pivot_and_yaw_ex(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
    roll_angle: f32,
) {
    render_md3_mesh_with_pivot_and_yaw_internal(mesh, frame_idx, origin_x, origin_y, scale, color, texture, texture_path, None, flip_x, angle, yaw_angle, pivot_x, pivot_y, roll_angle, 0.0, false);
}

pub fn render_md3_mesh_with_pivot_and_yaw_ex_shader(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    shader_textures: Option<&[Texture2D]>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
    roll_angle: f32,
) {
    render_md3_mesh_with_pivot_and_yaw_internal(mesh, frame_idx, origin_x, origin_y, scale, color, texture, texture_path, shader_textures, flip_x, angle, yaw_angle, pivot_x, pivot_y, roll_angle, 0.0, false);
}

pub fn render_md3_mesh_with_pivot_and_yaw_ex_quad(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
    roll_angle: f32,
) {
    render_md3_mesh_with_pivot_and_yaw_internal(mesh, frame_idx, origin_x, origin_y, scale, color, texture, texture_path, None, flip_x, angle, yaw_angle, pivot_x, pivot_y, roll_angle, 0.0, true);
}

pub fn render_md3_mesh_with_pivot_and_yaw(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
) {
    render_md3_mesh_with_pivot_and_yaw_internal(mesh, frame_idx, origin_x, origin_y, scale, color, texture, None, None, flip_x, angle, yaw_angle, pivot_x, pivot_y, 0.0, 0.0, false);
}

pub fn render_md3_mesh_with_pivot_and_yaw_ex_with_barrel(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
    roll_angle: f32,
    barrel_spin_angle: f32,
) {
    render_md3_mesh_with_pivot_and_yaw_internal(mesh, frame_idx, origin_x, origin_y, scale, color, texture, texture_path, None, flip_x, angle, yaw_angle, pivot_x, pivot_y, roll_angle, barrel_spin_angle, false);
}

pub fn render_md3_mesh_with_pivot_and_yaw_ex_quad_with_barrel(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    texture_path: Option<&str>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
    roll_angle: f32,
    barrel_spin_angle: f32,
) {
    render_md3_mesh_with_pivot_and_yaw_internal(mesh, frame_idx, origin_x, origin_y, scale, color, texture, texture_path, None, flip_x, angle, yaw_angle, pivot_x, pivot_y, roll_angle, barrel_spin_angle, true);
}

fn render_md3_mesh_with_pivot_and_yaw_internal(
    mesh: &Mesh,
    frame_idx: usize,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    color: Color,
    texture: Option<&Texture2D>,
    _texture_path: Option<&str>,
    _shader_textures: Option<&[Texture2D]>,
    flip_x: bool,
    angle: f32,
    yaw_angle: f32,
    pivot_x: f32,
    pivot_y: f32,
    roll_angle: f32,
    barrel_spin_angle: f32,
    use_quad_shader: bool,
) {
    
    if mesh.vertices.len() <= frame_idx {
        return;
    }
    let frame_verts = &mesh.vertices[frame_idx];
    if frame_verts.is_empty() || mesh.triangles.is_empty() {
        return;
    }

    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
        .trim_end_matches('\0')
        .to_string();

    let x_mult = if flip_x { -1.0 } else { 1.0 };
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let cos_y = yaw_angle.cos();
    let sin_y = yaw_angle.sin();
    let cos_r = roll_angle.cos();
    let sin_r = roll_angle.sin();
    let cos_barrel = barrel_spin_angle.cos();
    let sin_barrel = barrel_spin_angle.sin();

    let mut all_vertices: Vec<Vertex> = Vec::with_capacity(mesh.triangles.len() * 3);
    let mut all_indices: Vec<u16> = Vec::with_capacity(mesh.triangles.len() * 3);

    for triangle in &mesh.triangles {
        let v0_idx = triangle.vertex[0] as usize;
        let v1_idx = triangle.vertex[1] as usize;
        let v2_idx = triangle.vertex[2] as usize;

        if v0_idx >= frame_verts.len() ||
           v1_idx >= frame_verts.len() ||
           v2_idx >= frame_verts.len() ||
           v0_idx >= mesh.tex_coords.len() ||
           v1_idx >= mesh.tex_coords.len() ||
           v2_idx >= mesh.tex_coords.len() {
            continue;
        }

        let v0 = &frame_verts[v0_idx];
        let v1 = &frame_verts[v1_idx];
        let v2 = &frame_verts[v2_idx];

        let transform_vertex_and_normal = |v: &super::md3::Vertex| -> (Vec3, Vec3) {
            let mut vx = v.vertex[0] as f32 / 64.0 * scale * x_mult;
            let mut vy = v.vertex[1] as f32 / 64.0 * scale;
            let mut vz = v.vertex[2] as f32 / 64.0 * scale;

            if barrel_spin_angle.abs() > 0.001 {
                let vy_rot = vy * cos_barrel - vz * sin_barrel;
                let vz_rot = vy * sin_barrel + vz * cos_barrel;
                vy = vy_rot;
                vz = vz_rot;
            }

            let dx = vx;
            let dy = -vz;

            let rdx = dx * cos_a - dy * sin_a;
            let rdy = dx * sin_a + dy * cos_a;
            
            let ydx = rdx * cos_y - rdy * sin_y;
            let ydy = rdx * sin_y + rdy * cos_y;
            
            let roll_dx = ydx * cos_r - ydy * sin_r;
            let roll_dy = ydx * sin_r + ydy * cos_r;
            
            let final_x = origin_x + (roll_dx - pivot_x * x_mult);
            let final_y = origin_y + (roll_dy + pivot_y);

            let n = decode_md3_normal(v.normal);
            let mut nx = n.x * x_mult;
            let mut ny = n.y;
            let mut nz = n.z;

            if barrel_spin_angle.abs() > 0.001 {
                let ny_rot = ny * cos_barrel - nz * sin_barrel;
                let nz_rot = ny * sin_barrel + nz * cos_barrel;
                ny = ny_rot;
                nz = nz_rot;
            }

            let n_dx = nx;
            let n_dy = -nz;

            let n_rdx = n_dx * cos_a - n_dy * sin_a;
            let n_rdy = n_dx * sin_a + n_dy * cos_a;

            let n_ydx = n_rdx * cos_y - n_rdy * sin_y;
            let n_ydy = n_rdx * sin_y + n_rdy * cos_y;
            let n_yz = ny;

            let normal = Vec3::new(n_ydx, n_ydy, n_yz).normalize();

            (Vec3::new(final_x, final_y, 0.0), normal)
        };

        let (pos0, norm0) = transform_vertex_and_normal(v0);
        let (pos1, norm1) = transform_vertex_and_normal(v1);
        let (pos2, norm2) = transform_vertex_and_normal(v2);

        if let Some(_tex) = texture {
            let tc0 = &mesh.tex_coords[v0_idx];
            let tc1 = &mesh.tex_coords[v1_idx];
            let tc2 = &mesh.tex_coords[v2_idx];

            let base = all_vertices.len() as u16;
            
            all_vertices.push(Vertex {
                position: pos0,
                uv: Vec2::new(tc0.coord[0], tc0.coord[1]),
                color: color.into(),
                normal: Vec4::new(norm0.x, norm0.y, norm0.z, 0.0),
            });
            all_vertices.push(Vertex {
                position: pos1,
                uv: Vec2::new(tc1.coord[0], tc1.coord[1]),
                color: color.into(),
                normal: Vec4::new(norm1.x, norm1.y, norm1.z, 0.0),
            });
            all_vertices.push(Vertex {
                position: pos2,
                uv: Vec2::new(tc2.coord[0], tc2.coord[1]),
                color: color.into(),
                normal: Vec4::new(norm2.x, norm2.y, norm2.z, 0.0),
            });

            all_indices.push(base);
            all_indices.push(base + 1);
            all_indices.push(base + 2);
        } else {
            draw_triangle(
                Vec2::new(pos0.x, pos0.y),
                Vec2::new(pos1.x, pos1.y),
                Vec2::new(pos2.x, pos2.y),
                color,
            );
        }
    }

    if let Some(tex) = texture {
        if !all_vertices.is_empty() {
            let mesh_data = macroquad::models::Mesh { 
                vertices: all_vertices, 
                indices: all_indices, 
                texture: Some(tex.clone()) 
            };
            
            if use_quad_shader {
                let material = shader::create_quad_damage_outline_material();
                gl_use_material(material);
                material.set_uniform("time", get_time() as f32);
                material.set_uniform("outlineWidth", 2.5f32);
                draw_mesh(&mesh_data);
                count_shader!("md3_quad_pivot");
                gl_use_default_material();
            } else {
                draw_mesh(&mesh_data);
                count_shader!(&format!("md3_pivot_internal:{}", mesh_name));
            }
        }
    }
}

