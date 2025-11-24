use macroquad::prelude::*;
use super::map::{Map, LightSource};
use super::shader;
use super::light_grid::LightGrid;
use super::lightmap::Lightmap;
use std::sync::OnceLock;
#[cfg(feature = "profiler")]
use crate::profiler;

// Макрос для условного профилирования
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "profiler")]
        let _scope = profiler::scope($name);
        #[cfg(not(feature = "profiler"))]
        let _scope = ();
    };
}

static HYBRID_LIGHTING_MATERIAL: OnceLock<Material> = OnceLock::new();
static OVERLAY_LIGHTING_MATERIAL: OnceLock<Material> = OnceLock::new();

fn get_hybrid_lighting_material() -> &'static Material {
    HYBRID_LIGHTING_MATERIAL.get_or_init(|| shader::create_hybrid_lighting_material())
}

fn get_overlay_lighting_material() -> &'static Material {
    OVERLAY_LIGHTING_MATERIAL.get_or_init(|| shader::create_overlay_lighting_material().clone())
}

pub struct DeferredRenderer {
    pub scene_target: Option<RenderTarget>,
    map_width: usize,
    map_height: usize,
    last_screen_w: u32,
    last_screen_h: u32,
    last_render_scale: f32,
    light_grid: LightGrid,
    lightmap: Option<Lightmap>,
    static_lights_dirty: bool,
    obstacle_texture: Option<Texture2D>,
    dynamic_light_texture: Option<Texture2D>,
    linear_light_texture: Option<Texture2D>,
    light_data_buffer: [u8; 64],
    linear_light_data_buffer: [u8; 32],
    static_uniforms_set: bool,
    lit_scene_target: Option<RenderTarget>,
}

impl DeferredRenderer {
    pub fn new(map: &Map) -> Self {
        Self::new_with_scale(map, 2.0)
    }
    
    pub fn new_with_scale(map: &Map, render_scale: f32) -> Self {
        let screen_w = screen_width() as u32;
        let screen_h = screen_height() as u32;
        
        let target_w = (screen_w as f32 * render_scale) as u32;
        let target_h = (screen_h as f32 * render_scale) as u32;
        
        println!("[DeferredRenderer] Creating render target {}x{} (scale: {}x)", target_w, target_h, render_scale);
        let target = render_target_ex(target_w, target_h, RenderTargetParams { depth: false, sample_count: 1 });
        target.texture.set_filter(FilterMode::Linear);
        
        let light_grid = LightGrid::new(map);
        
        let dynamic_light_tex = Texture2D::from_rgba8(8, 2, &[0u8; 64]);
        let linear_light_tex = Texture2D::from_rgba8(4, 2, &[0u8; 32]);
        
        let lit_target = render_target_ex(target_w, target_h, RenderTargetParams { depth: false, sample_count: 1 });
        lit_target.texture.set_filter(FilterMode::Nearest);
        
        Self {
            scene_target: Some(target),
            map_width: map.width,
            map_height: map.height,
            last_screen_w: screen_w,
            last_screen_h: screen_h,
            last_render_scale: render_scale,
            light_grid,
            lightmap: None,
            static_lights_dirty: true,
            obstacle_texture: None,
            dynamic_light_texture: Some(dynamic_light_tex),
            linear_light_texture: Some(linear_light_tex),
            light_data_buffer: [0u8; 64],
            linear_light_data_buffer: [0u8; 32],
            static_uniforms_set: false,
            lit_scene_target: Some(lit_target),
        }
    }
    
    pub fn mark_static_lights_dirty(&mut self) {
        self.static_lights_dirty = true;
        self.lightmap = None;
        self.static_uniforms_set = false;
    }
    
    pub fn create_obstacle_texture(map: &Map) -> Texture2D {
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
        
        let image = Image {
            bytes: pixels,
            width: width as u16,
            height: height as u16,
        };
        let tex = Texture2D::from_image(&image);
        tex.set_filter(FilterMode::Linear);
        tex
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
            self.light_data_buffer[idx1 + 2] = (light.radius * map_size_inv * 100.0).min(255.0) as u8;
            self.light_data_buffer[idx1 + 3] = (light.intensity * 255.0).min(255.0) as u8;
            
            self.light_data_buffer[idx2] = light.r;
            self.light_data_buffer[idx2 + 1] = light.g;
            self.light_data_buffer[idx2 + 2] = light.b;
            self.light_data_buffer[idx2 + 3] = if light.flicker { 255 } else { 0 };
        }
        
