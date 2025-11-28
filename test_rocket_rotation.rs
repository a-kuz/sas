use macroquad::prelude::*;
use std::f32::consts::PI;

#[macroquad::main("Rocket Rotation Test")]
async fn main() {
    let mut pitch = 0.0f32;
    let mut yaw = 0.0f32;
    let mut roll = 0.0f32;
    
    let mut test_angle = 0.0f32;
    let mut auto_rotate = true;
    
    loop {
        clear_background(DARKGRAY);
        
        if auto_rotate {
            test_angle += 0.01;
            if test_angle > PI * 2.0 {
                test_angle = 0.0;
            }
        }
        
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        
        draw_text(&format!("Test Angle: {:.2} rad ({:.0}°)", test_angle, test_angle.to_degrees()), 10.0, 30.0, 20.0, WHITE);
        draw_text(&format!("Pitch: {:.2}", pitch), 10.0, 60.0, 20.0, WHITE);
        draw_text(&format!("Yaw: {:.2}", yaw), 10.0, 90.0, 20.0, WHITE);
        draw_text(&format!("Roll: {:.2}", roll), 10.0, 120.0, 20.0, WHITE);
        draw_text("Press SPACE to toggle auto-rotation", 10.0, 150.0, 20.0, YELLOW);
        draw_text("Arrow keys to adjust angles manually", 10.0, 180.0, 20.0, YELLOW);
        
        let vel_x = test_angle.cos();
        let vel_y = test_angle.sin();
        
        let arrow_len = 100.0;
        let arrow_x = center_x + vel_x * arrow_len;
        let arrow_y = center_y + vel_y * arrow_len;
        
        draw_line(center_x, center_y, arrow_x, arrow_y, 3.0, RED);
        draw_circle(arrow_x, arrow_y, 8.0, RED);
        
        draw_text("Direction", arrow_x + 10.0, arrow_y, 20.0, RED);
        
        draw_text(&format!("Config 1: pitch=0, yaw={:.2}, roll=0", test_angle - PI/2.0), 10.0, 250.0, 20.0, GREEN);
        draw_text(&format!("Config 2: pitch=0, yaw=PI/2, roll={:.2}", test_angle), 10.0, 280.0, 20.0, BLUE);
        draw_text(&format!("Config 3: pitch={:.2}, yaw=0, roll=0", test_angle), 10.0, 310.0, 20.0, ORANGE);
        draw_text(&format!("Config 4: pitch=0, yaw={:.2}, roll=0", test_angle), 10.0, 340.0, 20.0, PURPLE);
        
        if is_key_pressed(KeyCode::Space) {
            auto_rotate = !auto_rotate;
        }
        
        if is_key_down(KeyCode::Up) {
            pitch += 0.05;
        }
        if is_key_down(KeyCode::Down) {
            pitch -= 0.05;
        }
        if is_key_down(KeyCode::Left) {
            yaw -= 0.05;
        }
        if is_key_down(KeyCode::Right) {
            yaw += 0.05;
        }
        if is_key_down(KeyCode::Q) {
            roll -= 0.05;
        }
        if is_key_down(KeyCode::E) {
            roll += 0.05;
        }
        
        next_frame().await
    }
}

