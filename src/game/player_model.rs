use crate::game::md3::MD3Model;
use crate::game::md3_anim::AnimConfig;
use crate::game::weapon::Weapon;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct PlayerModel {
    pub lower: Option<MD3Model>,
    pub upper: Option<MD3Model>,
    pub head: Option<MD3Model>,
    pub weapon: Option<MD3Model>,
    pub anim_config: Option<AnimConfig>,
    pub textures: HashMap<String, Texture2D>,
    pub texture_paths: HashMap<String, String>,
    pub shader_textures: HashMap<String, Vec<Texture2D>>,
}

impl PlayerModel {
    
    fn should_skip_quad_for_texture(texture_path: Option<&str>) -> bool {
        if let Some(path) = texture_path {
            let path_lower = path.to_lowercase();
            path_lower.contains("_h.") || path_lower.contains("_h/") ||
            path_lower.contains("_a.") || path_lower.contains("_a/") ||
            path_lower.contains("_q.") || path_lower.contains("_q/") ||
            path_lower.contains("skate") || 
            path_lower.contains("null") ||
            path_lower.contains("_f.") ||
            path_lower.contains("/f_")
        } else {
            false
        }
    }
    
    pub fn load(model_name: &str) -> Result<Self, String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let base_path = format!("q3-resources/models/players/{}", model_name);
            
            println!("[MD3] Loading model: {}", model_name);
            
            let lower = match super::md3::MD3Model::load(format!("{}/lower.md3", base_path)) {
                Ok(model) => {
                    println!("[MD3] ✓ Loaded lower.md3: {} frames, {} meshes", 
                        model.header.num_bone_frames, model.header.num_meshes);
                    Some(model)
                }
                Err(e) => {
                    println!("[MD3] ✗ Failed to load lower.md3: {}", e);
                    None
                }
            };
            
            let upper = match super::md3::MD3Model::load(format!("{}/upper.md3", base_path)) {
                Ok(model) => {
                    println!("[MD3] ✓ Loaded upper.md3: {} frames, {} meshes", 
                        model.header.num_bone_frames, model.header.num_meshes);
                    Some(model)
                }
                Err(e) => {
                    println!("[MD3] ✗ Failed to load upper.md3: {}", e);
                    None
                }
            };
            
            let head = match super::md3::MD3Model::load(format!("{}/head.md3", base_path)) {
                Ok(model) => {
                    println!("[MD3] ✓ Loaded head.md3: {} frames, {} meshes", 
                        model.header.num_bone_frames, model.header.num_meshes);
                    Some(model)
                }
                Err(e) => {
                    println!("[MD3] ✗ Failed to load head.md3: {}", e);
                    None
                }
            };
            
            if lower.is_none() && upper.is_none() && head.is_none() {
                return Err(format!("Failed to load any model parts for {}", model_name));
            }
            
            let anim_config = AnimConfig::load(model_name).ok();
            if anim_config.is_some() {
                println!("[MD3] ✓ Loaded animation.cfg");
            } else {
                println!("[MD3] ✗ No animation.cfg found");
            }
            
            println!("[MD3] Model '{}' loaded successfully!", model_name);
            
