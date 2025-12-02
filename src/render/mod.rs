use crate::count_shader;
use macroquad::prelude::*;
use std::sync::OnceLock;

pub mod menu_shader;

static CUSTOM_FONT: OnceLock<Font> = OnceLock::new();
static Q3_NUMBERS: OnceLock<Q3Numbers> = OnceLock::new();
static Q3_BIGCHARS: OnceLock<Texture2D> = OnceLock::new();
static Q3_FONT2_PROP: OnceLock<Texture2D> = OnceLock::new();
static Q3_FONT1_PROP: OnceLock<Texture2D> = OnceLock::new();
static HUD_ICONS: OnceLock<HudIcons> = OnceLock::new();
static ITEM_ICONS: OnceLock<ItemIcons> = OnceLock::new();

pub struct Q3Numbers {
    pub digits: [Texture2D; 10],
    pub minus: Texture2D,
}

pub struct HudIcons {
    pub health_green: Texture2D,
    pub health_yellow: Texture2D,
    pub health_red: Texture2D,
    pub health_mega: Texture2D,
    pub armor_yellow: Texture2D,
    pub armor_red: Texture2D,
    pub armor_shard: Texture2D,
    pub weapon_gauntlet: Texture2D,
    pub weapon_machinegun: Texture2D,
    pub weapon_shotgun: Texture2D,
    pub weapon_grenade: Texture2D,
    pub weapon_rocket: Texture2D,
    pub weapon_lightning: Texture2D,
    pub weapon_railgun: Texture2D,
    pub weapon_plasma: Texture2D,
    pub weapon_bfg: Texture2D,
    pub ammo_machinegun: Texture2D,
    pub ammo_shotgun: Texture2D,
    pub ammo_grenade: Texture2D,
    pub ammo_rocket: Texture2D,
    pub ammo_lightning: Texture2D,
    pub ammo_railgun: Texture2D,
    pub ammo_plasma: Texture2D,
    pub ammo_bfg: Texture2D,
}

pub struct ItemIcons {
    pub health_green: Texture2D,
    pub health_yellow: Texture2D,
    pub health_red: Texture2D,
    pub health_mega: Texture2D,
    pub armor_yellow: Texture2D,
    pub armor_red: Texture2D,
    pub armor_shard: Texture2D,
    pub weapon_gauntlet: Texture2D,
    pub weapon_machinegun: Texture2D,
    pub weapon_shotgun: Texture2D,
    pub weapon_grenade: Texture2D,
    pub weapon_rocket: Texture2D,
    pub weapon_lightning: Texture2D,
    pub weapon_railgun: Texture2D,
    pub weapon_plasma: Texture2D,
    pub weapon_bfg: Texture2D,
    pub quad: Texture2D,
    pub regen: Texture2D,
    pub battle: Texture2D,
    pub flight: Texture2D,
    pub haste: Texture2D,
    pub invis: Texture2D,
}

const PROPB_HEIGHT: f32 = 36.0;
const PROPB_SPACE_WIDTH: f32 = 12.0;
const PROPB_GAP_WIDTH: f32 = 4.0;
const PROPB_MAP: [(u16, u16, u16); 26] = [
    (11, 12, 33),
    (49, 12, 31),
    (85, 12, 31),
    (120, 12, 30),
    (156, 12, 21),
    (183, 12, 21),
    (207, 12, 32),
    (13, 55, 30),
    (49, 55, 13),
    (66, 55, 29),
    (101, 55, 31),
    (135, 55, 21),
    (158, 55, 40),
    (204, 55, 32),
    (12, 97, 31),
    (48, 97, 31),
    (82, 97, 30),
    (118, 97, 30),
    (153, 97, 30),
    (185, 97, 25),
    (213, 97, 30),
    (11, 139, 32),
    (42, 139, 51),
    (93, 139, 32),
    (126, 139, 31),
    (158, 139, 25),
];
const _PROP_MAP: [(u16, u16, u16); 128] = [
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 8),
    (11, 122, 7),
    (154, 181, 14),
    (55, 122, 17),
    (79, 122, 18),
    (101, 122, 23),
    (153, 122, 18),
    (9, 93, 7),
    (207, 122, 8),
    (230, 122, 9),
    (177, 122, 18),
    (30, 152, 18),
    (85, 181, 7),
    (34, 93, 11),
    (110, 181, 6),
    (130, 152, 14),
    (22, 64, 17),
    (41, 64, 12),
    (58, 64, 17),
    (78, 64, 18),
    (98, 64, 19),
    (120, 64, 18),
    (141, 64, 18),
    (204, 64, 16),
    (162, 64, 17),
    (182, 64, 18),
    (59, 181, 7),
    (35, 181, 7),
    (203, 152, 14),
    (56, 93, 14),
    (228, 152, 14),
    (177, 181, 18),
    (28, 122, 22),
    (5, 4, 18),
    (27, 4, 18),
    (48, 4, 18),
    (69, 4, 17),
    (90, 4, 13),
    (106, 4, 13),
    (121, 4, 18),
    (143, 4, 17),
    (164, 4, 8),
    (175, 4, 16),
    (195, 4, 18),
    (216, 4, 12),
    (230, 4, 23),
    (6, 34, 18),
    (27, 34, 18),
    (48, 34, 18),
    (68, 34, 18),
    (90, 34, 17),
    (110, 34, 18),
    (130, 34, 14),
    (146, 34, 18),
    (166, 34, 19),
    (185, 34, 29),
    (215, 34, 18),
    (234, 34, 18),
    (5, 64, 14),
    (60, 152, 7),
    (106, 151, 13),
    (83, 152, 7),
    (128, 122, 17),
    (4, 152, 21),
    (134, 181, 5),
    (5, 4, 18),
    (27, 4, 18),
    (48, 4, 18),
    (69, 4, 17),
    (90, 4, 13),
    (106, 4, 13),
    (121, 4, 18),
    (143, 4, 17),
    (164, 4, 8),
    (175, 4, 16),
    (195, 4, 18),
    (216, 4, 12),
    (230, 4, 23),
    (6, 34, 18),
    (27, 34, 18),
    (48, 34, 18),
    (68, 34, 18),
    (90, 34, 17),
    (110, 34, 18),
    (130, 34, 14),
    (146, 34, 18),
    (166, 34, 19),
    (185, 34, 29),
    (215, 34, 18),
    (234, 34, 18),
    (5, 64, 14),
    (153, 152, 13),
    (11, 181, 5),
    (180, 152, 13),
    (79, 93, 17),
    (0, 0, 0),
];

