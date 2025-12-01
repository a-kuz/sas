use macroquad::prelude::*;

pub fn draw_player_sprite(
    screen_x: f32,
    screen_y: f32,
    direction: u8,
    color: Color,
    animation_frame: u8,
    is_attacking: bool,
    is_walking: bool,
    is_crouching: bool,
) {
    let flip = direction == 1 || direction == 2;
    let x_mult = if flip { -1.0 } else { 1.0 };

    let shadow_color = Color::from_rgba(
        (color.r * 255.0 * 0.4) as u8,
        (color.g * 255.0 * 0.4) as u8,
        (color.b * 255.0 * 0.4) as u8,
        255,
    );

    let outline_color = Color::from_rgba(0, 0, 0, 255);

    draw_ellipse(
        screen_x,
        screen_y + 18.0,
        14.0,
        5.0,
        0.0,
        Color::from_rgba(0, 0, 0, 80),
    );

    let crouch_y = if is_crouching { 6.0 } else { 0.0 };
    let walk_bob = if is_walking {
        (animation_frame as f32 / 4.0).sin() * 1.5
    } else {
        0.0
    };

    let leg_spread = if is_walking {
        let cycle = animation_frame % 8;
        match cycle {
            0 => (3.0, -3.0),
            1 => (2.5, -2.0),
            2 => (1.0, -0.5),
            3 => (0.0, 0.0),
            4 => (-3.0, 3.0),
            5 => (-2.5, 2.0),
            6 => (-1.0, 0.5),
            _ => (0.0, 0.0),
        }
    } else {
        (0.0, 0.0)
    };

    let left_leg_x = screen_x - 4.0 * x_mult + leg_spread.0 * x_mult;
    let right_leg_x = screen_x + 3.0 * x_mult + leg_spread.1 * x_mult;
    let leg_y = screen_y + 10.0 + crouch_y;
    let leg_height = if is_crouching { 4.0 } else { 7.0 };

    draw_rectangle(
        left_leg_x - 0.5,
        leg_y - 0.5,
        4.0,
        leg_height + 1.0,
        outline_color,
    );
    draw_rectangle(
        right_leg_x - 0.5,
        leg_y - 0.5,
        4.0,
        leg_height + 1.0,
        outline_color,
    );

    draw_rectangle(left_leg_x, leg_y, 3.0, leg_height, shadow_color);
    draw_rectangle(right_leg_x, leg_y, 3.0, leg_height, shadow_color);

    draw_rectangle(left_leg_x + 0.5, leg_y, 2.0, leg_height - 1.0, color);
    draw_rectangle(right_leg_x + 0.5, leg_y, 2.0, leg_height - 1.0, color);

    let torso_y = screen_y - 5.0 + walk_bob + crouch_y;
    let torso_width = 16.0;
    let torso_height = if is_crouching { 11.0 } else { 15.0 };

    draw_rectangle(
        screen_x - torso_width / 2.0 - 1.0,
        torso_y - 1.0,
        torso_width + 2.0,
        torso_height + 2.0,
        outline_color,
    );

    draw_rectangle(
        screen_x - torso_width / 2.0,
        torso_y,
        torso_width,
        torso_height,
        color,
    );

    draw_rectangle(
        screen_x - torso_width / 2.0,
        torso_y,
        torso_width,
        2.0,
        Color::from_rgba(
            (color.r * 255.0 * 1.2).min(255.0) as u8,
            (color.g * 255.0 * 1.2).min(255.0) as u8,
            (color.b * 255.0 * 1.2).min(255.0) as u8,
            255,
        ),
    );

    let arm_x_offset = if is_attacking { 4.0 } else { 0.0 };

    draw_rectangle(
        screen_x - 10.0 * x_mult - arm_x_offset * x_mult,
        torso_y + 3.0,
        3.0,
        9.0,
        shadow_color,
    );
    draw_rectangle(
        screen_x + 7.0 * x_mult + arm_x_offset * x_mult,
        torso_y + 3.0,
        3.0,
        9.0,
        shadow_color,
    );

    draw_rectangle(
        screen_x - 9.5 * x_mult - arm_x_offset * x_mult,
        torso_y + 3.5,
        2.0,
        8.0,
        color,
    );
    draw_rectangle(
        screen_x + 7.5 * x_mult + arm_x_offset * x_mult,
        torso_y + 3.5,
        2.0,
        8.0,
        color,
    );

    let head_y = screen_y - 13.0 + walk_bob + crouch_y;
    let head_size = 13.0;

    draw_rectangle(
        screen_x - head_size / 2.0 - 1.0,
        head_y - 1.0,
        head_size + 2.0,
        11.0,
        outline_color,
    );

    draw_rectangle(screen_x - head_size / 2.0, head_y, head_size, 9.0, color);

    draw_rectangle(
        screen_x - head_size / 2.0,
        head_y - 1.0,
        head_size,
        2.0,
        Color::from_rgba(
            (color.r * 255.0 * 1.3).min(255.0) as u8,
            (color.g * 255.0 * 1.3).min(255.0) as u8,
            (color.b * 255.0 * 1.3).min(255.0) as u8,
            255,
        ),
    );

    let eye_offset_x = if flip { 1.0 } else { -1.0 };
    let eye1_x = screen_x - 3.5 + eye_offset_x;
    let eye2_x = screen_x + 2.5 + eye_offset_x;
    let eye_y = head_y + 3.0;

    draw_circle(eye1_x, eye_y, 2.0, WHITE);
    draw_circle(eye2_x, eye_y, 2.0, WHITE);
    draw_circle(eye1_x + eye_offset_x * 0.5, eye_y, 1.2, BLACK);
    draw_circle(eye2_x + eye_offset_x * 0.5, eye_y, 1.2, BLACK);

    if is_attacking {
        draw_line(
            screen_x - 2.0,
            head_y + 6.5,
            screen_x + 2.0,
            head_y + 7.5,
            1.5,
            Color::from_rgba(0, 0, 0, 255),
        );
    }
}
