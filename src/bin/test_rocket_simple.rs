use macroquad::prelude::*;
use sas::game::md3::MD3Model;
use sas::game::md3_render;
use std::f32::consts::PI;

#[macroquad::main("Test Rocket Simple")]
async fn main() {
    set_pc_assets_folder(".");

    let model = match MD3Model::load("q3-resources/models/ammo/rocket/rocket.md3") {
        Ok(m) => {
            println!("Model loaded successfully!");
            println!("Meshes: {}", m.meshes.len());
            m
        }
        Err(e) => {
            eprintln!("Failed to load rocket model: {}", e);
            return;
        }
    };

    let texture_bytes = match std::fs::read("q3-resources/models/ammo/rocket/rocket.png") {
        Ok(b) => {
            println!("Texture loaded successfully! Size: {} bytes", b.len());
            b
        }
        Err(e) => {
            eprintln!("Failed to load texture: {}", e);
            return;
        }
    };
    let texture = Texture2D::from_file_with_format(&texture_bytes, None);
    texture.set_filter(FilterMode::Linear);

    let angles = vec![
        ("right_0deg", 0.0),
        ("up_90deg", PI / 2.0),
        ("left_180deg", PI),
        ("down_neg90deg", -PI / 2.0),
        ("up_right_45deg", PI / 4.0),
    ];

    for (name, angle) in angles {
        clear_background(Color::from_rgba(50, 50, 50, 255));

        let vel_x = angle.cos();
        let vel_y = angle.sin();

        draw_text(
            &format!("Angle: {:.2} rad ({:.0}°)", angle, angle.to_degrees()),
            10.0,
            30.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("vel_x={:.2}, vel_y={:.2}", vel_x, vel_y),
            10.0,
            60.0,
            25.0,
            YELLOW,
        );

        let center_x = 400.0;
        let center_y = 300.0;
        let arrow_len = 100.0;
        let arrow_x = center_x + vel_x * arrow_len;
        let arrow_y = center_y + vel_y * arrow_len;
        draw_line(center_x, center_y, arrow_x, arrow_y, 4.0, RED);
        draw_circle(arrow_x, arrow_y, 8.0, RED);

        if let Some(mesh) = model.meshes.first() {
            draw_text("Using render_md3_mesh_rotated:", 10.0, 100.0, 20.0, GREEN);
            draw_text(
                &format!("rotation = angle + PI/2 = {:.2}", angle + PI / 2.0),
                10.0,
                130.0,
                20.0,
                GREEN,
            );

            md3_render::render_md3_mesh_rotated(
                mesh,
                0,
                center_x,
                center_y,
                3.0,
                WHITE,
                Some(&texture),
                angle + PI / 2.0,
            );
        }

        draw_text(
            &format!("Press SPACE to save: {}", name),
            10.0,
            550.0,
            25.0,
            YELLOW,
        );

        next_frame().await;

        loop {
            if is_key_pressed(KeyCode::Space) {
                let image = get_screen_data();
                let filename = format!("rocket_simple_{}.png", name);
                image.export_png(&filename);
                println!("Saved: {}", filename);
                break;
            }
            next_frame().await;
        }
    }

    println!("\nAll test images saved!");
}