pub async fn load_custom_font() {
    let font_bytes = include_bytes!("../../assets/fonts/RobotoMono.ttf");
    match load_ttf_font_from_bytes(font_bytes) {
        Ok(font) => {
            let _ = CUSTOM_FONT.set(font);
            println!("[Font] Loaded custom font: Monaco");
        }
        Err(e) => {
            println!("[Font] Failed to load custom font: {}", e);
        }
    }
}

pub async fn load_q3_numbers() {
    let digit_names = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    let mut digits: Vec<Texture2D> = Vec::new();

    for name in &digit_names {
        let path = format!("q3-resources/gfx/2d/numbers/{}_32b.png", name);
        match load_texture(&path).await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                digits.push(tex);
            }
            Err(_) => {
                let fallback = Image::gen_image_color(32, 48, WHITE);
                digits.push(Texture2D::from_image(&fallback));
            }
        }
    }

    let minus_path = "q3-resources/gfx/2d/numbers/minus_32b.png";
    let minus = match load_texture(minus_path).await {
        Ok(tex) => {
            tex.set_filter(FilterMode::Linear);
            tex
        }
        Err(_) => {
            let fallback = Image::gen_image_color(32, 48, WHITE);
            Texture2D::from_image(&fallback)
        }
    };

    let numbers = Q3Numbers {
        digits: digits
            .try_into()
            .unwrap_or_else(|_| panic!("Invalid digit count")),
        minus,
    };

    let _ = Q3_NUMBERS.set(numbers);
    println!("[Q3HUD] Loaded Q3-style number textures");
}

pub async fn load_q3_bigchars() {
    let bigchars_path = "q3-resources/gfx/2d/bigchars.png";
    match load_texture(bigchars_path).await {
        Ok(tex) => {
            tex.set_filter(FilterMode::Linear);
            let _ = Q3_BIGCHARS.set(tex);
            println!("[Q3HUD] Loaded Q3 bigchars font");
        }
        Err(e) => {
            println!("[Q3HUD] Failed to load bigchars: {}", e);
        }
    }
}

pub async fn load_q3_font2_prop() {
    let path = "q3-resources/menu/art/font2_prop.png";
    match load_texture(path).await {
        Ok(tex) => {
            tex.set_filter(FilterMode::Linear);
            let _ = Q3_FONT2_PROP.set(tex);
            println!("[Q3HUD] Loaded Q3 font2_prop");
        }
        Err(e) => {
            println!("[Q3HUD] Failed to load font2_prop: {}", e);
        }
    }
}

pub async fn load_q3_font1_prop() {
    let path = "q3-resources/menu/art/font1_prop.png";
    match load_texture(path).await {
        Ok(tex) => {
            tex.set_filter(FilterMode::Linear);
            let _ = Q3_FONT1_PROP.set(tex);
            println!("[Q3HUD] Loaded Q3 font1_prop");
        }
        Err(e) => {
            println!("[Q3HUD] Failed to load font1_prop: {}", e);
        }
    }
}

pub async fn load_hud_icons() {
    async fn load_icon(path: &str) -> Texture2D {
        match load_texture(path).await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                tex
            }
            Err(_) => {
                let fallback = Image::gen_image_color(16, 16, WHITE);
                Texture2D::from_image(&fallback)
            }
        }
    }

    let icons = HudIcons {
        health_green: load_icon("q3-resources/icons/iconh_green.png").await,
        health_yellow: load_icon("q3-resources/icons/iconh_yellow.png").await,
        health_red: load_icon("q3-resources/icons/iconh_red.png").await,
        health_mega: load_icon("q3-resources/icons/iconh_mega.png").await,
        armor_yellow: load_icon("q3-resources/icons/iconr_yellow.png").await,
        armor_red: load_icon("q3-resources/icons/iconr_red.png").await,
        armor_shard: load_icon("q3-resources/icons/iconr_shard.png").await,
        weapon_gauntlet: load_icon("q3-resources/icons/iconw_gauntlet.png").await,
        weapon_machinegun: load_icon("q3-resources/icons/iconw_machinegun.png").await,
        weapon_shotgun: load_icon("q3-resources/icons/iconw_shotgun.png").await,
        weapon_grenade: load_icon("q3-resources/icons/iconw_grenade.png").await,
        weapon_rocket: load_icon("q3-resources/icons/iconw_rocket.png").await,
        weapon_lightning: load_icon("q3-resources/icons/iconw_lightning.png").await,
        weapon_railgun: load_icon("q3-resources/icons/iconw_railgun.png").await,
        weapon_plasma: load_icon("q3-resources/icons/iconw_plasma.png").await,
        weapon_bfg: load_icon("q3-resources/icons/iconw_bfg.png").await,
        ammo_machinegun: load_icon("q3-resources/icons/icona_machinegun.png").await,
        ammo_shotgun: load_icon("q3-resources/icons/icona_shotgun.png").await,
        ammo_grenade: load_icon("q3-resources/icons/icona_grenade.png").await,
        ammo_rocket: load_icon("q3-resources/icons/icona_rocket.png").await,
        ammo_lightning: load_icon("q3-resources/icons/icona_lightning.png").await,
        ammo_railgun: load_icon("q3-resources/icons/icona_railgun.png").await,
        ammo_plasma: load_icon("q3-resources/icons/icona_plasma.png").await,
        ammo_bfg: load_icon("q3-resources/icons/icona_bfg.png").await,
    };

    let _ = HUD_ICONS.set(icons);
    println!("[Q3HUD] Loaded HUD icons");
}