            Ok(Self {
                lower,
                upper,
                head,
                weapon: None,
                anim_config,
                textures: HashMap::new(),
                texture_paths: HashMap::new(),
                shader_textures: HashMap::new(),
            })
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            Err("Use load_async for WASM".to_string())
        }
    }
    
    pub async fn load_async(model_name: &str) -> Result<Self, String> {
        let base_path = format!("q3-resources/models/players/{}", model_name);
        
        println!("[MD3] Loading model: {}", model_name);
        
        let lower = match MD3Model::load_async(&format!("{}/lower.md3", base_path)).await {
            Ok(model) => {
                println!("[MD3] ✓ Loaded lower.md3: {} frames, {} meshes", 
                    model.header.num_bone_frames, model.header.num_meshes);
                Some(model)
            }
            Err(e) => {
                println!("[MD3] ✗ Failed to load lower.md3: {}", e);
                None
            }
        };
        
        let upper = match MD3Model::load_async(&format!("{}/upper.md3", base_path)).await {
            Ok(model) => {
                println!("[MD3] ✓ Loaded upper.md3: {} frames, {} meshes", 
                    model.header.num_bone_frames, model.header.num_meshes);
                Some(model)
            }
            Err(e) => {
                println!("[MD3] ✗ Failed to load upper.md3: {}", e);
                None
            }
        };
        
        let head = match MD3Model::load_async(&format!("{}/head.md3", base_path)).await {
            Ok(model) => {
                println!("[MD3] ✓ Loaded head.md3: {} frames, {} meshes", 
                    model.header.num_bone_frames, model.header.num_meshes);
                Some(model)
            }
            Err(e) => {
                println!("[MD3] ✗ Failed to load head.md3: {}", e);
                None
            }
        };
        
        if lower.is_none() && upper.is_none() && head.is_none() {
            return Err(format!("Failed to load any model parts for {}", model_name));
        }
        
        let anim_config = AnimConfig::load_async(model_name).await.ok();
        if anim_config.is_some() {
            println!("[MD3] ✓ Loaded animation.cfg");
        } else {
            println!("[MD3] ✗ No animation.cfg found");
        }
        
        println!("[MD3] Model '{}' loaded successfully!", model_name);
        
        Ok(Self {
            lower,
            upper,
            head,
            weapon: None,
            anim_config,
            textures: HashMap::new(),
            texture_paths: HashMap::new(),
            shader_textures: HashMap::new(),
        })
    }
    
    pub async fn load_weapon(&mut self, weapon: Weapon) {
        let weapon_path = match weapon {
            Weapon::Gauntlet => "q3-resources/models/weapons2/gauntlet/gauntlet.md3",
            Weapon::MachineGun => "q3-resources/models/weapons2/machinegun/machinegun.md3",
            Weapon::Shotgun => "q3-resources/models/weapons2/shotgun/shotgun.md3",
            Weapon::GrenadeLauncher => "q3-resources/models/weapons2/grenadel/grenadel.md3",
            Weapon::RocketLauncher => "q3-resources/models/weapons2/rocketl/rocketl.md3",
            Weapon::Lightning => "q3-resources/models/weapons2/lightning/lightning.md3",
            Weapon::Railgun => "q3-resources/models/weapons2/railgun/railgun.md3",
            Weapon::Plasmagun => "q3-resources/models/weapons2/plasma/plasma.md3",
            Weapon::BFG => "q3-resources/models/weapons2/bfg/bfg.md3",
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            self.weapon = MD3Model::load_async(weapon_path).await.ok();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.weapon = MD3Model::load(weapon_path).ok();
        }
    }
    
    pub async fn load_textures(&mut self, model_name: &str, skin_name: &str) {
        println!("[MD3] Loading textures for {} with skin {}", model_name, skin_name);
        self.textures.clear();
        self.texture_paths.clear();
        self.shader_textures.clear();

        let mut skins = vec![skin_name.to_string(), "default".to_string(), "red".to_string(), "blue".to_string()];
        skins.dedup();

        for part in ["lower", "upper", "head"] {
            let mut loaded = false;
            for skin in skins.iter() {
                if let Ok(mapping) = super::skin_loader::SkinMapping::load(model_name, skin, part).await {
                    for (mesh_name, texture_path) in mapping.mesh_textures {
                        if let Some(texture) = super::skin_loader::load_texture_file(&texture_path).await {
                            println!("[Texture] ✓ Mapped {} -> {}", mesh_name, texture_path);
                            self.texture_paths.insert(mesh_name.clone(), texture_path.clone());
                            self.textures.insert(mesh_name.clone(), texture);
                            
                            self.load_shader_textures(&mesh_name, &texture_path).await;
                            
                            loaded = true;
                        }
                    }
                    if loaded { break; }
                }
            }

            if !loaded {
                let dir = format!("q3-resources/models/players/{}", model_name);
                if let Ok(entries) = super::file_loader::read_dir(&dir) {
                    for name in entries {
                                    if name.starts_with(&format!("{}_", part)) && name.ends_with(".skin") {
                                        if let Some(pos) = name.find('_') {
                                            if let Some(dot) = name.rfind('.') {
                                                if pos + 1 < dot {
                                                    let alt = &name[pos + 1..dot];
                                        if let Ok(mapping) = super::skin_loader::SkinMapping::load(model_name, alt, part).await {
                                                        for (mesh_name, texture_path) in mapping.mesh_textures {
                                                            if let Some(texture) = super::skin_loader::load_texture_file(&texture_path).await {
                                                                println!("[Texture] ✓ Mapped {} -> {}", mesh_name, texture_path);
                                                                self.texture_paths.insert(mesh_name.clone(), texture_path.clone());
                                                                self.textures.insert(mesh_name.clone(), texture);
                                                                
                                                                self.load_shader_textures(&mesh_name, &texture_path).await;
                                                                
                                                                let _ = loaded;
                                                            }
                                                        }
                                                    }
                                                    break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    async fn load_shader_textures(&mut self, mesh_name: &str, base_texture_path: &str) {
        let shader_name = base_texture_path
            .replace("q3-resources/", "")
            .replace(".png", "")
            .replace(".tga", "");
        
        let mut additional_textures = Vec::new();
        
        if shader_name.contains("xaero_h") {
            if let Some(fire_tex) = super::skin_loader::load_texture_file("q3-resources/textures/sfx/firewalla.png").await {
                println!("[Shader] ✓ Loaded fire texture for {}", mesh_name);
                additional_textures.push(fire_tex);
            }
        } else if shader_name.contains("xaero_a") {
            if let Some(env_tex) = super::skin_loader::load_texture_file("q3-resources/textures/effects/envmapbfg.png").await {
                println!("[Shader] ✓ Loaded envmap texture for {}", mesh_name);
                additional_textures.push(env_tex);
            }
        }
        
        if !additional_textures.is_empty() {
            self.shader_textures.insert(mesh_name.to_string(), additional_textures);
        }
    }
    
    
    fn get_mesh_name(mesh_header: &super::md3::MeshHeader) -> String {
        String::from_utf8_lossy(&mesh_header.name)
            .trim_end_matches('\0')
            .to_string()
    }
    
    fn get_tag_name(tag: &super::md3::Tag) -> String {
        String::from_utf8_lossy(&tag.name)
            .trim_end_matches('\0')
            .to_string()
    }
    
    pub fn render_simple(&self, screen_x: f32, screen_y: f32, color: Color, scale: f32, flip_x: bool, pitch: f32, aim_angle: f32, lower_frame: usize, upper_frame: usize, weapon_model: Option<&crate::game::weapon_model_cache::WeaponModel>, _debug_md3: bool, lighting_context: Option<&super::md3_render::LightingContext>, model_yaw_offset: f32, model_roll: f32, somersault_angle: f32, has_quad_damage: bool, barrel_roll: f32) {
        let draw_gun = crate::cvar::get_cvar_integer("cg_drawGun");
        let _weapon_to_render = if draw_gun == 0 { None } else { weapon_model };
        let base_y_offset = 20.0 * (scale / 1.5);
        let base_y = screen_y - base_y_offset;
        let torso_offset_y = 0.0;
        let x_mult = if flip_x { -1.0 } else { 1.0 };
        
        let legs_yaw = if pitch.abs() > 0.3 {
            let intensity = ((pitch.abs() - 0.3) / 1.2).min(1.0);
            pitch.signum() * intensity * 1.2
        } else {
            0.0
        };
        
        let legs_total_yaw = model_yaw_offset + legs_yaw;
        
        let mut torso_tag_x = 0.0;
        let mut torso_tag_z = 0.0;
        
        // Invert roll angles if flipped to maintain correct rotation direction
        // Legs use Standard rotation (X, Z), Torso uses Pivot rotation (X, -Z).
        // To match directions: Right (!flip) uses +roll, Left (flip) uses -roll.
        let effective_model_roll = if flip_x { -model_roll } else { model_roll };
        let effective_legs_roll = if flip_x { somersault_angle } else { -somersault_angle };

        if let Some(ref lower) = self.lower {
            let safe_frame = lower_frame.min(lower.tags.len().saturating_sub(1));
            if let Some(tags_for_frame) = lower.tags.get(safe_frame) {
        if let Some(torso_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_torso") {
            // Legs -> Torso (Standard Rotation)
            torso_tag_x = torso_tag.position[0] * scale * x_mult; // Flip first
            torso_tag_z = torso_tag.position[2] * scale;
            
            if effective_legs_roll.abs() > 0.01 {
                let cos_r = effective_legs_roll.cos();
                let sin_r = effective_legs_roll.sin();
                let rotated_x = torso_tag_x * cos_r - torso_tag_z * sin_r;
                let rotated_z = torso_tag_x * sin_r + torso_tag_z * cos_r;
                torso_tag_x = rotated_x;
                torso_tag_z = rotated_z;
            }
        }
            }
            
            for (_mesh_idx, mesh) in lower.meshes.iter().enumerate() {
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let texture = self.textures.get(&mesh_name);
                let texture_path = self.texture_paths.get(&mesh_name);
                let shader_textures = self.shader_textures.get(&mesh_name);
                let safe_frame = lower_frame.min(mesh.vertices.len().saturating_sub(1));
                
                super::md3_render::render_md3_mesh_with_yaw_and_roll_shader(
                    mesh,
                    safe_frame,
                    screen_x,
                    base_y - torso_offset_y,
                    scale,
                    color,
                    texture,
                    texture_path.map(|s| s.as_str()),
                    shader_textures.map(|v| v.as_slice()),
                    flip_x,
                    0.0,
                    legs_total_yaw,
                    effective_legs_roll,
                    lighting_context,
                );

                if has_quad_damage && !Self::should_skip_quad_for_texture(texture_path.map(|s| s.as_str())) {
                    super::md3_render::render_md3_mesh_with_yaw_and_roll_quad(
                        mesh,
                        safe_frame,
                        screen_x,
                        base_y - torso_offset_y,
                        scale,
                        color,
                        texture,
                        texture_path.map(|s| s.as_str()),
                        flip_x,
                        0.0,
                        legs_total_yaw,
                        effective_legs_roll,
                        lighting_context,
                    );
                }
            }
        }
        
        let mut head_tag_x = 0.0;
        let mut head_tag_z = 0.0;
        let mut weapon_tag_x = 0.0;
        let mut weapon_tag_z = 0.0;
        
        if let Some(ref upper) = self.upper {
            // Torso origin (Standard: y = base - z)
            // Note: torso_tag_x is already flipped and rotated, so just add it.
            let torso_origin_x = screen_x + torso_tag_x; 
            let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z;
            
            let safe_frame = upper_frame.min(upper.tags.len().saturating_sub(1));
            if let Some(tags_for_frame) = upper.tags.get(safe_frame) {
                if let Some(head_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_head") {
                    head_tag_x = head_tag.position[0] * scale;
                    head_tag_z = head_tag.position[2] * scale;
                }
                if let Some(weapon_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_weapon") {
                    weapon_tag_x = weapon_tag.position[0] * scale;
                    weapon_tag_z = weapon_tag.position[2] * scale;
                }
            }
            
            for (_mesh_idx, mesh) in upper.meshes.iter().enumerate() {
                
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let texture = self.textures.get(&mesh_name);
                let texture_path = self.texture_paths.get(&mesh_name);
                let shader_textures = self.shader_textures.get(&mesh_name);
                let safe_frame = upper_frame.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_shader(
                    mesh,
                    safe_frame,
                    torso_origin_x,
                    torso_origin_y,
                    scale,
                    color,
                    texture,
                    texture_path.map(|s| s.as_str()),
                    shader_textures.map(|v| v.as_slice()),
                    flip_x,
                    pitch,
                    model_yaw_offset,
                    0.0,
                    0.0,
                    effective_model_roll,
                );

                if has_quad_damage && !Self::should_skip_quad_for_texture(texture_path.map(|s| s.as_str())) {
                    super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_quad(
                        mesh,
                        safe_frame,
                        torso_origin_x,
                        torso_origin_y,
                        scale,
                        color,
                        texture,
                        texture_path.map(|s| s.as_str()),
                        flip_x,
                        pitch,
                        model_yaw_offset,
                        0.0,
                        0.0,
                        effective_model_roll,
                    );
                }
            }
        }
        
        if let Some(weapon) = weapon_model {
            let torso_origin_x = screen_x + torso_tag_x;
            let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z;
            
            #[cfg(target_arch = "wasm32")]
            draw_circle(screen_x + 30.0, screen_y - 20.0, 5.0, RED);

            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let cos_y = model_yaw_offset.cos();
            let sin_y = model_yaw_offset.sin();
            let cos_r = effective_model_roll.cos();
            let sin_r = effective_model_roll.sin();
            
            let weapon_x = weapon_tag_x * x_mult;
            let weapon_z = -weapon_tag_z;
            
            // Pitch
            let rotated_weapon_x = weapon_x * cos_p - weapon_z * sin_p;
            let rotated_weapon_y = weapon_x * sin_p + weapon_z * cos_p;
            
            // Yaw
            let yaw_rotated_weapon_x = rotated_weapon_x * cos_y - rotated_weapon_y * sin_y;
            let yaw_rotated_weapon_y = rotated_weapon_x * sin_y + rotated_weapon_y * cos_y;
            
            // Roll (Pivot Logic)
            let roll_rotated_x = yaw_rotated_weapon_x * cos_r - yaw_rotated_weapon_y * sin_r;
            let roll_rotated_y = yaw_rotated_weapon_x * sin_r + yaw_rotated_weapon_y * cos_r;
            
            let origin_x = torso_origin_x + roll_rotated_x;
            let origin_y = torso_origin_y + roll_rotated_y;
            
            let weapon_color = WHITE;

            let weapon_angle = pitch;

            if !weapon.models.is_empty() {
                let base_model = &weapon.models[0];
                for mesh in &base_model.meshes {
                    let mesh_name = Self::get_mesh_name(&mesh.header);
                    let texture = weapon.textures.get(&mesh_name);
                    let safe_frame = 0.min(mesh.vertices.len().saturating_sub(1));
                    super::md3_render::render_md3_mesh_with_pivot(
                        mesh,
                        safe_frame,
                        origin_x,
                        origin_y,
                        scale * 1.0,
                        weapon_color,
                        texture,
                        flip_x,
                        weapon_angle,
                        0.0,
                        0.0,
                    );

                    if has_quad_damage {
                        super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_quad(
                            mesh,
                            safe_frame,
                            origin_x,
                            origin_y,
                            scale * 1.0,
                            weapon_color,
                            texture,
                            None,
                            flip_x,
                            weapon_angle,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                        );
                    }
                }

                let mut barrel_dx = 0.0f32;
                let mut barrel_dy = 0.0f32;
                if !base_model.tags.is_empty() {
                    let tags_frame0 = &base_model.tags[0];
                    if let Some(tag) = tags_frame0.iter().find(|t| Self::get_tag_name(t) == "tag_barrel") {
                        let bx = (tag.position[0] * scale) * if flip_x { -1.0 } else { 1.0 };
                        let bz = -(tag.position[2] * scale);
                        let r1x = bx * cos_p - bz * sin_p;
                        let r1y = bx * sin_p + bz * cos_p;
                        let r2x = r1x * cos_y - r1y * sin_y;
                        let r2y = r1x * sin_y + r1y * cos_y;
                        let r3x = r2x * cos_r - r2y * sin_r;
                        let r3y = r2x * sin_r + r2y * cos_r;
                        barrel_dx = r3x;
                        barrel_dy = r3y;
                    }
                }

                for child_model in weapon.models.iter().skip(1) {
                    for mesh in &child_model.meshes {
                        let mesh_name = Self::get_mesh_name(&mesh.header);
                        let texture = weapon.textures.get(&mesh_name);
                        let safe_frame = 0.min(mesh.vertices.len().saturating_sub(1));
                        super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_with_barrel(
                            mesh,
                            safe_frame,
                            origin_x + barrel_dx,
                            origin_y + barrel_dy,
                            scale * 1.0,
                            weapon_color,
                            texture,
                            None,
                            flip_x,
                            weapon_angle,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            barrel_roll,
                        );

                        if has_quad_damage {
                            super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_quad_with_barrel(
                                mesh,
                                safe_frame,
                                origin_x + barrel_dx,
                                origin_y + barrel_dy,
                                scale * 1.0,
                                weapon_color,
                                texture,
                                None,
                                flip_x,
                                weapon_angle,
                                0.0,
                                0.0,
                                0.0,
                                0.0,
                                barrel_roll,
                            );
                        }
                    }
                }
            }
        }
        
        if let Some(ref head) = self.head {
            let torso_origin_x = screen_x + torso_tag_x;
            let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z;

            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let cos_y = model_yaw_offset.cos();
            let sin_y = model_yaw_offset.sin();
            
            let head_x = head_tag_x * x_mult;
            let head_z = -head_tag_z;
            
            let rotated_head_x = head_x * cos_p - head_z * sin_p;
            let rotated_head_y = head_x * sin_p + head_z * cos_p;
            
            let yaw_rotated_x = rotated_head_x * cos_y - rotated_head_y * sin_y;
            let yaw_rotated_y = rotated_head_x * sin_y + rotated_head_y * cos_y;
            
            let cos_r = effective_model_roll.cos();
            let sin_r = effective_model_roll.sin();

            // Roll (Pivot Logic: X, -Z -> X, Y)
            let roll_rotated_x = yaw_rotated_x * cos_r - yaw_rotated_y * sin_r;
            let roll_rotated_y = yaw_rotated_x * sin_r + yaw_rotated_y * cos_r;
            
            let head_origin_x = torso_origin_x + roll_rotated_x;
            let head_origin_y = torso_origin_y + roll_rotated_y;

            let base_dir = if flip_x { std::f32::consts::PI } else { 0.0 };
            let mut rel = aim_angle - base_dir;
            while rel > std::f32::consts::PI { rel -= 2.0 * std::f32::consts::PI; }
            while rel < -std::f32::consts::PI { rel += 2.0 * std::f32::consts::PI; }
            let max_turn = 0.7_f32;
            rel = rel.clamp(-max_turn, max_turn);
            let head_angle = rel * 0.75;

            for (_mesh_idx, mesh) in head.meshes.iter().enumerate() {
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let texture = self.textures.get(&mesh_name);
                let texture_path = self.texture_paths.get(&mesh_name);
                let shader_textures = self.shader_textures.get(&mesh_name);
                let safe_frame = 0.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_shader(
                    mesh,
                    safe_frame,
                    head_origin_x,
                    head_origin_y,
                    scale,
                    color,
                    texture,
                    texture_path.map(|s| s.as_str()),
                    shader_textures.map(|v| v.as_slice()),
                    flip_x,
                    pitch + head_angle,
                    model_yaw_offset,
                    0.0,
                    0.0,
                    -effective_model_roll,
                );

                if has_quad_damage && !Self::should_skip_quad_for_texture(texture_path.map(|s| s.as_str())) {
                    super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_quad(
                        mesh,
                        safe_frame,
                        head_origin_x,
                        head_origin_y,
                        scale,
                        color,
                        texture,
                        None,
                        flip_x,
                        pitch + head_angle,
                        model_yaw_offset,
                        0.0,
                        0.0,
                        -effective_model_roll,
                    );
                }
            }
        }
    }

    pub fn render_shadow_with_light(&self, screen_x: f32, screen_y: f32, light_sx: f32, light_sy: f32, light_radius: f32, scale: f32, flip_x: bool, pitch: f32, aim_angle: f32, lower_frame: usize, upper_frame: usize, weapon_model: Option<&crate::game::weapon_model_cache::WeaponModel>, model_yaw_offset: f32, color: Color, barrel_roll: f32) {
        let draw_gun = crate::cvar::get_cvar_integer("cg_drawGun");
        let weapon_to_render = if draw_gun == 0 { None } else { weapon_model };
        let base_y_offset = 20.0 * (scale / 1.5);
        let base_y = screen_y - base_y_offset;
        let torso_offset_y = 0.0;
        let x_mult = if flip_x { -1.0 } else { 1.0 };
        let shadow_color = color;
        let dir_x = screen_x - light_sx;
        let dir_y = screen_y - light_sy;
        let dist = (dir_x * dir_x + dir_y * dir_y).sqrt().max(1.0);
        let n_x = dir_x / dist;
        let proximity = (1.0 - (dist / light_radius)).clamp(0.0, 1.0);
        let shadow_scale = scale * (1.0 + 0.35 * proximity);
        let offset_len = 14.0 + 40.0 * proximity;
        let dx = n_x * offset_len;
        let dy = -6.0 - 8.0 * proximity;

        let mut torso_tag_x = 0.0;
        let mut torso_tag_z = 0.0;

        if let Some(ref lower) = self.lower {
            let safe_frame = lower_frame.min(lower.tags.len().saturating_sub(1));
            if let Some(tags_for_frame) = lower.tags.get(safe_frame) {
                if let Some(torso_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_torso") {
                    torso_tag_x = torso_tag.position[0] * shadow_scale;
                    torso_tag_z = torso_tag.position[2] * shadow_scale;
                }
            }

            for mesh in lower.meshes.iter() {
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let _texture = self.textures.get(&mesh_name);
                let safe_frame = lower_frame.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_yaw(
                    mesh,
                    safe_frame,
                    screen_x + dx,
                    base_y - torso_offset_y + dy,
                    shadow_scale,
                    shadow_color,
                    None,
                    flip_x,
                    0.0,
                    model_yaw_offset,
                    None,
                );
            }
        }

        let mut head_tag_x = 0.0;
        let mut head_tag_z = 0.0;
        let mut weapon_tag_x = 0.0;
        let mut weapon_tag_z = 0.0;

        if let Some(ref upper) = self.upper {
            let torso_origin_x = screen_x + torso_tag_x * x_mult + dx;
            let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z + dy;

            let safe_frame = upper_frame.min(upper.tags.len().saturating_sub(1));
            if let Some(tags_for_frame) = upper.tags.get(safe_frame) {
                if let Some(head_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_head") {
                    head_tag_x = head_tag.position[0] * shadow_scale;
                    head_tag_z = head_tag.position[2] * shadow_scale;
                }
                if let Some(weapon_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_weapon") {
                    weapon_tag_x = weapon_tag.position[0] * shadow_scale;
                    weapon_tag_z = weapon_tag.position[2] * shadow_scale;
                }
            }

            for mesh in upper.meshes.iter() {
                let mesh_name = Self::get_mesh_name(&mesh.header);
                let _texture = self.textures.get(&mesh_name);
                let safe_frame = upper_frame.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_pivot(
                    mesh,
                    safe_frame,
                    torso_origin_x,
                    torso_origin_y,
                    shadow_scale,
                    shadow_color,
                    None,
                    flip_x,
                    pitch,
                    0.0,
                    0.0,
                );
            }
        }

                if let Some(weapon) = weapon_to_render {
            let torso_origin_x = screen_x + torso_tag_x * x_mult + dx;
            let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z + dy;

            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let rotated_weapon_x = (weapon_tag_x * x_mult) * cos_p - (-weapon_tag_z) * sin_p;
            let rotated_weapon_y = (weapon_tag_x * x_mult) * sin_p + (-weapon_tag_z) * cos_p;

            let origin_x = torso_origin_x + rotated_weapon_x;
            let origin_y = torso_origin_y + rotated_weapon_y;

            let weapon_angle = pitch;

            if !weapon.models.is_empty() {
                let base_model = &weapon.models[0];
                for mesh in &base_model.meshes {
                    let safe_frame = 0.min(mesh.vertices.len().saturating_sub(1));
                    super::md3_render::render_md3_mesh_with_pivot(
                        mesh,
                        safe_frame,
                        origin_x,
                        origin_y,
                        shadow_scale * 1.0,
                        shadow_color,
                        None,
                        flip_x,
                        weapon_angle,
                        0.0,
                        0.0,
                    );
                }

                let mut barrel_dx = 0.0f32;
                let mut barrel_dy = 0.0f32;
                if !base_model.tags.is_empty() {
                    let tags_frame0 = &base_model.tags[0];
                    if let Some(tag) = tags_frame0.iter().find(|t| Self::get_tag_name(t) == "tag_barrel") {
                        let bx = (tag.position[0] * shadow_scale) * if flip_x { -1.0 } else { 1.0 };
                        let bz = -(tag.position[2] * shadow_scale);
                        let rdx = bx * cos_p - bz * sin_p;
                        let rdy = bx * sin_p + bz * cos_p;
                        barrel_dx = rdx;
                        barrel_dy = rdy;
                    }
                }

                for child_model in weapon.models.iter().skip(1) {
                    for mesh in &child_model.meshes {
                        let safe_frame = 0.min(mesh.vertices.len().saturating_sub(1));
                        super::md3_render::render_md3_mesh_with_pivot_and_yaw_ex_with_barrel(
                            mesh,
                            safe_frame,
                            origin_x + barrel_dx,
                            origin_y + barrel_dy,
                            shadow_scale * 1.0,
                            shadow_color,
                            None,
                            None,
                            flip_x,
                            weapon_angle,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            barrel_roll,
                        );
                    }
                }
            }
        }

        if let Some(ref head) = self.head {
            let torso_origin_x = screen_x + torso_tag_x * x_mult + dx;
            let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z + dy;

            let cos_p = pitch.cos();
            let sin_p = pitch.sin();
            let rotated_head_x = (head_tag_x * x_mult) * cos_p - (-head_tag_z) * sin_p;
            let rotated_head_y = (head_tag_x * x_mult) * sin_p + (-head_tag_z) * cos_p;
            
            let head_origin_x = torso_origin_x + rotated_head_x;
            let head_origin_y = torso_origin_y + rotated_head_y;

            let base_dir = if flip_x { std::f32::consts::PI } else { 0.0 };
            let mut rel = aim_angle - base_dir;
            while rel > std::f32::consts::PI { rel -= 2.0 * std::f32::consts::PI; }
            while rel < -std::f32::consts::PI { rel += 2.0 * std::f32::consts::PI; }
            let max_turn = 0.7_f32;
            rel = rel.clamp(-max_turn, max_turn);
            let head_angle = rel * 0.75;

            for mesh in head.meshes.iter() {
                let safe_frame = 0.min(mesh.vertices.len().saturating_sub(1));
                super::md3_render::render_md3_mesh_with_pivot(
                    mesh,
                    safe_frame,
                    head_origin_x,
                    head_origin_y,
                    shadow_scale,
                    shadow_color,
                    None,
                    flip_x,
                    pitch + head_angle,
                    0.0,
                    0.0,
                );
            }
        }
    }

    pub fn compute_frames(
        config: &AnimConfig,
        dt: f32,
        is_walking: bool,
        is_attacking: bool,
        on_ground: bool,
        is_dead: bool,
        is_crouching: bool,
        animation_time: f32,
    ) -> (usize, usize, f32) {
        let legs_anim = if is_dead {
            &config.death1
        } else if !on_ground {
            &config.legs_jump
        } else if is_crouching && is_walking {
            &config.legs_walkcr
        } else if is_crouching {
            &config.legs_idlecr
        } else if is_walking {
            &config.legs_run
        } else {
            &config.legs_idle
        };

        let torso_anim = if is_dead {
            &config.death1
        } else if is_attacking {
            &config.torso_attack
        } else {
            &config.torso_stand
        };

        let new_time = animation_time + dt;

        let legs_fps = legs_anim.fps as f32;
        let legs_frame_in_anim = (new_time * legs_fps) as usize;
        let legs_frame_offset = if legs_anim.looping_frames > 0 && !is_dead {
            legs_frame_in_anim % legs_anim.looping_frames
        } else {
            legs_frame_in_anim.min(legs_anim.num_frames.saturating_sub(1))
        };
        let lower_frame = (legs_anim.first_frame + legs_frame_offset).min(190);

        let torso_fps = torso_anim.fps as f32;
        let torso_frame_in_anim = (new_time * torso_fps) as usize;
        let torso_frame_offset = if torso_anim.looping_frames > 0 && !is_dead {
            torso_frame_in_anim % torso_anim.looping_frames
        } else {
            torso_frame_in_anim.min(torso_anim.num_frames.saturating_sub(1))
        };
        let upper_frame = (torso_anim.first_frame + torso_frame_offset).min(152);

        (lower_frame, upper_frame, new_time)
    }
    
    pub fn get_barrel_position(
        &self,
        player_x: f32,
        player_y: f32,
        flip_x: bool,
        pitch: f32,
        aim_angle: f32,
        lower_frame: usize,
        upper_frame: usize,
        weapon_model: Option<&crate::game::weapon_model_cache::WeaponModel>,
    ) -> (f32, f32) {
        let draw_gun = crate::cvar::get_cvar_integer("cg_drawGun");
        let weapon_to_render = if draw_gun == 0 { None } else { weapon_model };
        
        let scale = 2.0;
        let base_y_offset = 20.0 * (scale / 1.5);
        let base_y = player_y - base_y_offset;
        let torso_offset_y = 0.0;
        let x_mult = if flip_x { -1.0 } else { 1.0 };
        
        let mut torso_tag_x = 0.0;
        let mut torso_tag_z = 0.0;
        
        if let Some(ref lower) = self.lower {
            let safe_frame = lower_frame.min(lower.tags.len().saturating_sub(1));
            if let Some(tags_for_frame) = lower.tags.get(safe_frame) {
                if let Some(torso_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_torso") {
                    torso_tag_x = torso_tag.position[0] * scale;
                    torso_tag_z = torso_tag.position[2] * scale;
                }
            }
        }
        
        let mut weapon_tag_x = 0.0;
        let mut weapon_tag_z = 0.0;
        
        if let Some(ref upper) = self.upper {
            let safe_frame = upper_frame.min(upper.tags.len().saturating_sub(1));
            if let Some(tags_for_frame) = upper.tags.get(safe_frame) {
                if let Some(weapon_tag) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_weapon") {
                    weapon_tag_x = weapon_tag.position[0] * scale;
                    weapon_tag_z = weapon_tag.position[2] * scale;
                }
            }
        }
        
        let torso_origin_x = player_x + torso_tag_x * x_mult;
        let torso_origin_y = (base_y - torso_offset_y) - torso_tag_z;
        
        let cos_p = pitch.cos();
        let sin_p = pitch.sin();
        let rotated_weapon_x = (weapon_tag_x * x_mult) * cos_p - (-weapon_tag_z) * sin_p;
        let rotated_weapon_y = (weapon_tag_x * x_mult) * sin_p + (-weapon_tag_z) * cos_p;
        
        let weapon_origin_x = torso_origin_x + rotated_weapon_x;
        let weapon_origin_y = torso_origin_y + rotated_weapon_y;
        
        let mut barrel_world_x = weapon_origin_x;
        let mut barrel_world_y = weapon_origin_y;
        
                if let Some(weapon) = weapon_to_render {
            if !weapon.models.is_empty() {
                let base_model = &weapon.models[0];
                if !base_model.tags.is_empty() {
                    let tags_frame0 = &base_model.tags[0];
                    if let Some(tag) = tags_frame0.iter().find(|t| Self::get_tag_name(t) == "tag_barrel") {
                        let tx = tag.position[0] * scale;
                        let tz = tag.position[2] * scale;
                        let weapon_angle = aim_angle;
                        let barrel_local_x = tx * weapon_angle.cos() - (-tz) * weapon_angle.sin();
                        let barrel_local_y = tx * weapon_angle.sin() + (-tz) * weapon_angle.cos();
                        barrel_world_x = weapon_origin_x + barrel_local_x * if flip_x { -1.0 } else { 1.0 };
                        barrel_world_y = weapon_origin_y + barrel_local_y;
                    }
                }
            }
        }
        
        (barrel_world_x, barrel_world_y)
    }
}

