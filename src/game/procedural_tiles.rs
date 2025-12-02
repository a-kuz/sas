use macroquad::prelude::*;

#[allow(dead_code)]
pub fn render_procedural_tile_world(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    texture_id: u16,
    neighbors: (bool, bool, bool, bool),
    world_x: f32,
    world_y: f32,
) {
    render_procedural_tile_world_ex(
        x, y, width, height, texture_id, neighbors, world_x, world_y, None,
    );
}

pub fn render_procedural_tile_world_ex(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    texture_id: u16,
    neighbors: (bool, bool, bool, bool),
    world_x: f32,
    world_y: f32,
    neighbor_textures: Option<(Option<u16>, Option<u16>, Option<u16>, Option<u16>)>,
) {
    let (left_same, right_same, top_same, bottom_same) = neighbors;
    let (left_tex, right_tex, top_tex, bottom_tex) =
        neighbor_textures.unwrap_or((None, None, None, None));

    let base_color = get_base_color(texture_id);
    let edge_color = get_edge_color(texture_id);

    draw_rectangle(x, y, width, height, base_color);

    add_texture_detail(x, y, width, height, texture_id, world_x, world_y);

    let has_left_transition = left_tex.map_or(false, |t| t != texture_id);
    let has_right_transition = right_tex.map_or(false, |t| t != texture_id);
    let has_top_transition = top_tex.map_or(false, |t| t != texture_id);
    let has_bottom_transition = bottom_tex.map_or(false, |t| t != texture_id);

    if has_left_transition && left_same {
        render_smooth_transition_vertical(
            x,
            y,
            5.0,
            height,
            texture_id,
            left_tex.unwrap(),
            world_y,
        );
    }
    if has_right_transition && right_same {
        render_smooth_transition_vertical(
            x + width - 5.0,
            y,
            5.0,
            height,
            texture_id,
            right_tex.unwrap(),
            world_y,
        );
    }
    if has_top_transition && top_same {
        render_smooth_transition_horizontal(
            x,
            y,
            width,
            5.0,
            texture_id,
            top_tex.unwrap(),
            world_x,
        );
    }
    if has_bottom_transition && bottom_same {
        render_smooth_transition_horizontal(
            x,
            y + height - 5.0,
            width,
            5.0,
            texture_id,
            bottom_tex.unwrap(),
            world_x,
        );
    }

    let corner_radius = 6.0;

    if !bottom_same && !has_bottom_transition {
        draw_rectangle(
            x,
            y + height - 2.0,
            width,
            2.0,
            darken_color(base_color, 0.6),
        );
    }

    if !left_same && !has_left_transition {
        let cap_top = if !top_same { corner_radius } else { 0.0 };
        let cap_bottom = if !bottom_same { corner_radius } else { 0.0 };
        let side_height = height - cap_top - cap_bottom;
        let start_y = y + cap_top;

        if side_height > 0.0 {
            draw_rectangle(x, start_y, 6.0, side_height, darken_color(base_color, 0.82));
            draw_rectangle(x, start_y, 2.0, side_height, darken_color(base_color, 0.7));
            draw_rectangle(
                x + 4.0,
                start_y,
                2.0,
                side_height,
                darken_color(base_color, 0.68),
            );
        }
    }

    if !right_same && !has_right_transition {
        let cap_top = if !top_same { corner_radius } else { 0.0 };
        let cap_bottom = if !bottom_same { corner_radius } else { 0.0 };
        let side_height = height - cap_top - cap_bottom;
        let start_y = y + cap_top;

        if side_height > 0.0 {
            draw_rectangle(
                x + width - 6.0,
                start_y,
                6.0,
                side_height,
                darken_color(base_color, 0.8),
            );
            draw_rectangle(
                x + width - 6.0,
                start_y,
                2.0,
                side_height,
                darken_color(base_color, 0.64),
            );
            draw_rectangle(
                x + width - 2.0,
                start_y,
                2.0,
                side_height,
                darken_color(base_color, 0.62),
            );
        }
    }

    if !top_same && !has_top_transition {
        draw_rectangle(x, y, width, 6.0, edge_color);
        draw_rectangle(x, y, width, 1.5, lighten_color(edge_color, 1.5));

        if is_grass(texture_id) {
            let grass_start_x = if !left_same { width * 0.25 } else { 0.0 };
            let grass_end_x = if !right_same { width * 0.75 } else { width };
            let grass_width = grass_end_x - grass_start_x;

            if grass_width > 5.0 {
                render_grass_top(
                    x + grass_start_x,
                    y,
                    grass_width,
                    left_same,
                    right_same,
                    world_x + grass_start_x,
                    world_y,
                );
            }
        } else {
            add_top_decoration(x, y, width, texture_id, world_x);
        }
    }

    if !top_same && !left_same && !has_top_transition && !has_left_transition {
        draw_rounded_corner(x, y, corner_radius, true, true, edge_color);
    }
    if !top_same && !right_same && !has_top_transition && !has_right_transition {
        draw_rounded_corner(x + width, y, corner_radius, false, true, edge_color);
    }
    if !bottom_same && !left_same && !has_bottom_transition && !has_left_transition {
        draw_rounded_corner(x, y + height, corner_radius, true, false, edge_color);
    }
    if !bottom_same && !right_same && !has_bottom_transition && !has_right_transition {
        draw_rounded_corner(
            x + width,
            y + height,
            corner_radius,
            false,
            false,
            edge_color,
        );
    }
}