pub async fn load_item_icons() {
    async fn load_icon(path: &str) -> Texture2D {
        match load_texture(path).await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                tex
            }
            Err(_) => {
                let fallback = Image::gen_image_color(32, 32, WHITE);
                Texture2D::from_image(&fallback)
            }
        }
    }

    let icons = ItemIcons {
        health_green: load_icon("q3-resources/icons/iconh_green.png").await,
        health_yellow: load_icon("q3-resources/icons/iconh_yellow.png").await,
        health_red: load_icon("q3-resources/icons/iconh_red.png").await,
        health_mega: load_icon("q3-resources/icons/iconh_mega.png").await,
        armor_yellow: load_icon("q3-resources/icons/iconr_yellow.png").await,
        armor_red: load_icon("q3-resources/icons/iconr_red.png").await,
        armor_shard: load_icon("q3-resources/icons/iconr_shard.png").await,
        weapon_gauntlet: load_icon("q3-resources/icons/iconw_gauntlet.png").await,
        weapon_machinegun: load_icon("q3-resources/icons/iconw_machinegun.png").await,
        weapon_shotgun: load_icon("q3-resources/icons/iconw_shotgun.png").await,
        weapon_grenade: load_icon("q3-resources/icons/iconw_grenade.png").await,
        weapon_rocket: load_icon("q3-resources/icons/iconw_rocket.png").await,
        weapon_lightning: load_icon("q3-resources/icons/iconw_lightning.png").await,
        weapon_railgun: load_icon("q3-resources/icons/iconw_railgun.png").await,
        weapon_plasma: load_icon("q3-resources/icons/iconw_plasma.png").await,
        weapon_bfg: load_icon("q3-resources/icons/iconw_bfg.png").await,
        quad: load_icon("q3-resources/icons/quad.png").await,
        regen: load_icon("q3-resources/icons/regen.png").await,
        battle: load_icon("q3-resources/icons/envirosuit.png").await,
        flight: load_icon("q3-resources/icons/flight.png").await,
        haste: load_icon("q3-resources/icons/haste.png").await,
        invis: load_icon("q3-resources/icons/invis.png").await,
    };

    let _ = ITEM_ICONS.set(icons);
    println!("[ItemIcons] Loaded item icons");
}

pub fn get_item_icons() -> Option<&'static ItemIcons> {
    ITEM_ICONS.get()
}

