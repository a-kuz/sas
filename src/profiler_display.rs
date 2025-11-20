use macroquad::prelude::*;

pub fn draw_profiler() {
    if !crate::profiler::is_enabled() {
        return;
    }

    let samples = crate::profiler::get_samples();
    if samples.is_empty() {
        return;
    }

    let panel_width = 480.0;
    let panel_height = 480.0;
    let panel_x = screen_width() - panel_width - 10.0;
    let panel_y = 50.0;

    draw_rectangle(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::from_rgba(0, 0, 0, 200),
    );
    
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        2.0,
        Color::from_rgba(0, 200, 255, 255),
    );

    let title_color = Color::from_rgba(0, 255, 255, 255);
    draw_text(
        "PROFILER [F8]",
        panel_x + 10.0,
        panel_y + 20.0,
        18.0,
        title_color,
    );

    let mut y = panel_y + 45.0;
    let line_height = 16.0;
    let _bar_x = panel_x + 150.0;
    let _bar_max_width = panel_width - 260.0;

    draw_text("Name", panel_x + 10.0, y, 12.0, WHITE);
    draw_text("Current", panel_x + 140.0, y, 12.0, WHITE);
    draw_text("Avg", panel_x + 210.0, y, 12.0, WHITE);
    draw_text("Min", panel_x + 260.0, y, 12.0, WHITE);
    draw_text("Max", panel_x + 310.0, y, 12.0, WHITE);
    draw_text("Graph", panel_x + 360.0, y, 12.0, WHITE);
    y += line_height + 5.0;

    draw_line(
        panel_x + 5.0,
        y,
        panel_x + panel_width - 5.0,
        y,
        1.0,
        Color::from_rgba(100, 100, 100, 255),
    );
    y += 8.0;

    let max_time = samples.iter()
        .map(|(_, current, _, _, max)| current.max(*max))
        .fold(0.0, f64::max)
        .max(1.0);

    for (i, (name, current, avg, min, max)) in samples.iter().take(25).enumerate() {
        if i % 2 == 0 {
            draw_rectangle(
                panel_x + 5.0,
                y - 12.0,
                panel_width - 10.0,
                line_height,
                Color::from_rgba(30, 30, 40, 100),
            );
        }

        let color = if *current > 5.0 {
            Color::from_rgba(255, 80, 80, 255)
        } else if *current > 2.0 {
            Color::from_rgba(255, 200, 80, 255)
        } else {
            Color::from_rgba(150, 255, 150, 255)
        };

        draw_text(name, panel_x + 10.0, y, 11.0, WHITE);

        draw_text(
            &format!("{:.2}", current),
            panel_x + 140.0,
            y,
            11.0,
            color,
        );

        draw_text(
            &format!("{:.2}", avg),
            panel_x + 210.0,
            y,
            11.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        draw_text(
            &format!("{:.2}", min),
            panel_x + 260.0,
            y,
            11.0,
            Color::from_rgba(150, 150, 150, 255),
        );

        draw_text(
            &format!("{:.2}", max),
            panel_x + 310.0,
            y,
            11.0,
            Color::from_rgba(255, 150, 150, 255),
        );

        let history = crate::profiler::get_history(name);
        if !history.is_empty() {
            let graph_x = panel_x + 360.0;
            let graph_y = y - 10.0;
            let graph_width = 70.0;
            let graph_height = 12.0;

            draw_rectangle_lines(
                graph_x,
                graph_y,
                graph_width,
                graph_height,
                0.5,
                Color::from_rgba(100, 100, 100, 150),
            );

            let points_to_show = history.len().min(35);
            let start_idx = if history.len() > points_to_show {
                history.len() - points_to_show
            } else {
                0
            };

            for (idx, &value) in history[start_idx..].iter().enumerate() {
                let ratio = (value / max_time).min(1.0) as f32;
                let bar_height = ratio * graph_height;
                let bar_x_pos = graph_x + (idx as f32 / points_to_show as f32) * graph_width;
                let bar_width = (graph_width / points_to_show as f32).max(1.0);

                let bar_color = if value > 5.0 {
                    Color::from_rgba(255, 80, 80, 200)
                } else if value > 2.0 {
                    Color::from_rgba(255, 200, 80, 200)
                } else {
                    Color::from_rgba(80, 200, 80, 200)
                };

                draw_rectangle(
                    bar_x_pos,
                    graph_y + graph_height - bar_height,
                    bar_width,
                    bar_height,
                    bar_color,
                );
            }
        }

        y += line_height;
    }

    let bottom_y = panel_y + panel_height - 25.0;
    draw_line(
        panel_x + 5.0,
        bottom_y,
        panel_x + panel_width - 5.0,
        bottom_y,
        1.0,
        Color::from_rgba(100, 100, 100, 255),
    );

    let total_time = samples.iter().find(|(name, _, _, _, _)| *name == "render_total")
        .map(|(_, current, _, _, _)| current)
        .unwrap_or(&0.0);

    let shader_stats = crate::profiler::get_shader_stats();
    let total_draws: usize = shader_stats.iter().map(|(_, count)| count).sum();
    
    // Debug: всегда показываем секцию шейдеров для отладки
    let debug_show_shaders = true;
    
    let help_color = Color::from_rgba(150, 150, 150, 255);
    draw_text(
        &format!("Total: {:.2}ms | Target: <8.33ms (120 FPS)", total_time),
        panel_x + 10.0,
        bottom_y + 15.0,
        11.0,
        help_color,
    );
    
    draw_text(
        &format!("Draw calls: {}", total_draws),
        panel_x + 10.0,
        bottom_y + 30.0,
        11.0,
        help_color,
    );
    
    if debug_show_shaders || !shader_stats.is_empty() {
        let mut shader_y = bottom_y + 50.0;
        draw_text("Shader Stats:", panel_x + 10.0, shader_y, 12.0, WHITE);
        shader_y += 15.0;
        
        if shader_stats.is_empty() {
            draw_text(
                "No shader stats (draw calls = 0)",
                panel_x + 15.0,
                shader_y,
                10.0,
                Color::from_rgba(255, 200, 80, 255),
            );
        } else {
            for (shader_name, count) in shader_stats.iter().take(8) {
                let shader_color = if *count > 1000 {
                    Color::from_rgba(255, 80, 80, 255)
                } else if *count > 500 {
                    Color::from_rgba(255, 200, 80, 255)
                } else {
                    Color::from_rgba(150, 255, 150, 255)
                };
                
                draw_text(
                    &format!("{}: {}", shader_name, count),
                    panel_x + 15.0,
                    shader_y,
                    10.0,
                    shader_color,
                );
                shader_y += 12.0;
            }
        }
    }
}

