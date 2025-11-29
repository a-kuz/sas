use macroquad::prelude::*;
use sas::game::md3::MD3Model;
use sas::game::player_model::PlayerModel;
use sas::game::weapon::Weapon;
use sas::game::skin_loader;
use std::collections::HashMap;
use std::sync::OnceLock;

struct Viewer {
    player_model: Option<PlayerModel>,
    weapon_model: Option<MD3Model>,
    weapon_textures: HashMap<String, Texture2D>,
    yaw: f32,
    pitch: f32,
    roll: f32,
    scale: f32,
    lower_frame: usize,
    upper_frame: usize,
}

#[derive(Clone, Copy)]
struct Orientation {
    origin: Vec3,
    axis: [Vec3; 3],
}

impl Viewer {

    fn identity_axis() -> [Vec3; 3] {
        [vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)]
    }

    fn axis_from_mat3(m: Mat3) -> [Vec3; 3] {
        let cols = m.to_cols_array();
        [
            vec3(cols[0], cols[1], cols[2]),
            vec3(cols[3], cols[4], cols[5]),
            vec3(cols[6], cols[7], cols[8]),
        ]
    }

    fn matrix_multiply_axis(a: [Vec3; 3], b: [Vec3; 3]) -> [Vec3; 3] {
        let mut out = [Vec3::ZERO; 3];
        for i in 0..3 {
            out[i].x = a[i].x * b[0].x + a[i].y * b[1].x + a[i].z * b[2].x;
            out[i].y = a[i].x * b[0].y + a[i].y * b[1].y + a[i].z * b[2].y;
            out[i].z = a[i].x * b[0].z + a[i].y * b[1].z + a[i].z * b[2].z;
        }
        out
    }

    fn orientation_to_mat4(orientation: &Orientation) -> Mat4 {
        Mat4::from_cols(
            vec4(orientation.axis[0].x, orientation.axis[0].y, orientation.axis[0].z, 0.0),
            vec4(orientation.axis[1].x, orientation.axis[1].y, orientation.axis[1].z, 0.0),
            vec4(orientation.axis[2].x, orientation.axis[2].y, orientation.axis[2].z, 0.0),
            vec4(orientation.origin.x, orientation.origin.y, orientation.origin.z, 1.0),
        )
    }

    fn attach_rotated_entity(
        parent: &Orientation,
        local_axis: [Vec3; 3],
        tag_pos: Vec3,
        tag_axis: [[f32; 3]; 3],
    ) -> Orientation {
        let mut origin = parent.origin;
        origin += parent.axis[0] * tag_pos.x;
        origin += parent.axis[1] * tag_pos.y;
        origin += parent.axis[2] * tag_pos.z;

        let tag_axis_vec = [
            vec3(tag_axis[0][0], tag_axis[0][1], tag_axis[0][2]),
            vec3(tag_axis[1][0], tag_axis[1][1], tag_axis[1][2]),
            vec3(tag_axis[2][0], tag_axis[2][1], tag_axis[2][2]),
        ];

        let temp = Self::matrix_multiply_axis(local_axis, tag_axis_vec);
        let axis = Self::matrix_multiply_axis(temp, parent.axis);
        Orientation { origin, axis }
    }
    async fn load(player_name: &str, weapon: Weapon) -> Result<Self, String> {
        let mut player_model = PlayerModel::load_async(player_name).await
            .map_err(|e| format!("Failed to load player model: {}", e))?;
        
        player_model.load_textures(player_name, "default").await;
        
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
        
        let weapon_model = MD3Model::load_async(weapon_path).await.ok();
        let mut weapon_textures = HashMap::new();
        
        let texture_path = match weapon {
            Weapon::Gauntlet => "models/weapons2/gauntlet/gauntlet.png",
            Weapon::MachineGun => "models/weapons2/machinegun/machinegun.png",
            Weapon::Shotgun => "models/weapons2/shotgun/shotgun.png",
            Weapon::GrenadeLauncher => "models/weapons2/grenadel/grenadel.png",
            Weapon::RocketLauncher => "models/weapons2/rocketl/rocketl.png",
            Weapon::Lightning => "models/weapons2/lightning/lightning.png",
            Weapon::Railgun => "models/weapons2/railgun/railgun.png",
            Weapon::Plasmagun => "models/weapons2/plasma/plasma.png",
            Weapon::BFG => "models/weapons2/bfg/bfg.png",
        };
        
        if let Some(ref weapon_mdl) = weapon_model {
            if let Some(tx) = skin_loader::load_texture_file(&format!("q3-resources/{}", texture_path)).await {
                for mesh in &weapon_mdl.meshes {
                    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                        .trim_end_matches('\0')
                        .to_string();
                    weapon_textures.insert(mesh_name.clone(), tx.clone());
                }
            }
        }
        
        Ok(Self {
            player_model: Some(player_model),
            weapon_model,
            weapon_textures,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            scale: 12.0,
            lower_frame: 0,
            upper_frame: 0,
        })
    }
    
    fn get_tag_name(tag: &sas::game::md3::Tag) -> String {
        String::from_utf8_lossy(&tag.name)
            .trim_end_matches('\0')
            .to_string()
    }
    
    fn render_mesh_3d(
        mesh: &sas::game::md3::Mesh,
        frame: usize,
        model_matrix: Mat4,
        texture: Option<&Texture2D>,
    ) {
        let safe_frame = frame.min(mesh.vertices.len().saturating_sub(1));
        let frame_verts = &mesh.vertices[safe_frame];
        if frame_verts.is_empty() || mesh.triangles.is_empty() {
            return;
        }
        
        let mut vertices: Vec<Vertex> = Vec::with_capacity(mesh.triangles.len() * 3);
        let mut indices: Vec<u16> = Vec::with_capacity(mesh.triangles.len() * 3);
        
        for tri in &mesh.triangles {
            let i0 = tri.vertex[0] as usize;
            let i1 = tri.vertex[1] as usize;
            let i2 = tri.vertex[2] as usize;
            if i0 >= frame_verts.len() || i1 >= frame_verts.len() || i2 >= frame_verts.len() {
                continue;
            }
            if i0 >= mesh.tex_coords.len() || i1 >= mesh.tex_coords.len() || i2 >= mesh.tex_coords.len() {
                continue;
            }
            let v0 = &frame_verts[i0];
            let v1 = &frame_verts[i1];
            let v2 = &frame_verts[i2];
            let p0 = model_matrix * vec4(v0.vertex[0] as f32 / 64.0, v0.vertex[1] as f32 / 64.0, v0.vertex[2] as f32 / 64.0, 1.0);
            let p1 = model_matrix * vec4(v1.vertex[0] as f32 / 64.0, v1.vertex[1] as f32 / 64.0, v1.vertex[2] as f32 / 64.0, 1.0);
            let p2 = model_matrix * vec4(v2.vertex[0] as f32 / 64.0, v2.vertex[1] as f32 / 64.0, v2.vertex[2] as f32 / 64.0, 1.0);
            let t0 = &mesh.tex_coords[i0];
            let t1 = &mesh.tex_coords[i1];
            let t2 = &mesh.tex_coords[i2];
            let base = vertices.len() as u16;
            vertices.push(Vertex { position: vec3(p0.x, p0.y, p0.z), uv: vec2(t0.coord[0], t0.coord[1]), color: [255,255,255,255], normal: vec4(0.0,0.0,1.0,0.0) });
            vertices.push(Vertex { position: vec3(p1.x, p1.y, p1.z), uv: vec2(t1.coord[0], t1.coord[1]), color: [255,255,255,255], normal: vec4(0.0,0.0,1.0,0.0) });
            vertices.push(Vertex { position: vec3(p2.x, p2.y, p2.z), uv: vec2(t2.coord[0], t2.coord[1]), color: [255,255,255,255], normal: vec4(0.0,0.0,1.0,0.0) });
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 1);
        }
        
        if !vertices.is_empty() {
            let mesh_data = macroquad::models::Mesh {
                vertices,
                indices,
                texture: texture.cloned(),
            };
            draw_mesh(&mesh_data);
        }
    }

    fn render(&self) {
        let material = simple_model_material();
        gl_use_material(material);
        
        let base_rot = Mat3::from_rotation_y(self.yaw)
            * Mat3::from_rotation_x(self.pitch)
            * Mat3::from_rotation_z(self.roll);
        let mut base_axis = Self::axis_from_mat3(base_rot);
        for axis in &mut base_axis {
            *axis *= self.scale;
        }
        let lower_orientation = Orientation {
            origin: vec3(0.0, 0.0, 0.0),
            axis: base_axis,
        };
        
        if let Some(ref player) = self.player_model {
            let mut torso_tag = None;
            
            if let Some(ref lower) = player.lower {
                let lower_frame_idx = self.lower_frame.min(lower.tags.len().saturating_sub(1));
                if let Some(tags_for_frame) = lower.tags.get(lower_frame_idx) {
                    if let Some(torso_tag_ref) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_torso") {
                        torso_tag = Some((
                            vec3(
                                torso_tag_ref.position[0],
                                torso_tag_ref.position[1],
                                torso_tag_ref.position[2],
                            ),
                            torso_tag_ref.axis,
                        ));
                    }
                }
                
                let lower_model_m = Self::orientation_to_mat4(&lower_orientation);
                for mesh in &lower.meshes {
                    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                        .trim_end_matches('\0')
                        .to_string();
                    let texture = player.textures.get(&mesh_name);
                    Self::render_mesh_3d(mesh, lower_frame_idx, lower_model_m, texture);
                }
            }
            
            let mut torso_orientation = None;
            if let Some((torso_pos, torso_axis)) = torso_tag {
                let orient = Self::attach_rotated_entity(
                    &lower_orientation,
                    Self::identity_axis(),
                    torso_pos,
                    torso_axis,
                );
                torso_orientation = Some(orient);
                
                if let Some(ref upper) = player.upper {
                    for mesh in &upper.meshes {
                        let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                            .trim_end_matches('\0')
                            .to_string();
                        let texture = player.textures.get(&mesh_name);
                        Self::render_mesh_3d(
                            mesh,
                            upper_frame_idx.min(mesh.vertices.len().saturating_sub(1)),
                            Self::orientation_to_mat4(&orient),
                            texture,
                        );
                    }
                }
            }
            
            let mut head_tag = None;
            let mut weapon_tag = None;
            let mut upper_frame_idx = self.upper_frame;
            if let Some(ref upper) = player.upper {
                upper_frame_idx = upper_frame_idx.min(upper.tags.len().saturating_sub(1));
                if let Some(tags_for_frame) = upper.tags.get(upper_frame_idx) {
                    if let Some(head_tag_ref) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_head") {
                        head_tag = Some((
                            vec3(
                                head_tag_ref.position[0],
                                head_tag_ref.position[1],
                                head_tag_ref.position[2],
                            ),
                            head_tag_ref.axis,
                        ));
                    }
                    if let Some(weapon_tag_ref) = tags_for_frame.iter().find(|t| Self::get_tag_name(t) == "tag_weapon") {
                        weapon_tag = Some((
                            vec3(
                                weapon_tag_ref.position[0],
                                weapon_tag_ref.position[1],
                                weapon_tag_ref.position[2],
                            ),
                            weapon_tag_ref.axis,
                        ));
                    }
                }
            }
            
            if let (Some(ref head_model), Some(torso_orient), Some((head_pos, head_axis))) =
                (player.head.as_ref(), torso_orientation, head_tag)
            {
                let head_orient = Self::attach_rotated_entity(
                    &torso_orient,
                    Self::identity_axis(),
                    head_pos,
                    head_axis,
                );
                for mesh in &head_model.meshes {
                    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                        .trim_end_matches('\0')
                        .to_string();
                    let texture = player.textures.get(&mesh_name);
                    Self::render_mesh_3d(mesh, 0, Self::orientation_to_mat4(&head_orient), texture);
                }
            }
            
            if let (Some(ref weapon_model), Some(torso_orient), Some((weapon_pos, weapon_axis))) =
                (self.weapon_model.as_ref(), torso_orientation, weapon_tag)
            {
                let weapon_orient = Self::attach_rotated_entity(
                    &torso_orient,
                    Self::identity_axis(),
                    weapon_pos,
                    weapon_axis,
                );
                for mesh in &weapon_model.meshes {
                    let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                        .trim_end_matches('\0')
                        .to_string();
                    let texture = self.weapon_textures.get(&mesh_name);
                    Self::render_mesh_3d(mesh, 0, Self::orientation_to_mat4(&weapon_orient), texture);
                }
            }
        }
        
        gl_use_default_material();
    }
}

