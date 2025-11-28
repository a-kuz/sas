use macroquad::prelude::*;
use sas::game::md3::MD3Model;
use sas::game::md3_render;
use std::f32::consts::PI;

#[macroquad::main("Render Machinegun Flash")]
async fn main() {
    set_pc_assets_folder(".");
    
    let render_target = render_target(800, 600);
    render_target.texture.set_filter(FilterMode::Nearest);
    
    let model = MD3Model::load("q3-resources/models/weapons2/machinegun/machinegun_flash.md3").unwrap();
    
    let texture_bytes = std::fs::read("q3-resources/models/weapons2/machinegun/f_machinegun.png").unwrap();
    let texture = Texture2D::from_file_with_format(&texture_bytes, None);
    texture.set_filter(FilterMode::Linear);
    
    let test_angles = vec![0.0, PI/4.0, PI/2.0, 3.0*PI/4.0, PI, -PI/4.0, -PI/2.0];
    
    for angle in test_angles {
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            zoom: vec2(1.0 / 400.0, -1.0 / 300.0),
            target: vec2(400.0, 300.0),
            ..Default::default()
        });
        
        clear_background(DARKGRAY);
        
        draw_text(&format!("Angle: {:.0}°", angle.to_degrees()), 
                  10.0, 30.0, 30.0, WHITE);
        
        let offset_x = angle.cos() * 20.0;
        let offset_y = angle.sin() * 20.0;
        
        draw_line(400.0, 300.0, 400.0 + offset_x, 300.0 + offset_y, 3.0, GREEN);
        draw_text("Flash position", 410.0 + offset_x, 295.0 + offset_y, 20.0, GREEN);
        
        let arrow_end_x = 400.0 + angle.cos() * 100.0;
        let arrow_end_y = 300.0 + angle.sin() * 100.0;
        draw_line(400.0, 300.0, arrow_end_x, arrow_end_y, 4.0, RED);
        draw_text("Direction", 410.0, 305.0, 20.0, RED);
        
        for mesh in &model.meshes {
            md3_render::render_md3_mesh_rotated_with_additive(
                mesh,
                0,
                400.0 + offset_x,
                300.0 + offset_y,
                1.2,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/weapons2/machinegun/f_machinegun.png"),
                0.0,
                true,
            );
        }
        
        set_default_camera();
        next_frame().await;
        
        let filename = format!("machinegun_flash_{:.0}deg.png", angle.to_degrees());
        render_target.texture.get_texture_data().export_png(&filename);
        println!("Saved {}", filename);
    }
    
    println!("\nTesting with random roll:");
    
    for i in 0..5 {
        let angle = PI / 4.0;
        let roll = (i as f32 - 2.0) * 5.0;
        
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            zoom: vec2(1.0 / 400.0, -1.0 / 300.0),
            target: vec2(400.0, 300.0),
            ..Default::default()
        });
        
        clear_background(DARKGRAY);
        
        draw_text(&format!("Angle: {:.0}°, Roll: {:.1}°", angle.to_degrees(), roll), 
                  10.0, 30.0, 30.0, WHITE);
        
        let offset_x = angle.cos() * 20.0;
        let offset_y = angle.sin() * 20.0;
        
        for mesh in &model.meshes {
            md3_render::render_md3_mesh_rotated_with_additive(
                mesh,
                0,
                400.0 + offset_x,
                300.0 + offset_y,
                1.2,
                WHITE,
                Some(&texture),
                Some("q3-resources/models/weapons2/machinegun/f_machinegun.png"),
                roll.to_radians(),
                true,
            );
        }
        
        set_default_camera();
        next_frame().await;
        
        let filename = format!("machinegun_flash_roll_{:.0}.png", roll);
        render_target.texture.get_texture_data().export_png(&filename);
        println!("Saved {}", filename);
    }
    
    println!("\nDone! Check the PNG files.");
}