pub fn draw_q3_small_char(x: f32, y: f32, size: f32, ch: u8, color: Color) {
    if let Some(bigchars) = Q3_BIGCHARS.get() {
        let ch_idx = ch as usize;
        let row = ch_idx / 16;
        let col = ch_idx % 16;

        let char_size = 16.0;
        let src_x = (col as f32) * char_size;
        let src_y = (row as f32) * char_size;

        draw_texture_ex(
            bigchars,
            x,
            y,
            color,
            DrawTextureParams {
                source: Some(Rect::new(src_x, src_y, char_size, char_size)),
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    } else {
        let ch_char = ch as char;
        draw_text(&ch_char.to_string(), x, y + size, size, color);
    }
}

fn draw_q3_char(x: f32, y: f32, ch: char, size: f32, color: Color) {
    let bigchars = match Q3_BIGCHARS.get() {
        Some(t) => t,
        None => return,
    };

    let ch_code = ch as u8;
    let col: i32;
    let row: i32;

    row = (ch_code >> 4) as i32;
    col = (ch_code & 15) as i32;

    let char_w = 16.0;
    let char_h = 16.0;
    let src_x = col as f32 * char_w;
    let src_y = row as f32 * char_h;

    let dx = x.round();
    let dy = y.round();
    let ds = size.round();
    let inset = 0.5;

    draw_texture_ex(
        bigchars,
        dx,
        dy,
        color,
        DrawTextureParams {
            source: Some(Rect::new(
                src_x + inset,
                src_y + inset,
                char_w - inset * 2.0,
                char_h - inset * 2.0,
            )),
            dest_size: Some(vec2(ds, ds)),
            ..Default::default()
        },
    );
    count_shader!("ui_default");
}

pub fn draw_q3_string(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let advance = size.round();
    let mut curr_x = x.round();
    let base_y = y.round();
    for ch in text.chars() {
        if ch != ' ' {
            draw_q3_char(curr_x, base_y, ch, advance, color);
        }
        curr_x += advance;
    }
}

pub fn draw_q3_banner_string(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let tex = match Q3_FONT2_PROP.get() {
        Some(t) => t,
        None => {
            draw_q3_string(text, x, y, size, color);
            return;
        }
    };
    let mut ax = x.round();
    let ay = y.round();
    let size_scale = size / PROPB_HEIGHT;
    let shadow = Color::from_rgba(0, 0, 0, (color.a * 255.0).round() as u8);
    for ch in text.chars() {
        let upper = ch.to_ascii_uppercase();
        if upper == ' ' {
            ax += (PROPB_SPACE_WIDTH + PROPB_GAP_WIDTH) * size_scale;
            continue;
        }
        if ('A'..='Z').contains(&upper) {
            let idx = (upper as u8 - b'A') as usize;
            let (sx, sy, w) = PROPB_MAP[idx];
            let aw = (w as f32) * size_scale;
            let ah = PROPB_HEIGHT * size_scale;
            let dx = ax.round();
            let dy = ay.round();
            let inset = 0.5;
            let src = Rect::new(
                sx as f32 + inset,
                sy as f32 + inset,
                (w as f32) - inset * 2.0,
                PROPB_HEIGHT - inset * 2.0,
            );
            draw_texture_ex(
                tex,
                dx + 2.0,
                dy + 2.0,
                shadow,
                DrawTextureParams {
                    source: Some(src),
                    dest_size: Some(vec2(aw, ah)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                tex,
                dx,
                dy,
                color,
                DrawTextureParams {
                    source: Some(src),
                    dest_size: Some(vec2(aw, ah)),
                    ..Default::default()
                },
            );
            ax += (aw + PROPB_GAP_WIDTH * size_scale).round();
        } else {
            draw_q3_char(ax + 2.0, ay + 2.0, upper, size, shadow);
            draw_q3_char(ax, ay, upper, size, color);
            ax += size.round();
        }
    }
}

pub fn measure_q3_banner_string(text: &str, size: f32) -> f32 {
    if Q3_FONT2_PROP.get().is_none() {
        return text.len() as f32 * size;
    }

    let mut width = 0.0;
    let size_scale = size / PROPB_HEIGHT;

    for ch in text.chars() {
        let upper = ch.to_ascii_uppercase();
        if upper == ' ' {
            width += (PROPB_SPACE_WIDTH + PROPB_GAP_WIDTH) * size_scale;
            continue;
        }
        if ('A'..='Z').contains(&upper) {
            let idx = (upper as u8 - b'A') as usize;
            let (_sx, _sy, w) = PROPB_MAP[idx];
            let aw = (w as f32) * size_scale;
            width += (aw + PROPB_GAP_WIDTH * size_scale).round();
        } else {
            width += size.round();
        }
    }

    width
}

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub zoom: f32,
    pub target_zoom: f32,
    pub dead_zone_w: f32,
    pub dead_zone_h: f32,
    pub shake_x: f32,
    pub shake_y: f32,
    pub shake_intensity: f32,
    pub tracking_projectile_id: Option<u32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            zoom: 1.0,
            target_zoom: 1.0,
            dead_zone_w: 160.0,
            dead_zone_h: 120.0,
            shake_x: 0.0,
            shake_y: 0.0,
            shake_intensity: 0.0,
            tracking_projectile_id: None,
        }
    }

    pub fn follow(&mut self, target_x: f32, target_y: f32) {
        self.follow_with_size(target_x, target_y, screen_width(), screen_height());
    }
    
    pub fn follow_with_size(&mut self, target_x: f32, target_y: f32, view_w: f32, view_h: f32) {
        const HUD_HEIGHT: f32 = 80.0;

        let effective_view_h = view_h - HUD_HEIGHT;

        let center_y_offset = effective_view_h * 0.5;

        let left = self.x + view_w * 0.5 - self.dead_zone_w * 0.5;
        let right = self.x + view_w * 0.5 + self.dead_zone_w * 0.5;
        let top = self.y + center_y_offset - self.dead_zone_h * 0.5;
        let bottom = self.y + center_y_offset + self.dead_zone_h * 0.5;

        let mut desired_x = self.x;
        let mut desired_y = self.y;
        if target_x < left {
            desired_x = target_x - (view_w * 0.5 - self.dead_zone_w * 0.5);
        }
        if target_x > right {
            desired_x = target_x - (view_w * 0.5 + self.dead_zone_w * 0.5);
        }
        if target_y < top {
            desired_y = target_y - (center_y_offset - self.dead_zone_h * 0.5);
        }
        if target_y > bottom {
            desired_y = target_y - (center_y_offset + self.dead_zone_h * 0.5);
        }

        self.target_x = desired_x;
        self.target_y = desired_y;
        self.target_zoom = 1.0;
    }

    pub fn follow_two_players(&mut self, p1_x: f32, p1_y: f32, p2_x: f32, p2_y: f32) {
        const HUD_HEIGHT: f32 = 80.0;

        let center_x = (p1_x + p2_x) * 0.5;
        let center_y = (p1_y + p2_y) * 0.5;

        let dist_x = (p1_x - p2_x).abs();
        let dist_y = (p1_y - p2_y).abs();

        let view_w = screen_width();
        let view_h = screen_height();
        let effective_view_h = view_h - HUD_HEIGHT;

        let min_zoom_w = dist_x + 400.0;
        let min_zoom_h = dist_y + 300.0;

        let zoom_factor_x = if min_zoom_w > view_w {
            min_zoom_w / view_w
        } else {
            1.0
        };
        let zoom_factor_y = if min_zoom_h > effective_view_h {
            min_zoom_h / effective_view_h
        } else {
            1.0
        };
        let zoom_factor = zoom_factor_x.max(zoom_factor_y).min(2.5);

        self.dead_zone_w = 160.0 * zoom_factor;
        self.dead_zone_h = 120.0 * zoom_factor;

        self.target_x = center_x - view_w * 0.5;
        self.target_y = center_y - effective_view_h * 0.5;
        self.target_zoom = 1.0;
    }

    pub fn follow_projectile(&mut self, projectile_x: f32, projectile_y: f32) {
        const HUD_HEIGHT: f32 = 80.0;

        let view_w = screen_width();
        let view_h = screen_height();
        let effective_view_h = view_h - HUD_HEIGHT;

        self.target_x = projectile_x - view_w * 0.5;
        self.target_y = projectile_y - effective_view_h * 0.5;
    }

    pub fn follow_projectile_with_zoom(&mut self, projectile_x: f32, projectile_y: f32) {
        self.follow_projectile_with_zoom_size(projectile_x, projectile_y, screen_width(), screen_height());
    }
    
    pub fn follow_projectile_with_zoom_size(&mut self, projectile_x: f32, projectile_y: f32, view_w: f32, view_h: f32) {
        const HUD_HEIGHT: f32 = 80.0;

        let effective_view_h = view_h - HUD_HEIGHT;

        self.target_x = projectile_x - view_w * 0.5;
        self.target_y = projectile_y - effective_view_h * 0.5;
        self.target_zoom = 1.2;
    }

    pub fn update(&mut self, dt: f32, map_width: f32, map_height: f32) {
        self.update_with_size(dt, map_width, map_height, screen_width(), screen_height());
    }
    
    pub fn update_with_size(&mut self, dt: f32, map_width: f32, map_height: f32, screen_w: f32, screen_h: f32) {
        const SMOOTHNESS: f32 = 3.0;
        const HUD_HEIGHT: f32 = 80.0;

        self.x += (self.target_x - self.x) * SMOOTHNESS * dt;
        self.y += (self.target_y - self.y) * SMOOTHNESS * dt;
        self.zoom += (self.target_zoom - self.zoom) * 2.0 * dt;

        let map_w_pixels = map_width * 32.0;
        let map_h_pixels = map_height * 16.0;

        let effective_view_h = screen_h - HUD_HEIGHT;

        self.x = self.x.clamp(0.0, (map_w_pixels - screen_w).max(0.0));
        self.y = self.y.clamp(0.0, (map_h_pixels - effective_view_h).max(0.0));

        if self.shake_intensity > 0.1 {
            self.shake_x = fastrand::f32() * self.shake_intensity * 2.0 - self.shake_intensity;
            self.shake_y = fastrand::f32() * self.shake_intensity * 2.0 - self.shake_intensity;
            self.shake_intensity *= 0.85;
        } else {
            self.shake_x = 0.0;
            self.shake_y = 0.0;
            self.shake_intensity = 0.0;
        }

        self.x = (self.x + self.shake_x).round();
        self.y = (self.y + self.shake_y).round();
    }
}

pub fn draw_text_outlined(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let shadow = Color::from_rgba(0, 0, 0, 255);

    if let Some(font) = CUSTOM_FONT.get() {
        let params = TextParams {
            font: Some(font),
            font_size: size as u16,
            color: shadow,
            ..Default::default()
        };
        draw_text_ex(text, x + 1.0, y + 1.0, params);

        let params = TextParams {
            font: Some(font),
            font_size: size as u16,
            color,
            ..Default::default()
        };
        draw_text_ex(text, x, y, params);
    } else {
        draw_text(text, x + 1.0, y + 1.0, size, shadow);
        draw_text(text, x, y, size, color);
    }
}

pub fn draw_hud(
    health: i32,
    armor: i32,
    ammo: u8,
    _weapon_name: &str,
    frags: i32,
    weapon: u8,
    leader_frags: i32,
    has_weapon: &[bool; 9],
    ammo_counts: &[u8; 9],
    match_time: f32,
    time_limit: f32,
) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    let hud_height = 80.0;
    let hud_y = screen_h - hud_height;

    draw_rectangle(
        0.0,
        hud_y,
        screen_w,
        hud_height,
        Color::from_rgba(0, 0, 0, 180),
    );
    draw_line(
        0.0,
        hud_y,
        screen_w,
        hud_y,
        2.0,
        Color::from_rgba(100, 100, 100, 255),
    );

    let q3_red = Color::from_rgba(255, 51, 51, 255);
    let q3_yellow = Color::from_rgba(255, 255, 0, 255);
    let q3_white = Color::from_rgba(255, 255, 255, 255);
    let q3_green = Color::from_rgba(0, 255, 0, 255);
    let q3_blue = Color::from_rgba(100, 149, 237, 255);

    let number_size = 40.0;
    let number_y = hud_y + 35.0;

    let ammo_x = 30.0;
    let ammo_color = if ammo < 10 { q3_red } else { q3_white };
    draw_hud_element_with_icon(
        ammo_x,
        number_y,
        ammo as i32,
        ammo_color,
        number_size,
        get_ammo_icon(weapon),
    );

    let health_x = 150.0;
    let health_color = if health > super::game::constants::STARTING_HEALTH {
        q3_blue
    } else if health > 75 {
        q3_green
    } else if health > 25 {
        q3_yellow
    } else {
        q3_red
    };
    draw_hud_element_with_icon(
        health_x,
        number_y,
        health,
        health_color,
        number_size,
        get_health_icon(health),
    );

    let armor_x = screen_w - 120.0;
    if armor > 0 {
        let armor_color = q3_green;
        draw_hud_element_with_icon(
            armor_x,
            number_y,
            armor,
            armor_color,
            number_size,
            get_armor_icon(armor),
        );
    }

    draw_weapon_icons_with_ammo(weapon, hud_y + 5.0, has_weapon, ammo_counts);

    draw_match_timer_and_scores(frags, leader_frags, match_time, time_limit);
}

