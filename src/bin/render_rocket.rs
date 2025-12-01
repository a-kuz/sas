use macroquad::prelude::*;
use sas::game::md3::MD3Model;
use sas::game::md3_render;
use std::f32::consts::PI;

#[macroquad::main("Render Rocket")]
async fn main() {
    set_pc_assets_folder(".");

    let render_target = render_target(800, 600);
    render_target.texture.set_filter(FilterMode::Nearest);

    let model = MD3Model::load("q3-resources/models/ammo/rocket/rocket.md3").unwrap();
    let texture_bytes = std::fs::read("q3-resources/models/ammo/rocket/rocket.png").unwrap();
    let texture = Texture2D::from_file_with_format(&texture_bytes, None);
    texture.set_filter(FilterMode::Linear);

    let test_cases = vec![
        ("pitch_angle", -PI / 2.0, 0.0, 0.0),
        ("yaw_angle", 0.0, -PI / 2.0, 0.0),
        ("roll_angle", 0.0, 0.0, -PI / 2.0),
        ("neg_pitch", PI / 2.0, 0.0, 0.0),
        ("neg_yaw", 0.0, PI / 2.0, 0.0),
        ("neg_roll", 0.0, 0.0, PI / 2.0),
    ];

    for (name, pitch, yaw, roll) in test_cases {
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            zoom: vec2(1.0 / 400.0, -1.0 / 300.0),
            target: vec2(400.0, 300.0),
            ..Default::default()
        });

        clear_background(DARKGRAY);

        draw_text(
            &format!("pitch={:.2}, yaw={:.2}, roll={:.2}", pitch, yaw, roll),
            10.0,
            30.0,
            30.0,
            WHITE,
        );

        draw_line(400.0, 300.0, 400.0, 200.0, 3.0, GREEN);
        draw_text("UP", 410.0, 200.0, 25.0, GREEN);

        draw_line(400.0, 300.0, 500.0, 300.0, 3.0, RED);
        draw_text("RIGHT", 510.0, 305.0, 25.0, RED);

        if let Some(mesh) = model.meshes.first() {
            md3_render::render_md3_mesh_with_yaw_and_roll(
                mesh,
                0,
                400.0,
                300.0,
                3.0,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/ammo/rocket/rocket.png"),
                false,
                pitch,
                yaw,
                roll,
                None,
            );
        }

        set_default_camera();
        next_frame().await;

        render_target
            .texture
            .get_texture_data()
            .export_png(&format!("test_{}.png", name));
        println!("Saved test_{}.png", name);
    }

    println!("\nTesting flight angles:");

    let flight_angles = vec![
        0.0,
        PI / 4.0,
        PI / 2.0,
        3.0 * PI / 4.0,
        PI,
        -PI / 4.0,
        -PI / 2.0,
    ];

    for flight_angle in flight_angles {
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            zoom: vec2(1.0 / 400.0, -1.0 / 300.0),
            target: vec2(400.0, 300.0),
            ..Default::default()
        });

        clear_background(DARKGRAY);

        let vel_x = flight_angle.cos();
        let vel_y = flight_angle.sin();

        draw_text(
            &format!(
                "Flight: {:.0}° (vel_x={:.2}, vel_y={:.2})",
                flight_angle.to_degrees(),
                vel_x,
                vel_y
            ),
            10.0,
            30.0,
            25.0,
            WHITE,
        );

        let arrow_end_x = 400.0 + vel_x * 100.0;
        let arrow_end_y = 300.0 + vel_y * 100.0;
        draw_line(400.0, 300.0, arrow_end_x, arrow_end_y, 4.0, RED);
        draw_circle(arrow_end_x, arrow_end_y, 6.0, RED);

        draw_text(
            "render_md3_mesh_with_pivot_and_yaw_ex: angle",
            10.0,
            70.0,
            20.0,
            YELLOW,
        );
        for mesh in &model.meshes {
            md3_render::render_md3_mesh_with_pivot_and_yaw_ex(
                mesh,
                0,
                400.0,
                300.0,
                2.5,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/ammo/rocket/rocket.png"),
                false,
                flight_angle,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        }

        set_default_camera();
        next_frame().await;

        let filename = format!("flight_{:.0}deg.png", flight_angle.to_degrees());
        render_target
            .texture
            .get_texture_data()
            .export_png(&filename);
        println!("Saved {}", filename);
    }

    println!("\nDone! Check the PNG files.");
}