fn simple_model_material() -> &'static Material {
    static MAT: OnceLock<Material> = OnceLock::new();
    MAT.get_or_init(|| {
        load_material(
            ShaderSource::Glsl {
                vertex: r#"#version 100
                attribute vec3 position;
                attribute vec2 texcoord;
                attribute vec4 color0;
                varying lowp vec2 uv;
                varying lowp vec4 color;
                uniform mat4 Model;
                uniform mat4 Projection;
                void main() {
                    gl_Position = Projection * Model * vec4(position, 1.0);
                    color = color0 / 255.0;
                    uv = texcoord;
                }"#,
                fragment: r#"#version 100
                varying lowp vec2 uv;
                varying lowp vec4 color;
                uniform sampler2D Texture;
                void main() {
                    gl_FragColor = texture2D(Texture, uv) * color;
                }"#,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: miniquad::Comparison::LessOrEqual,
                    depth_write: true,
                    cull_face: miniquad::CullFace::Back,
                    ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap()
    })
}

#[macroquad::main("MD3 Player with Weapon Viewer")]
async fn main() {
    set_pc_assets_folder(".");

    let mut viewer = Viewer::load(
        "sarge",
        Weapon::Plasmagun,
    )
    .await
    .expect("load model");

    loop {
        clear_background(BLACK);

        set_camera(&Camera3D {
            position: vec3(0.0, 0.0, 200.0),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            fovy: 45.0,
            ..Default::default()
        });

        let md = mouse_delta_position();
        viewer.yaw += md.x * 0.01;
        viewer.pitch += md.y * 0.01;

        if is_key_down(KeyCode::Q) { viewer.roll -= 0.05; }
        if is_key_down(KeyCode::E) { viewer.roll += 0.05; }
        if is_key_down(KeyCode::A) { viewer.yaw -= 0.05; }
        if is_key_down(KeyCode::D) { viewer.yaw += 0.05; }
        if is_key_down(KeyCode::W) { viewer.pitch -= 0.05; }
        if is_key_down(KeyCode::S) { viewer.pitch += 0.05; }

        if is_key_pressed(KeyCode::R) { viewer.yaw = 0.0; viewer.pitch = 0.0; viewer.roll = 0.0; }
        if is_key_down(KeyCode::Minus) { viewer.scale = (viewer.scale - 0.5).max(1.0); }
        if is_key_down(KeyCode::Equal) { viewer.scale = (viewer.scale + 0.5).min(50.0); }
        
        if let Some(ref player) = viewer.player_model {
            if let Some(ref lower) = player.lower {
                let max_lower = lower.tags.len().saturating_sub(1);
                if is_key_pressed(KeyCode::Left) { viewer.lower_frame = viewer.lower_frame.saturating_sub(1).min(max_lower); }
                if is_key_pressed(KeyCode::Right) { viewer.lower_frame = (viewer.lower_frame + 1).min(max_lower); }
            }
            if let Some(ref upper) = player.upper {
                let max_upper = upper.tags.len().saturating_sub(1);
                if is_key_pressed(KeyCode::Up) { viewer.upper_frame = viewer.upper_frame.saturating_sub(1).min(max_upper); }
                if is_key_pressed(KeyCode::Down) { viewer.upper_frame = (viewer.upper_frame + 1).min(max_upper); }
            }
        }

        while viewer.yaw > std::f32::consts::PI { viewer.yaw -= 2.0 * std::f32::consts::PI; }
        while viewer.yaw < -std::f32::consts::PI { viewer.yaw += 2.0 * std::f32::consts::PI; }
        while viewer.pitch > std::f32::consts::PI { viewer.pitch -= 2.0 * std::f32::consts::PI; }
        while viewer.pitch < -std::f32::consts::PI { viewer.pitch += 2.0 * std::f32::consts::PI; }
        while viewer.roll > std::f32::consts::PI { viewer.roll -= 2.0 * std::f32::consts::PI; }
        while viewer.roll < -std::f32::consts::PI { viewer.roll += 2.0 * std::f32::consts::PI; }

        viewer.render();

        set_default_camera();
        draw_text(&format!("Pitch: {:.2}", viewer.pitch), 10.0, 40.0, 22.0, WHITE);
        draw_text(&format!("Yaw:   {:.2}", viewer.yaw),   10.0, 65.0, 22.0, WHITE);
        draw_text(&format!("Roll:  {:.2}", viewer.roll),  10.0, 90.0, 22.0, WHITE);
        draw_text(&format!("Scale: {:.1}", viewer.scale), 10.0, 115.0, 22.0, WHITE);
        draw_text(&format!("Lower Frame: {}", viewer.lower_frame), 10.0, 140.0, 22.0, WHITE);
        draw_text(&format!("Upper Frame: {}", viewer.upper_frame), 10.0, 165.0, 22.0, WHITE);
        draw_text("Mouse X/Y: Yaw/Pitch | Q/E: Roll | A/D: Yaw | W/S: Pitch | -/+: Scale | R: Reset", 10.0, 190.0, 18.0, YELLOW);
        draw_text("Left/Right: Lower Frame | Up/Down: Upper Frame", 10.0, 215.0, 18.0, YELLOW);

        next_frame().await;
    }
}