fn draw_hud_element_with_icon(
    x: f32,
    y: f32,
    number: i32,
    color: Color,
    size: f32,
    icon: Option<&Texture2D>,
) {
    if let Some(icon_tex) = icon {
        let icon_size = 24.0;
        draw_texture_ex(
            icon_tex,
            x - icon_size - 5.0,
            y + (size - icon_size) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(icon_size, icon_size)),
                ..Default::default()
            },
        );
    }
    draw_large_q3_number(x, y, number, color, size);
}

fn get_health_icon(health: i32) -> Option<&'static Texture2D> {
    let icons = HUD_ICONS.get()?;
    Some(if health > super::game::constants::STARTING_HEALTH {
        &icons.health_mega
    } else if health > 75 {
        &icons.health_green
    } else if health > 25 {
        &icons.health_yellow
    } else {
        &icons.health_red
    })
}

fn get_armor_icon(armor: i32) -> Option<&'static Texture2D> {
    let icons = HUD_ICONS.get()?;
    Some(if armor > 100 {
        &icons.armor_red
    } else if armor > 50 {
        &icons.armor_yellow
    } else {
        &icons.armor_shard
    })
}

fn get_ammo_icon(weapon: u8) -> Option<&'static Texture2D> {
    let icons = HUD_ICONS.get()?;
    Some(match weapon {
        1 => &icons.ammo_machinegun,
        2 => &icons.ammo_shotgun,
        3 => &icons.ammo_grenade,
        4 => &icons.ammo_rocket,
        5 => &icons.ammo_lightning,
        6 => &icons.ammo_railgun,
        7 => &icons.ammo_plasma,
        8 => &icons.ammo_bfg,
        _ => &icons.ammo_machinegun,
    })
}

fn get_weapon_icon(weapon: u8) -> Option<&'static Texture2D> {
    let icons = HUD_ICONS.get()?;
    Some(match weapon {
        0 => &icons.weapon_gauntlet,
        1 => &icons.weapon_machinegun,
        2 => &icons.weapon_shotgun,
        3 => &icons.weapon_grenade,
        4 => &icons.weapon_rocket,
        5 => &icons.weapon_lightning,
        6 => &icons.weapon_railgun,
        7 => &icons.weapon_plasma,
        8 => &icons.weapon_bfg,
        _ => &icons.weapon_gauntlet,
    })
}

fn draw_large_q3_number(x: f32, y: f32, number: i32, color: Color, size: f32) {
    let numbers = match Q3_NUMBERS.get() {
        Some(n) => n,
        None => return,
    };

    let num_str = format!("{}", number.abs());
    let char_width = size * 0.6;
    let mut draw_x = x;

    if number < 0 {
        draw_texture_ex(
            &numbers.minus,
            draw_x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(char_width, size)),
                ..Default::default()
            },
        );
        draw_x += char_width;
    }

    for ch in num_str.chars() {
        if let Some(digit) = ch.to_digit(10) {
            draw_texture_ex(
                &numbers.digits[digit as usize],
                draw_x,
                y,
                color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(char_width, size)),
                    ..Default::default()
                },
            );
            draw_x += char_width;
        }
    }
}

