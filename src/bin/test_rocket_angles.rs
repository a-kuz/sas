use macroquad::prelude::*;
use sas::game::md3::MD3Model;
use sas::game::md3_render;
use std::f32::consts::PI;

#[macroquad::main("Test Rocket Angles")]
async fn main() {
    set_pc_assets_folder(".");
    
    let model = match MD3Model::load("q3-resources/models/ammo/rocket/rocket.md3") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load rocket model: {}", e);
            return;
        }
    };

    let texture_bytes = match std::fs::read("q3-resources/models/ammo/rocket/rocket.png") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to load texture: {}", e);
            return;
        }
    };
    let texture = Texture2D::from_file_with_format(&texture_bytes, None);
    texture.set_filter(FilterMode::Linear);

    let test_configs = vec![
        ("pitch_only", PI / 2.0, 0.0, 0.0),
        ("yaw_only", 0.0, PI / 2.0, 0.0),
        ("roll_only", 0.0, 0.0, PI / 2.0),
        ("pitch_neg", -PI / 2.0, 0.0, 0.0),
        ("yaw_neg", 0.0, -PI / 2.0, 0.0),
        ("roll_neg", 0.0, 0.0, -PI / 2.0),
        ("yaw_pi2_roll_pi2", 0.0, PI / 2.0, PI / 2.0),
        ("yaw_neg_pi2_roll_pi2", 0.0, -PI / 2.0, PI / 2.0),
        ("pitch_pi2_yaw_pi2", PI / 2.0, PI / 2.0, 0.0),
    ];

    for (name, pitch, yaw, roll) in test_configs {
        clear_background(Color::from_rgba(50, 50, 50, 255));
        
        draw_text(&format!("pitch={:.2}, yaw={:.2}, roll={:.2}", pitch, yaw, roll), 
                  10.0, 30.0, 30.0, WHITE);
        
        draw_line(400.0, 300.0, 500.0, 300.0, 2.0, RED);
        draw_text("→ Right (0°)", 510.0, 305.0, 20.0, RED);
        
        draw_line(400.0, 300.0, 400.0, 200.0, 2.0, GREEN);
        draw_text("↑ Up (-90°)", 410.0, 195.0, 20.0, GREEN);
        
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
        
        next_frame().await;
        
        let image = get_screen_data();
        image.export_png(&format!("rocket_test_{}.png", name));
        println!("Saved: rocket_test_{}.png", name);
    }

    println!("\nNow testing angle rotations for flight direction...");
    
    let angles = vec![0.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0, PI, -PI / 4.0, -PI / 2.0];
    
    for angle in angles {
        clear_background(Color::from_rgba(50, 50, 50, 255));
        
        let vel_x = angle.cos();
        let vel_y = angle.sin();
        
        draw_text(&format!("Flight angle: {:.2} rad ({:.0}°)", angle, angle.to_degrees()), 
                  10.0, 30.0, 30.0, WHITE);
        draw_text(&format!("vel_x={:.2}, vel_y={:.2}", vel_x, vel_y), 
                  10.0, 60.0, 25.0, YELLOW);
        
        let arrow_len = 80.0;
        let arrow_x = 400.0 + vel_x * arrow_len;
        let arrow_y = 300.0 + vel_y * arrow_len;
        draw_line(400.0, 300.0, arrow_x, arrow_y, 3.0, RED);
        draw_circle(arrow_x, arrow_y, 5.0, RED);
        
        if let Some(mesh) = model.meshes.first() {
            draw_text("Config A: yaw=-PI/2, roll=angle", 10.0, 100.0, 20.0, GREEN);
            md3_render::render_md3_mesh_with_yaw_and_roll(
                mesh,
                0,
                200.0,
                300.0,
                2.0,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/ammo/rocket/rocket.png"),
                false,
                0.0,
                -PI / 2.0,
                angle,
                None,
            );
            
            draw_text("Config B: yaw=PI/2, roll=angle", 10.0, 130.0, 20.0, BLUE);
            md3_render::render_md3_mesh_with_yaw_and_roll(
                mesh,
                0,
                400.0,
                300.0,
                2.0,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/ammo/rocket/rocket.png"),
                false,
                0.0,
                PI / 2.0,
                angle,
                None,
            );
            
            draw_text("Config C: pitch=angle, yaw=0", 10.0, 160.0, 20.0, ORANGE);
            md3_render::render_md3_mesh_with_yaw_and_roll(
                mesh,
                0,
                600.0,
                300.0,
                2.0,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/ammo/rocket/rocket.png"),
                false,
                angle,
                0.0,
                0.0,
                None,
            );
        }
        
        next_frame().await;
        
        let image = get_screen_data();
        let filename = format!("rocket_flight_angle_{:.0}deg.png", angle.to_degrees());
        image.export_png(&filename);
        println!("Saved: {}", filename);
    }
    
    println!("\nAll test images saved! Check the project directory.");
}

