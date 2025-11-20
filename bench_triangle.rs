use macroquad::prelude::*;

#[macroquad::main("Triangle Benchmark")]
async fn main() {
    let mut frame_count = 0u64;
    let mut last_fps_update = get_time();
    let mut fps_display = 0;
    let mut triangle_count: i32 = 100;
    
    loop {
        let t_frame_start = get_time();
        
        clear_background(BLACK);
        
        for i in 0..triangle_count {
            let offset = i as f32 * 2.0;
            draw_triangle(
                Vec2::new(100.0 + offset, 100.0),
                Vec2::new(200.0 + offset, 100.0),
                Vec2::new(150.0 + offset, 200.0),
                Color::from_rgba(255, 100, 100, 255),
            );
        }
        
        if is_key_pressed(KeyCode::Up) {
            triangle_count += 100;
            println!("Triangles: {}", triangle_count);
        }
        if is_key_pressed(KeyCode::Down) {
            triangle_count = triangle_count.saturating_sub(100).max(1);
            println!("Triangles: {}", triangle_count);
        }
        
        if get_time() - last_fps_update >= 0.5 {
            fps_display = get_fps();
            last_fps_update = get_time();
        }
        
        let frame_time = (get_time() - t_frame_start) * 1000.0;
        
        draw_text(&format!("FPS: {}", fps_display), 10.0, 30.0, 30.0, WHITE);
        draw_text(&format!("Frame time: {:.2}ms", frame_time), 10.0, 60.0, 30.0, WHITE);
        draw_text(&format!("Triangles: {}", triangle_count), 10.0, 90.0, 30.0, WHITE);
        draw_text(&format!("Frame: {}", frame_count), 10.0, 120.0, 30.0, WHITE);
        draw_text("UP/DOWN: change triangle count", 10.0, 150.0, 20.0, GRAY);
        
        if frame_count % 60 == 0 {
            println!("Frame time: {:.2}ms, FPS: {}, Triangles: {}", frame_time, fps_display, triangle_count);
        }
        
        frame_count += 1;
        next_frame().await;
    }
}