fn draw_weapon_icons_with_ammo(
    current_weapon: u8,
    y: f32,
    has_weapon: &[bool; 9],
    ammo_counts: &[u8; 9],
) {
    let screen_w = screen_width();
    let icon_size = 28.0;
    let icon_spacing = 35.0;

    let available_weapons: Vec<u8> = (1..=8)
        .filter(|&weapon_id| {
            let idx = weapon_id as usize;
            has_weapon[idx] && (ammo_counts[idx] > 0 || weapon_id == 1)
        })
        .collect();

    if available_weapons.is_empty() {
        return;
    }

    let total_width = available_weapons.len() as f32 * icon_spacing - icon_spacing;
    let start_x = screen_w * 0.5 - total_width * 0.5;

    for (i, weapon_id) in available_weapons.iter().enumerate() {
        let x = start_x + i as f32 * icon_spacing;
        let is_current = *weapon_id == current_weapon;
        let weapon_ammo = ammo_counts[*weapon_id as usize];

        let bg_color = if is_current {
            Color::from_rgba(255, 255, 0, 100)
        } else {
            Color::from_rgba(100, 100, 100, 50)
        };

        draw_rectangle(x - 2.0, y - 2.0, icon_size + 4.0, icon_size + 4.0, bg_color);

        if is_current {
            draw_rectangle_lines(
                x - 2.0,
                y - 2.0,
                icon_size + 4.0,
                icon_size + 4.0,
                2.0,
                Color::from_rgba(255, 255, 0, 255),
            );
        }

        if let Some(weapon_icon) = get_weapon_icon(*weapon_id) {
            let icon_color = if is_current {
                WHITE
            } else {
                Color::from_rgba(200, 200, 200, 255)
            };
            draw_texture_ex(
                weapon_icon,
                x + 2.0,
                y + 2.0,
                icon_color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(icon_size - 4.0, icon_size - 4.0)),
                    ..Default::default()
                },
            );
        }

        if *weapon_id != 1 {
            let ammo_color = if weapon_ammo < 10 {
                Color::from_rgba(255, 100, 100, 255)
            } else if is_current {
                Color::from_rgba(255, 255, 0, 255)
            } else {
                Color::from_rgba(200, 200, 200, 255)
            };
            draw_q3_string(
                &weapon_ammo.to_string(),
                x + 1.0,
                y + icon_size - 2.0,
                8.0,
                ammo_color,
            );
        }

        draw_q3_string(
            &weapon_id.to_string(),
            x + icon_size - 8.0,
            y + 8.0,
            6.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }
}

fn draw_match_timer_and_scores(
    player_score: i32,
    competitor_score: i32,
    match_time: f32,
    time_limit: f32,
) {
    let screen_w = screen_width();
    let y = 30.0;

    let time_remaining = (time_limit - match_time).max(0.0);
    let minutes = (time_remaining / 60.0) as u32;
    let seconds = (time_remaining % 60.0) as u32;

    let score_text = format!(
        "{} {:02}:{:02} {}",
        player_score, minutes, seconds, competitor_score
    );
    draw_q3_string(
        &score_text,
        screen_w * 0.5 - 60.0,
        y,
        20.0,
        Color::from_rgba(255, 255, 255, 255),
    );
}

pub fn draw_crosshair(player_x: f32, player_y: f32, camera_x: f32, camera_y: f32, aim_angle: f32) {
    let crosshair_size_cvar = crate::cvar::get_cvar_float("cg_crosshairSize");
    if crosshair_size_cvar <= 0.0 {
        return;
    }

    let distance = 200.0;
    let angle = aim_angle;

    let world_crosshair_x = player_x + angle.cos() * distance;
    let world_crosshair_y = player_y + angle.sin() * distance;

    let screen_crosshair_x = world_crosshair_x - camera_x;
    let screen_crosshair_y = world_crosshair_y - camera_y;

    let size = crosshair_size_cvar * 0.5;
    let gap = 6.0;
    let thickness = 2.0;

    draw_circle(
        screen_crosshair_x,
        screen_crosshair_y,
        2.0,
        Color::from_rgba(0, 255, 0, 100),
    );

    draw_line(
        screen_crosshair_x - size - gap,
        screen_crosshair_y,
        screen_crosshair_x - gap,
        screen_crosshair_y,
        thickness,
        Color::from_rgba(0, 255, 0, 255),
    );
    draw_line(
        screen_crosshair_x + gap,
        screen_crosshair_y,
        screen_crosshair_x + size + gap,
        screen_crosshair_y,
        thickness,
        Color::from_rgba(0, 255, 0, 255),
    );
    draw_line(
        screen_crosshair_x,
        screen_crosshair_y - size - gap,
        screen_crosshair_x,
        screen_crosshair_y - gap,
        thickness,
        Color::from_rgba(0, 255, 0, 255),
    );
    draw_line(
        screen_crosshair_x,
        screen_crosshair_y + gap,
        screen_crosshair_x,
        screen_crosshair_y + size + gap,
        thickness,
        Color::from_rgba(0, 255, 0, 255),
    );

    draw_circle_lines(
        screen_crosshair_x,
        screen_crosshair_y,
        gap - 1.0,
        1.0,
        Color::from_rgba(0, 255, 0, 150),
    );
}

pub fn draw_main_menu(selected: usize, items: &[&str], _hover_idx: Option<usize>) {
    let w = screen_width();
    let h = screen_height();
    clear_background(Color::from_rgba(18, 22, 28, 255));

    draw_q3_banner_string(
        "SAS III",
        w * 0.5 - 100.0,
        80.0,
        48.0,
        Color::from_rgba(255, 176, 0, 255),
    );
    draw_q3_banner_string(
        "STILL ALIVE SOMEHOW??",
        w * 0.5 - 160.0,
        130.0,
        24.0,
        Color::from_rgba(180, 220, 255, 220),
    );

    let item_h = 54.0;
    let item_w = 400.0;
    let start_y = h * 0.5 - (items.len() as f32 * (item_h + 12.0)) * 0.5;

    for (i, label) in items.iter().enumerate() {
        let y = start_y + (i as f32) * (item_h + 12.0);
        let x = w * 0.5 - item_w * 0.5;

        let text_color = if i == selected {
            Color::from_rgba(255, 64, 64, 255)
        } else {
            Color::from_rgba(210, 220, 230, 255)
        };
        if i == selected {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 36.0, text_color);
        } else {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 30.0, text_color);
        }
    }

    draw_q3_string(
        "ENTER/CLICK TO SELECT",
        w * 0.5 - 120.0,
        h - 40.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
}

