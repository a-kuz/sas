use macroquad::prelude::*;

pub fn render_compact_status_bar(
    map_name: &str,
    tool_name: &str,
    show_grid: bool,
    zoom: f32,
) {
    let status = format!("{} | {} | Grid:{} | Zoom:{:.1}x | H:Help", 
        map_name, 
        tool_name, 
        if show_grid { "ON" } else { "OFF" }, 
        zoom
    );
    draw_text(&status, 10.0, 22.0, 20.0, WHITE);
}