        if let Some(ref tex) = self.dynamic_light_texture {
            tex.update(&Image {
                bytes: self.light_data_buffer.to_vec(),
                width: 8,
                height: 2,
            });
        }
    }
    
    pub fn begin_scene(&mut self) {
        self.begin_scene_with_scale(self.last_render_scale);
    }
    
    pub fn begin_scene_with_scale(&mut self, render_scale: f32) {
        let current_w = screen_width() as u32;
        let current_h = screen_height() as u32;
        
        if current_w != self.last_screen_w || current_h != self.last_screen_h || render_scale != self.last_render_scale {
            let target_w = (current_w as f32 * render_scale) as u32;
            let target_h = (current_h as f32 * render_scale) as u32;
            let target = render_target_ex(target_w, target_h, RenderTargetParams { depth: false, sample_count: 1 });
            target.texture.set_filter(FilterMode::Linear);
            self.scene_target = Some(target);
            
            let lit_target = render_target_ex(target_w, target_h, RenderTargetParams { depth: false, sample_count: 1 });
            lit_target.texture.set_filter(FilterMode::Nearest);
            self.lit_scene_target = Some(lit_target);
            
            self.last_screen_w = current_w;
            self.last_screen_h = current_h;
            self.last_render_scale = render_scale;
        }
        
        if let Some(ref target) = self.scene_target {
            set_camera(&Camera2D {
                render_target: Some(target.clone()),
                zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
                target: vec2(screen_width() / 2.0, screen_height() / 2.0),
                offset: vec2(0.0, 0.0),
                ..Default::default()
            });
        }
        
        clear_background(Color::from_rgba(14, 16, 20, 255));
    }
    
    pub fn end_scene(&self) {
        set_default_camera();
    }
    
    pub fn apply_lighting(&mut self, map: &Map, static_lights: &[LightSource], all_lights: &[LightSource], linear_lights: &[super::map::LinearLight], camera_x: f32, camera_y: f32, zoom: f32, ambient: f32, disable_shadows: bool, disable_dynamic_lights: bool, cartoon_shader: bool) {
        let scene_texture = match &self.scene_target {
            Some(target) => target.texture.clone(),
            None => {
                println!("[DeferredRenderer] ERROR: No scene target!");
                return;
            },
        };
        
        {
            profile_scope!("lighting_lightmap_check");
            if self.lightmap.is_none() || self.static_lights_dirty {
                println!("[Lightmap] Building lightmap for {} static lights...", static_lights.len());
                self.lightmap = Some(Lightmap::new(map, static_lights, ambient));
                self.light_grid.rebuild(static_lights, map);
                self.static_lights_dirty = false;
                println!("[Lightmap] Done!");
            }
        }
        
        let dynamic_lights: Vec<LightSource> = {
            profile_scope!("lighting_filter_dynamic");
            all_lights.iter()
                .filter(|light| {
                    !static_lights.iter().any(|sl| 
                        (sl.x - light.x).abs() < 1.0 && (sl.y - light.y).abs() < 1.0
                    )
                })
                .cloned()
                .collect()
        };
        
        let material = get_hybrid_lighting_material();
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        let map_w = self.map_width as f32 * 32.0;
        let map_h = self.map_height as f32 * 16.0;
        
        let active_dynamic_lights: Vec<LightSource> = {
            profile_scope!("lighting_cull_lights");
            let mut lights = Vec::with_capacity(8);
            for light in &dynamic_lights {
                let dx = light.x - (camera_x + (screen_w / zoom) * 0.5);
                let dy = light.y - (camera_y + (screen_h / zoom) * 0.5);
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < light.radius + (screen_w.max(screen_h) / zoom) {
                    lights.push(light.clone());
                    if lights.len() >= 8 {
                        break;
                    }
                }
            }
            lights
        };
        
        {
            profile_scope!("lighting_update_light_tex");
            self.update_light_data_buffer(&active_dynamic_lights, map_w, map_h);
            self.update_linear_light_data_buffer(linear_lights, map_w, map_h);
        }
        
        let obstacle_tex = {
            profile_scope!("lighting_create_obstacle_tex");
            if self.obstacle_texture.is_none() {
                self.obstacle_texture = Some(Self::create_obstacle_texture(map));
            }
            self.obstacle_texture.as_ref().unwrap().clone()
        };
        
        {
            profile_scope!("lighting_bind_material");
            gl_use_material(material);
        }
        
        {
            profile_scope!("lighting_set_textures");
            material.set_texture("sceneTexture", scene_texture);
            material.set_texture("lightmapTexture", self.lightmap.as_ref().unwrap().texture.clone());
            material.set_texture("dynamicLightData", self.dynamic_light_texture.as_ref().unwrap().clone());
            material.set_texture("linearLightData", self.linear_light_texture.as_ref().unwrap().clone());
            material.set_texture("obstacleTex", obstacle_tex);
        }
        
        {
            profile_scope!("lighting_set_uniforms");
            
            if !self.static_uniforms_set {
                material.set_uniform("mapSize", [map_w, map_h]);
                material.set_uniform("tileSize", [32.0 as f32, 16.0 as f32]);
                self.static_uniforms_set = true;
            }
            
            material.set_uniform("screenToWorld", [screen_w / zoom, screen_h / zoom]);
            material.set_uniform("cameraPos", [camera_x, camera_y]);
            material.set_uniform("time", get_time() as f32);
            
            if disable_dynamic_lights {
                material.set_uniform("numDynamicLights", 0);
                material.set_uniform("numLinearLights", 0);
            } else {
                material.set_uniform("numDynamicLights", active_dynamic_lights.len() as i32);
                material.set_uniform("numLinearLights", linear_lights.len().min(4) as i32);
            }
            material.set_uniform("disableShadows", if disable_shadows { 1 } else { 0 });
        }
        
        {
            profile_scope!("lighting_draw_fullscreen");
            if cartoon_shader {
                if let Some(ref lit_target) = self.lit_scene_target {
                    let scaled_w = screen_w * self.last_render_scale;
                    let scaled_h = screen_h * self.last_render_scale;
                    set_camera(&Camera2D {
                        render_target: Some(lit_target.clone()),
                        zoom: vec2(2.0 / scaled_w, 2.0 / scaled_h),
                        target: vec2(scaled_w / 2.0, scaled_h / 2.0),
                        offset: vec2(0.0, 0.0),
                        ..Default::default()
                    });
                    draw_rectangle(0.0, 0.0, scaled_w, scaled_h, WHITE);
                    set_default_camera();
                }
            } else {
                draw_rectangle(0.0, 0.0, screen_w, screen_h, WHITE);
            }
        }
        
        {
            profile_scope!("lighting_unbind_material");
            gl_use_default_material();
        }
        
        if cartoon_shader {
            profile_scope!("cartoon_shader");
            
            let lit_texture = match &self.lit_scene_target {
                Some(target) => target.texture.clone(),
                None => {
                    println!("[DeferredRenderer] ERROR: No lit scene target!");
                    return;
                },
            };
            
            let scaled_w = screen_w * self.last_render_scale;
            let scaled_h = screen_h * self.last_render_scale;
            
            let cartoon_material = shader::create_cartoon_shader_material();
            gl_use_material(cartoon_material);
            cartoon_material.set_texture("sceneTexture", lit_texture);
            cartoon_material.set_uniform("screenSize", [scaled_w, scaled_h]);
            draw_rectangle(0.0, 0.0, screen_w, screen_h, WHITE);
            gl_use_default_material();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn apply_wasm_overlay_lighting(&mut self, map: &Map, all_lights: &[LightSource], linear_lights: &[super::map::LinearLight], camera_x: f32, camera_y: f32, zoom: f32, _ambient: f32) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let map_w = self.map_width as f32 * 32.0;
        let map_h = self.map_height as f32 * 16.0;
        
        let lightmap_tex = self.lightmap.as_ref().unwrap().texture.clone();
        
        let dynamic_lights: Vec<LightSource> = all_lights.iter()
            .filter(|light| {
                !map.lights.iter().any(|sl| 
                    (sl.x - light.x).abs() < 1.0 && (sl.y - light.y).abs() < 1.0
                )
            })
            .cloned()
            .collect();
        
        let active_lights: Vec<LightSource> = dynamic_lights.iter()
            .filter(|light| {
                let dx = light.x - (camera_x + (screen_w / zoom) * 0.5);
                let dy = light.y - (camera_y + (screen_h / zoom) * 0.5);
                let dist = (dx * dx + dy * dy).sqrt();
                dist < light.radius + (screen_w.max(screen_h) / zoom)
            })
            .take(8)
            .cloned()
            .collect();
        
        self.update_light_data_buffer(&active_lights, map_w, map_h);
        self.update_linear_light_data_buffer(linear_lights, map_w, map_h);
        
        let obstacle_tex = {
            if self.obstacle_texture.is_none() {
                self.obstacle_texture = Some(Self::create_obstacle_texture(map));
            }
            self.obstacle_texture.as_ref().unwrap().clone()
        };
        
        let material = get_overlay_lighting_material();
        gl_use_material(material);
        
        material.set_texture("lightmapTexture", lightmap_tex.clone());
        material.set_texture("lightData", self.dynamic_light_texture.as_ref().unwrap().clone());
        material.set_texture("linearLightData", self.linear_light_texture.as_ref().unwrap().clone());
        material.set_texture("obstacleTex", obstacle_tex.clone());
        material.set_uniform("screenToWorld", [screen_w / zoom, screen_h / zoom]);
        material.set_uniform("screenSize", [screen_w, screen_h]);
        material.set_uniform("cameraPos", [camera_x, camera_y]);
        material.set_uniform("mapSize", [map_w, map_h]);
        material.set_uniform("tileSize", [32.0 as f32, 16.0 as f32]);
        material.set_uniform("time", get_time() as f32);
        material.set_uniform("numLights", active_lights.len() as i32);
        material.set_uniform("numLinearLights", linear_lights.len().min(4) as i32);
        material.set_uniform("rectOrigin", [0.0f32, 0.0f32]);
        material.set_uniform("rectSize", [screen_w, screen_h]);

        material.set_uniform("outputMode", 1i32);
        draw_rectangle(0.0, 0.0, screen_w, screen_h, WHITE);

        let mut min_x = screen_w;
        let mut min_y = screen_h;
        let mut max_x = 0.0f32;
        let mut max_y = 0.0f32;

        for l in &active_lights {
            let r = l.radius * zoom;
            let sx = (l.x - camera_x) * zoom;
            let sy = (l.y - camera_y) * zoom;
            let sx = sx.clamp(0.0, screen_w);
            let sy = sy.clamp(0.0, screen_h);
            let lx = (sx - r).max(0.0);
            let ly = (sy - r).max(0.0);
            let hx = (sx + r).min(screen_w);
            let hy = (sy + r).min(screen_h);
            if lx < min_x { min_x = lx; }
            if ly < min_y { min_y = ly; }
            if hx > max_x { max_x = hx; }
            if hy > max_y { max_y = hy; }
        }

        for bl in linear_lights.iter().take(4) {
            let sx0 = ((bl.start_x - camera_x) * zoom).clamp(0.0, screen_w);
            let sy0 = ((bl.start_y - camera_y) * zoom).clamp(0.0, screen_h);
            let sx1 = ((bl.end_x - camera_x) * zoom).clamp(0.0, screen_w);
            let sy1 = ((bl.end_y - camera_y) * zoom).clamp(0.0, screen_h);
            let pad = bl.width * zoom;
            let lx = sx0.min(sx1) - pad;
            let ly = sy0.min(sy1) - pad;
            let hx = sx0.max(sx1) + pad;
            let hy = sy0.max(sy1) + pad;
            if lx < min_x { min_x = lx.max(0.0); }
            if ly < min_y { min_y = ly.max(0.0); }
            if hx > max_x { max_x = hx.min(screen_w); }
            if hy > max_y { max_y = hy.min(screen_h); }
        }

        if max_x > min_x && max_y > min_y {
            material.set_uniform("outputMode", 2i32);
            material.set_uniform("rectOrigin", [min_x, min_y]);
            material.set_uniform("rectSize", [max_x - min_x, max_y - min_y]);
            draw_rectangle(min_x, min_y, max_x - min_x, max_y - min_y, WHITE);
        }

        gl_use_default_material();
    }

    fn update_linear_light_data_buffer(&mut self, linear_lights: &[super::map::LinearLight], map_w: f32, map_h: f32) {
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
            tex.update(&Image {
                bytes: self.linear_light_data_buffer.to_vec(),
                width: 4,
                height: 2,
            });
        }
    }
}

