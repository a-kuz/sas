use macroquad::prelude::*;
use sas::game::md3::MD3Model;
use sas::game::skin_loader;
use std::collections::HashMap;
use std::sync::OnceLock;

struct Viewer {
    model: MD3Model,
    textures: HashMap<String, Texture2D>,
    yaw: f32,
    pitch: f32,
    roll: f32,
    scale: f32,
}

impl Viewer {
    async fn load_model(model_path: &str, texture_paths: &[&str]) -> Result<Self, String> {
        let model = MD3Model::load(model_path).map_err(|e| format!("{}", e))?;
        let mut textures = HashMap::new();
        for mesh in &model.meshes {
            let mesh_name = String::from_utf8_lossy(&mesh.header.name)
                .trim_end_matches('\0')
                .to_string();
            for t in texture_paths {
                if let Some(tx) = skin_loader::load_texture_file(&format!("q3-resources/{}", t)).await {
                    if !textures.contains_key(&mesh_name) {
                        textures.insert(mesh_name.clone(), tx.clone());
                    }
                }
            }
        }
        Ok(Self { model, textures, yaw: 0.0, pitch: 0.0, roll: 0.0, scale: 12.0 })
    }

    fn render(&self) {
        let rot = Mat4::from_rotation_y(self.yaw)
            * Mat4::from_rotation_x(self.pitch)
            * Mat4::from_rotation_z(self.roll);
        let model_m = Mat4::from_scale(vec3(self.scale, self.scale, self.scale)) * rot;
        let material = simple_model_material();
        gl_use_material(material);
        for mesh in &self.model.meshes {
            let frame = 0.min(mesh.vertices.len().saturating_sub(1));
            let frame_verts = &mesh.vertices[frame];
            if frame_verts.is_empty() || mesh.triangles.is_empty() {
                continue;
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
                let p0 = model_m * vec4(v0.vertex[0] as f32 / 64.0, v0.vertex[1] as f32 / 64.0, v0.vertex[2] as f32 / 64.0, 1.0);
                let p1 = model_m * vec4(v1.vertex[0] as f32 / 64.0, v1.vertex[1] as f32 / 64.0, v1.vertex[2] as f32 / 64.0, 1.0);
                let p2 = model_m * vec4(v2.vertex[0] as f32 / 64.0, v2.vertex[1] as f32 / 64.0, v2.vertex[2] as f32 / 64.0, 1.0);
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
                let mesh_name = String::from_utf8_lossy(&mesh.header.name).trim_end_matches('\0').to_string();
                let mesh_data = macroquad::models::Mesh {
                    vertices,
                    indices,
                    texture: self.textures.get(&mesh_name).cloned(),
                };
                draw_mesh(&mesh_data);
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

#[macroquad::main("MD3 Simple Viewer")]
async fn main() {
    set_pc_assets_folder(".");

    let mut viewer = Viewer::load_model(
        "q3-resources/models/weapons2/plasma/plasma.md3",
        &["models/weapons2/plasma/plasma.png"],
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
        draw_text("Mouse X/Y: Yaw/Pitch | Q/E: Roll | A/D: Yaw | W/S: Pitch | -/+: Scale | R: Reset", 10.0, 145.0, 18.0, YELLOW);

        next_frame().await;
    }
}
