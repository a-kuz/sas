use macroquad::prelude::*;
use super::types::*;
use std::collections::HashMap;
use crate::count_shader;

pub struct Renderer {
    texture_cache: HashMap<u64, Texture2D>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            texture_cache: HashMap::new(),
        }
    }
    
    fn build_shadow_map(batch: &RenderBatch) -> Texture2D {
        let width = batch.map_size.x as usize;
        let height = batch.map_size.y as usize;
        let mut pixels = vec![0u8; width * height * 4];
        
        for caster in &batch.shadow_casters {
            let min_x = caster.min.x.max(0.0).min(batch.map_size.x) as usize;
            let max_x = caster.max.x.max(0.0).min(batch.map_size.x) as usize;
            let min_y = caster.min.y.max(0.0).min(batch.map_size.y) as usize;
            let max_y = caster.max.y.max(0.0).min(batch.map_size.y) as usize;
            
            for py in min_y..max_y {
                for px in min_x..max_x {
                    let idx = (py * width + px) * 4;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = 255;
                        pixels[idx + 1] = 255;
                        pixels[idx + 2] = 255;
                        pixels[idx + 3] = 255;
                    }
                }
            }
        }
        
        let image = Image {
            bytes: pixels,
            width: width as u16,
            height: height as u16,
        };
        let tex = Texture2D::from_image(&image);
        tex.set_filter(FilterMode::Linear);
        tex
    }
    
    pub fn render(&mut self, batch: &RenderBatch, textures: &HashMap<u64, Texture2D>) {
        if batch.meshes.is_empty() {
            return;
        }
        
        let shadow_map = Self::build_shadow_map(batch);
        
        let estimated_verts = batch.meshes.iter().map(|m| m.triangles.len() * 3).sum();
        let mut batched_meshes: HashMap<u64, Vec<Vertex>> = HashMap::with_capacity(textures.len());
        let mut batched_indices: HashMap<u64, Vec<u16>> = HashMap::with_capacity(textures.len());
        
        for mesh in &batch.meshes {
            let verts = batched_meshes.entry(mesh.texture_id).or_insert_with(|| Vec::with_capacity(estimated_verts / textures.len().max(1)));
            let indices = batched_indices.entry(mesh.texture_id).or_insert_with(|| Vec::with_capacity(estimated_verts / textures.len().max(1)));
            
            for tri in &mesh.triangles {
                let base = verts.len() as u16;
                
                verts.push(Vertex {
                    position: tri.v0,
                    uv: tri.uv0,
                    color: [255, 255, 255, 255],
                    normal: Vec4::new(tri.n0.x, tri.n0.y, tri.n0.z, 0.0),
                });
                verts.push(Vertex {
                    position: tri.v1,
                    uv: tri.uv1,
                    color: [255, 255, 255, 255],
                    normal: Vec4::new(tri.n1.x, tri.n1.y, tri.n1.z, 0.0),
                });
                verts.push(Vertex {
                    position: tri.v2,
                    uv: tri.uv2,
                    color: [255, 255, 255, 255],
                    normal: Vec4::new(tri.n2.x, tri.n2.y, tri.n2.z, 0.0),
                });
                
                indices.push(base);
                indices.push(base + 1);
                indices.push(base + 2);
            }
        }
        
        
        for (texture_id, verts) in batched_meshes.iter() {
            if let Some(texture) = textures.get(texture_id) {
                if let Some(inds) = batched_indices.get(texture_id) {
                    texture.set_filter(FilterMode::Linear);
                    
                    let mesh_data = macroquad::models::Mesh {
                        vertices: verts.clone(),
                        indices: inds.clone(),
                        texture: Some(texture.clone()),
                    };
                    
                    draw_mesh(&mesh_data);
                    count_shader!("renderer_batched");
                }
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}