pub fn draw_map_select_menu(selected: usize, items: &[&str], _hover_idx: Option<usize>) {
    println!(
        "[RENDER] draw_map_select_menu called with {} items: {:?}",
        items.len(),
        items
    );

    let w = screen_width();
    let h = screen_height();
    clear_background(Color::from_rgba(18, 22, 28, 255));

    draw_q3_banner_string(
        "SELECT MAP",
        w * 0.5 - 100.0,
        80.0,
        40.0,
        Color::from_rgba(255, 176, 0, 255),
    );

    if items.is_empty() {
        draw_q3_banner_string(
            "NO MAPS FOUND",
            w * 0.5 - 120.0,
            h * 0.5,
            32.0,
            Color::from_rgba(255, 100, 100, 255),
        );
        println!("[RENDER] WARNING: No items to display!");
        return;
    }

    let item_h = 54.0;
    let item_w = 400.0;
    let start_y = h * 0.5 - (items.len() as f32 * (item_h + 12.0)) * 0.5;

    for (i, label) in items.iter().enumerate() {
        println!("[RENDER] Drawing item {}: '{}'", i, label);
        let y = start_y + (i as f32) * (item_h + 12.0);
        let x = w * 0.5 - item_w * 0.5;

        let text_color = if i == selected {
            Color::from_rgba(255, 64, 64, 255)
        } else {
            Color::from_rgba(210, 220, 230, 255)
        };
        if i == selected {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 36.0, text_color);
        } else {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 30.0, text_color);
        }
    }

    draw_q3_string(
        "ENTER TO SELECT  ESC TO GO BACK",
        w * 0.5 - 140.0,
        h - 40.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
}

pub fn draw_1v1_map_select_menu(selected: usize, items: &[&str], _hover_idx: Option<usize>) {
    println!(
        "[RENDER] draw_1v1_map_select_menu called with {} items: {:?}",
        items.len(),
        items
    );

    let w = screen_width();
    let h = screen_height();
    clear_background(Color::from_rgba(18, 22, 28, 255));

    draw_q3_banner_string(
        "1V1 LOCAL",
        w * 0.5 - 80.0,
        60.0,
        40.0,
        Color::from_rgba(255, 176, 0, 255),
    );
    draw_q3_banner_string(
        "SELECT MAP",
        w * 0.5 - 100.0,
        110.0,
        32.0,
        Color::from_rgba(180, 220, 255, 220),
    );

    if items.is_empty() {
        draw_q3_banner_string(
            "NO MAPS FOUND",
            w * 0.5 - 120.0,
            h * 0.5,
            32.0,
            Color::from_rgba(255, 100, 100, 255),
        );
        println!("[RENDER] WARNING: No items to display!");
        return;
    }

    let item_h = 54.0;
    let item_w = 400.0;
    let start_y = h * 0.5 - (items.len() as f32 * (item_h + 12.0)) * 0.5 + 20.0;

    for (i, label) in items.iter().enumerate() {
        println!("[RENDER] Drawing item {}: '{}'", i, label);
        let y = start_y + (i as f32) * (item_h + 12.0);
        let x = w * 0.5 - item_w * 0.5;

        let text_color = if i == selected {
            Color::from_rgba(255, 64, 64, 255)
        } else {
            Color::from_rgba(210, 220, 230, 255)
        };
        if i == selected {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 36.0, text_color);
        } else {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 30.0, text_color);
        }
    }

    draw_q3_string(
        "CONTROLS:",
        w * 0.5 - 60.0,
        h - 100.0,
        18.0,
        Color::from_rgba(255, 200, 120, 255),
    );
    draw_q3_string(
        "PLAYER1: WASD + SPACE SHOOT + 1-5 WEAPONS",
        w * 0.5 - 200.0,
        h - 78.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
    draw_q3_string(
        "PLAYER2: ARROWS + RCTRL SHOOT + 6-0 WEAPONS",
        w * 0.5 - 200.0,
        h - 62.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
    draw_q3_string(
        "ENTER TO SELECT  ESC TO GO BACK",
        w * 0.5 - 140.0,
        h - 40.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
}

pub fn draw_settings_menu(
    selected: usize,
    items: &[&str],
    selected_model: &str,
    _hover_idx: Option<usize>,
) {
    let w = screen_width();
    let h = screen_height();
    clear_background(Color::from_rgba(18, 22, 28, 255));

    draw_q3_banner_string(
        "SETTINGS",
        w * 0.5 - 80.0,
        60.0,
        48.0,
        Color::from_rgba(255, 176, 0, 255),
    );
    draw_q3_banner_string(
        "PLAYER MODEL",
        w * 0.5 - 120.0,
        110.0,
        32.0,
        Color::from_rgba(180, 220, 255, 220),
    );

    draw_q3_string(
        "CURRENT MODEL:",
        w * 0.5 - 80.0,
        150.0,
        20.0,
        Color::from_rgba(200, 210, 220, 255),
    );
    draw_q3_string(
        selected_model,
        w * 0.5 + 40.0,
        150.0,
        20.0,
        Color::from_rgba(120, 200, 255, 255),
    );

    let max_visible = 6;
    let item_h = 54.0;
    let item_w = 400.0;

    let scroll_offset = if selected >= max_visible {
        selected - max_visible + 1
    } else {
        0
    };

    let visible_items: Vec<(usize, &str)> = items
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_visible)
        .map(|(i, s)| (i, *s))
        .collect();

    let start_y = h * 0.5 - (visible_items.len() as f32 * (item_h + 12.0)) * 0.5 + 40.0;

    for (idx, (orig_i, label)) in visible_items.iter().enumerate() {
        let y = start_y + (idx as f32) * (item_h + 12.0);
        let x = w * 0.5 - item_w * 0.5;

        let text_color = if *orig_i == selected {
            Color::from_rgba(255, 64, 64, 255)
        } else {
            Color::from_rgba(210, 220, 230, 255)
        };
        if *orig_i == selected {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 36.0, text_color);
        } else {
            draw_q3_banner_string(&label.to_uppercase(), x + 18.0, y + 10.0, 30.0, text_color);
        }
    }

    if items.len() > max_visible {
        let scroll_text = format!("{} / {}", selected + 1, items.len());
        draw_q3_string(
            &scroll_text,
            w - 100.0,
            150.0,
            14.0,
            Color::from_rgba(180, 190, 200, 200),
        );
    }

    draw_q3_string(
        "USE UP/DOWN TO CHANGE MODEL",
        w * 0.5 - 140.0,
        h - 80.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
    draw_q3_string(
        "F5/F6 ALSO WORK IN GAME",
        w * 0.5 - 100.0,
        h - 62.0,
        14.0,
        Color::from_rgba(140, 150, 160, 180),
    );
    draw_q3_string(
        "ESC TO GO BACK",
        w * 0.5 - 60.0,
        h - 40.0,
        14.0,
        Color::from_rgba(180, 190, 200, 200),
    );
}

pub fn draw_hud_player2(
    health: i32,
    armor: i32,
    ammo: u8,
    _weapon_name: &str,
    _frags: i32,
    weapon: u8,
    _leader_frags: i32,
    has_weapon: &[bool; 9],
    ammo_counts: &[u8; 9],
) {
    let screen_w = screen_width();

    let hud_height = 80.0;
    let hud_y = 10.0;

    draw_rectangle(
        0.0,
        hud_y,
        screen_w,
        hud_height,
        Color::from_rgba(0, 0, 0, 180),
    );
    draw_line(
        0.0,
        hud_y + hud_height,
        screen_w,
        hud_y + hud_height,
        2.0,
        Color::from_rgba(100, 100, 100, 255),
    );

    let q3_red = Color::from_rgba(255, 51, 51, 255);
    let q3_yellow = Color::from_rgba(255, 255, 0, 255);
    let q3_white = Color::from_rgba(255, 255, 255, 255);
    let q3_blue = Color::from_rgba(100, 149, 237, 255);

    let number_size = 40.0;
    let number_y = hud_y + 35.0;

    let ammo_x = 30.0;
    let ammo_color = if ammo < 10 { q3_red } else { q3_blue };
    draw_hud_element_with_icon(
        ammo_x,
        number_y,
        ammo as i32,
        ammo_color,
        number_size,
        get_ammo_icon(weapon),
    );

    let health_x = 150.0;
    let health_color = if health > 100 {
        q3_white
    } else if health > 40 {
        q3_yellow
    } else {
        q3_red
    };
    draw_hud_element_with_icon(
        health_x,
        number_y,
        health,
        health_color,
        number_size,
        get_health_icon(health),
    );

    let armor_x = screen_w - 120.0;
    if armor > 0 {
        let armor_color = q3_blue;
        draw_hud_element_with_icon(
            armor_x,
            number_y,
            armor,
            armor_color,
            number_size,
            get_armor_icon(armor),
        );
    }

    draw_weapon_icons_player2_with_ammo(weapon, hud_y + 5.0, has_weapon, ammo_counts);
}

fn draw_weapon_icons_player2_with_ammo(
    current_weapon: u8,
    y: f32,
    has_weapon: &[bool; 9],
    ammo_counts: &[u8; 9],
) {
    let screen_w = screen_width();
    let icon_size = 28.0;
    let icon_spacing = 35.0;

    let available_weapons: Vec<u8> = (1..=8)
        .filter(|&weapon_id| {
            let idx = weapon_id as usize;
            has_weapon[idx] && (ammo_counts[idx] > 0 || weapon_id == 1)
        })
        .collect();

    if available_weapons.is_empty() {
        return;
    }

    let total_width = available_weapons.len() as f32 * icon_spacing - icon_spacing;
    let start_x = screen_w * 0.5 - total_width * 0.5;

    for (i, weapon_id) in available_weapons.iter().enumerate() {
        let x = start_x + i as f32 * icon_spacing;
        let is_current = *weapon_id == current_weapon;
        let weapon_ammo = ammo_counts[*weapon_id as usize];

        let bg_color = if is_current {
            Color::from_rgba(100, 149, 237, 100)
        } else {
            Color::from_rgba(100, 100, 100, 50)
        };

        draw_rectangle(x - 2.0, y - 2.0, icon_size + 4.0, icon_size + 4.0, bg_color);

        if is_current {
            draw_rectangle_lines(
                x - 2.0,
                y - 2.0,
                icon_size + 4.0,
                icon_size + 4.0,
                2.0,
                Color::from_rgba(100, 149, 237, 255),
            );
        }

        if let Some(weapon_icon) = get_weapon_icon(*weapon_id) {
            let icon_color = if is_current {
                Color::from_rgba(100, 149, 237, 255)
            } else {
                Color::from_rgba(200, 200, 200, 255)
            };
            draw_texture_ex(
                weapon_icon,
                x + 2.0,
                y + 2.0,
                icon_color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(icon_size - 4.0, icon_size - 4.0)),
                    ..Default::default()
                },
            );
        }

        if *weapon_id != 1 {
            let ammo_color = if weapon_ammo < 10 {
                Color::from_rgba(255, 100, 100, 255)
            } else if is_current {
                Color::from_rgba(100, 149, 237, 255)
            } else {
                Color::from_rgba(200, 200, 200, 255)
            };
            draw_q3_string(
                &weapon_ammo.to_string(),
                x + 1.0,
                y + icon_size - 2.0,
                8.0,
                ammo_color,
            );
        }

        draw_q3_string(
            &weapon_id.to_string(),
            x + icon_size - 8.0,
            y + 8.0,
            6.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }
}