fn draw_rounded_corner(
    x: f32,
    y: f32,
    radius: f32,
    is_left: bool,
    is_top: bool,
    edge_color: Color,
) {
    let bg = Color::from_rgba(14, 16, 20, 255);

    let (cx, cy, x_dir, y_dir) = if is_left && is_top {
        (x + radius, y + radius, -1.0, -1.0)
    } else if !is_left && is_top {
        (x - radius, y + radius, 1.0, -1.0)
    } else if is_left && !is_top {
        (x + radius, y - radius, -1.0, 1.0)
    } else {
        (x - radius, y - radius, 1.0, 1.0)
    };

    let start_angle = if is_left && is_top {
        std::f32::consts::PI
    } else if !is_left && is_top {
        1.5 * std::f32::consts::PI
    } else if is_left && !is_top {
        0.5 * std::f32::consts::PI
    } else {
        0.0
    };

    let pixel_radius = radius.ceil() as i32;
    for px in 0..=pixel_radius {
        for py in 0..=pixel_radius {
            let dx = px as f32;
            let dy = py as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius {
                let render_x = x + x_dir * dx;
                let render_y = y + y_dir * dy;
                draw_rectangle(render_x, render_y, 1.0, 1.0, bg);
            }
        }
    }

    let segments = 20;
    for i in 0..segments {
        let angle1 = start_angle + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
        let angle2 = start_angle + ((i + 1) as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
        let px1 = cx + angle1.cos() * radius;
        let py1 = cy + angle1.sin() * radius;
        let px2 = cx + angle2.cos() * radius;
        let py2 = cy + angle2.sin() * radius;
        draw_line(px1, py1, px2, py2, 1.2, lighten_color(edge_color, 1.04));
    }

    let inner_r1 = radius - 1.0;
    let inner_r2 = radius - 2.0;
    let inner_r3 = radius - 3.0;

    if inner_r1 > 0.0 {
        for i in 0..segments {
            let angle1 = start_angle + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let angle2 =
                start_angle + ((i + 1) as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let px1 = cx + angle1.cos() * inner_r1;
            let py1 = cy + angle1.sin() * inner_r1;
            let px2 = cx + angle2.cos() * inner_r1;
            let py2 = cy + angle2.sin() * inner_r1;
            draw_line(px1, py1, px2, py2, 0.8, Color::from_rgba(0, 0, 0, 30));
        }
    }

    if inner_r2 > 0.0 {
        for i in 0..segments {
            let angle1 = start_angle + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let angle2 =
                start_angle + ((i + 1) as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let px1 = cx + angle1.cos() * inner_r2;
            let py1 = cy + angle1.sin() * inner_r2;
            let px2 = cx + angle2.cos() * inner_r2;
            let py2 = cy + angle2.sin() * inner_r2;
            draw_line(px1, py1, px2, py2, 0.6, Color::from_rgba(0, 0, 0, 18));
        }
    }

    if inner_r3 > 0.0 {
        for i in 0..segments {
            let angle1 = start_angle + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let angle2 =
                start_angle + ((i + 1) as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let px1 = cx + angle1.cos() * inner_r3;
            let py1 = cy + angle1.sin() * inner_r3;
            let px2 = cx + angle2.cos() * inner_r3;
            let py2 = cy + angle2.sin() * inner_r3;
            draw_line(px1, py1, px2, py2, 0.5, Color::from_rgba(0, 0, 0, 10));
        }
    }
}

fn render_smooth_transition_horizontal(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    from_tex: u16,
    to_tex: u16,
    world_x: f32,
) {
    let color_from = get_base_color(from_tex);
    let color_to = get_base_color(to_tex);

    let hash = |v: f32| -> f32 { ((v * 12.9898).sin() * 43758.5453).fract() };

    for py_offset in 0..(height as i32) {
        let py = y + py_offset as f32;
        let progress = py_offset as f32 / height;

        for px_offset in 0..(width as i32) {
            let px = x + px_offset as f32;
            let world_px = world_x + px_offset as f32;

            let seed = hash(world_px * 0.3 + progress * 10.0);
            let noise = (hash(seed + 0.5) - 0.5) * 0.25;
            let t = (progress + noise).clamp(0.0, 1.0);

            let blended = blend_colors(color_from, color_to, t);
            draw_rectangle(px, py, 1.0, 1.0, blended);

            if hash(seed + 0.7) > 0.85 {
                let spot_alpha = (30.0 * (1.0 - (t - 0.5).abs() * 2.0)) as u8;
                draw_rectangle(px, py, 1.0, 1.0, Color::from_rgba(0, 0, 0, spot_alpha));
            }
        }
    }
}

fn render_smooth_transition_vertical(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    from_tex: u16,
    to_tex: u16,
    world_y: f32,
) {
    let color_from = get_base_color(from_tex);
    let color_to = get_base_color(to_tex);

    let hash = |v: f32| -> f32 { ((v * 12.9898).sin() * 43758.5453).fract() };

    for px_offset in 0..(width as i32) {
        let px = x + px_offset as f32;
        let progress = px_offset as f32 / width;

        for py_offset in 0..(height as i32) {
            let py = y + py_offset as f32;
            let world_py = world_y + py_offset as f32;

            let seed = hash(world_py * 0.3 + progress * 10.0);
            let noise = (hash(seed + 0.5) - 0.5) * 0.25;
            let t = (progress + noise).clamp(0.0, 1.0);

            let blended = blend_colors(color_from, color_to, t);
            draw_rectangle(px, py, 1.0, 1.0, blended);

            if hash(seed + 0.7) > 0.85 {
                let spot_alpha = (30.0 * (1.0 - (t - 0.5).abs() * 2.0)) as u8;
                draw_rectangle(px, py, 1.0, 1.0, Color::from_rgba(0, 0, 0, spot_alpha));
            }
        }
    }
}

pub fn get_base_color_pub(texture_id: u16) -> Color {
    get_base_color(texture_id)
}

pub fn render_procedural_tile_simple(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    texture_id: u16,
    neighbors: (bool, bool, bool, bool),
    _world_x: f32,
    _world_y: f32,
) {
    let (left_same, right_same, top_same, bottom_same) = neighbors;

    let base_color = get_base_color(texture_id);
    let edge_color = get_edge_color(texture_id);

    draw_rectangle(x, y, width, height, base_color);

    if !bottom_same {
        draw_rectangle(
            x,
            y + height - 2.0,
            width,
            2.0,
            darken_color(base_color, 0.6),
        );
    }

    if !left_same {
        draw_rectangle(x, y, 2.0, height, darken_color(base_color, 0.75));
    }

    if !right_same {
        draw_rectangle(
            x + width - 2.0,
            y,
            2.0,
            height,
            darken_color(base_color, 0.70),
        );
    }

    if !top_same {
        draw_rectangle(x, y, width, 3.0, edge_color);
        draw_rectangle(x, y, width, 1.0, lighten_color(edge_color, 1.3));
    }
}

fn get_base_color(texture_id: u16) -> Color {
    match texture_id {
        0 => Color::from_rgba(88, 62, 42, 255),
        1 => Color::from_rgba(75, 75, 80, 255),
        2 => Color::from_rgba(95, 82, 62, 255),
        3 => Color::from_rgba(85, 90, 75, 255),
        4 => Color::from_rgba(110, 95, 70, 255),
        5 => Color::from_rgba(68, 72, 78, 255),
        6 => Color::from_rgba(130, 100, 75, 255),
        7 => Color::from_rgba(60, 65, 72, 255),
        8 => Color::from_rgba(78, 82, 88, 255),
        9 => Color::from_rgba(92, 78, 60, 255),
        _ => Color::from_rgba(55, 57, 62, 255),
    }
}

fn get_edge_color(texture_id: u16) -> Color {
    match texture_id {
        0 => Color::from_rgba(70, 48, 32, 255),
        1 => Color::from_rgba(85, 85, 90, 255),
        2 => Color::from_rgba(105, 92, 72, 255),
        3 => Color::from_rgba(95, 100, 85, 255),
        4 => Color::from_rgba(120, 105, 80, 255),
        5 => Color::from_rgba(78, 82, 88, 255),
        6 => Color::from_rgba(140, 110, 85, 255),
        7 => Color::from_rgba(70, 75, 82, 255),
        8 => Color::from_rgba(88, 92, 98, 255),
        9 => Color::from_rgba(102, 88, 70, 255),
        _ => Color::from_rgba(65, 67, 72, 255),
    }
}

fn darken_color(color: Color, factor: f32) -> Color {
    Color::from_rgba(
        (color.r * 255.0 * factor) as u8,
        (color.g * 255.0 * factor) as u8,
        (color.b * 255.0 * factor) as u8,
        color.a as u8,
    )
}

fn lighten_color(color: Color, factor: f32) -> Color {
    Color::from_rgba(
        ((color.r * 255.0 * factor).min(255.0)) as u8,
        ((color.g * 255.0 * factor).min(255.0)) as u8,
        ((color.b * 255.0 * factor).min(255.0)) as u8,
        color.a as u8,
    )
}

fn blend_colors(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        ((a.r * 255.0) * (1.0 - t) + (b.r * 255.0) * t) as u8,
        ((a.g * 255.0) * (1.0 - t) + (b.g * 255.0) * t) as u8,
        ((a.b * 255.0) * (1.0 - t) + (b.b * 255.0) * t) as u8,
        255,
    )
}

fn add_top_decoration(x: f32, y: f32, width: f32, texture_id: u16, world_x: f32) {
    let hash = |v: f32| -> f32 { ((v * 17.231).sin() * 43758.5453).fract() };

    match texture_id {
        2 | 3 | 4 | 6 | 9 => {
            let num_details = (width / 4.0) as i32;
            for i in 0..num_details {
                let world_pos = world_x + i as f32 * 4.0;
                let seed = hash(world_pos);
                if seed < 0.6 {
                    let px = x + i as f32 * 4.0 + hash(world_pos + 0.5) * 2.0;
                    let height = 2.0 + hash(world_pos + 1.0) * 2.0;
                    draw_rectangle(px, y, 1.0, height, Color::from_rgba(0, 0, 0, 40));
                    draw_rectangle(
                        px,
                        y,
                        1.0,
                        1.0,
                        lighten_color(get_edge_color(texture_id), 1.1),
                    );
                }
            }
        }
        _ => {}
    }
}

fn add_texture_detail(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    texture_id: u16,
    world_x: f32,
    world_y: f32,
) {
    let hash = |v: f32| -> f32 { ((v * 12.9898).sin() * 43758.5453).fract() };
    let seed_base = hash(world_x * 0.1 + world_y * 0.1);

    match texture_id {
        0 => {
            for i in 0..4 {
                let t = i as f32 / 4.0;
                let yy = y + 6.0 + t * (height - 12.0);
                let wave_offset = hash(world_y + yy) * 2.0;
                draw_rectangle(
                    x + wave_offset,
                    yy,
                    width - wave_offset * 2.0,
                    1.0,
                    Color::from_rgba(60, 42, 28, 100),
                );
            }
        }
        1 => {
            let num_blocks = (width / 12.0).max(1.0) as i32;
            for i in 0..num_blocks {
                let bx = x + 6.0 + i as f32 * 12.0;
                let by = y + 6.0 + hash(world_x + i as f32) * 3.0;
                draw_rectangle(bx, by, 4.0, 5.0, Color::from_rgba(65, 65, 70, 200));
                draw_rectangle(bx, by, 1.0, 5.0, Color::from_rgba(80, 80, 85, 200));
            }
        }
        2 | 4 | 9 => {
            let num_cracks = (width / 8.0) as i32;
            for i in 0..num_cracks {
                let cx = x + 4.0 + i as f32 * 8.0;
                let cy = y + 6.0 + hash(world_x + i as f32) * 4.0;
                let crack_len = 3.0 + hash(world_y + i as f32) * 3.0;
                draw_line(
                    cx,
                    cy,
                    cx,
                    cy + crack_len,
                    1.0,
                    Color::from_rgba(0, 0, 0, 30),
                );
            }
        }
        3 | 5 | 8 => {
            let num_panels = ((width * height) / 200.0).max(1.0) as i32;
            for i in 0..num_panels {
                let px = x + 6.0 + hash(seed_base + i as f32) * (width - 12.0);
                let py = y + 4.0 + hash(seed_base + i as f32 + 0.5) * (height - 8.0);
                draw_rectangle(px, py, 5.0, 3.0, Color::from_rgba(0, 0, 0, 15));
                draw_rectangle(px, py, 5.0, 1.0, Color::from_rgba(255, 255, 255, 6));
            }
        }
        6 => {
            let num_rivets = (width / 12.0) as i32;
            for i in 0..num_rivets {
                let rx = x + 6.0 + i as f32 * 12.0;
                let ry = y + 7.0 + hash(world_x + i as f32) * 3.0;
                draw_circle(rx, ry, 1.5, Color::from_rgba(0, 0, 0, 25));
                draw_circle(rx - 0.2, ry - 0.2, 1.0, Color::from_rgba(150, 120, 90, 200));
            }
        }
        _ => {}
    }
}

fn is_grass(texture_id: u16) -> bool {
    texture_id == 0 || texture_id == 7 || texture_id == 9
}

fn render_grass_top(
    x: f32,
    y: f32,
    width: f32,
    _left_neighbor: bool,
    _right_neighbor: bool,
    world_x: f32,
    _world_y: f32,
) {
    let hash = |n: f32| -> f32 { (n.sin() * 43758.5453).fract() };
    let time = crate::time::get_time() as f32;

    let num_blades = (width / 5.0) as i32;
    for i in 0..num_blades {
        let blade_world_x = world_x + i as f32 * 5.0;
        let blade_id = (blade_world_x / 5.0) as i32;
        let seed = hash(blade_id as f32);

        if seed < 0.4 {
            continue;
        }

        let blade_local_x = i as f32 * 5.0 + hash(blade_id as f32 * 1.1) * 3.0;
        let bx = x + blade_local_x;
        let h = 3.0 + hash(blade_id as f32 * 1.3) * 2.5;
        let wind = (time * 1.5 + blade_id as f32 * 0.5).sin() * 1.5;

        let blade_color1 = Color::from_rgba(70, 120, 50, 220);
        let blade_color2 = Color::from_rgba(100, 160, 70, 180);

        draw_line(bx, y - 1.0, bx + wind, y - h, 1.3, blade_color1);
        draw_line(
            bx + 0.4,
            y - 0.4,
            bx + 0.4 + wind * 0.7,
            y - h * 0.7,
            0.8,
            blade_color2,
        );
    }
}