pub fn get_item_icon_for_type(
    item_type: &crate::game::map::ItemType,
) -> Option<&'static Texture2D> {
    let icons = ITEM_ICONS.get()?;

    use crate::game::map::ItemType;
    match item_type {
        ItemType::Health25 => Some(&icons.health_green),
        ItemType::Health50 => Some(&icons.health_yellow),
        ItemType::Health100 => Some(&icons.health_mega),
        ItemType::Armor50 => Some(&icons.armor_yellow),
        ItemType::Armor100 => Some(&icons.armor_red),
        ItemType::Shotgun => Some(&icons.weapon_shotgun),
        ItemType::GrenadeLauncher => Some(&icons.weapon_grenade),
        ItemType::RocketLauncher => Some(&icons.weapon_rocket),
        ItemType::LightningGun => Some(&icons.weapon_lightning),
        ItemType::Railgun => Some(&icons.weapon_railgun),
        ItemType::Plasmagun => Some(&icons.weapon_plasma),
        ItemType::BFG => Some(&icons.weapon_bfg),
        ItemType::Quad => Some(&icons.quad),
        ItemType::Regen => Some(&icons.regen),
        ItemType::Battle => Some(&icons.battle),
        ItemType::Flight => Some(&icons.flight),
        ItemType::Haste => Some(&icons.haste),
        ItemType::Invis => Some(&icons.invis),
    }
}

pub fn draw_item_icon(
    x: f32,
    y: f32,
    item_type: &crate::game::map::ItemType,
    size: f32,
    color: Color,
) {
    if let Some(icon) = get_item_icon_for_type(item_type) {
        let half_size = size * 0.5;
        draw_texture_ex(
            &icon,
            x - half_size,
            y - half_size,
            color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(size, size)),
                ..Default::default()
            },
        );
    }
}
