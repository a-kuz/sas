use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub spawn_points: Vec<SpawnPoint>,
    pub items: Vec<Item>,
    pub jumppads: Vec<JumpPad>,
    pub teleporters: Vec<Teleporter>,
    pub lights: Vec<LightSource>,
    #[serde(default)]
    pub background_elements: Vec<BackgroundElement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackgroundElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub texture_name: String,
    pub alpha: f32,
    pub additive: bool,
    pub scale: f32,
    #[serde(default)]
    pub animation_speed: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub solid: bool,
    pub texture_id: u16,
    #[serde(default)]
    pub shader_name: Option<String>,
    #[serde(default)]
    pub detail_texture: Option<String>,
    #[serde(default)]
    pub glow_texture: Option<String>,
    #[serde(default)]
    pub blend_alpha: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
    pub team: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub item_type: ItemType,
    pub respawn_time: u32,
    pub active: bool,
    #[serde(default)]
    pub vel_x: f32,
    #[serde(default)]
    pub vel_y: f32,
    #[serde(default)]
    pub dropped: bool,
    #[serde(default)]
    pub yaw: f32,
    #[serde(default)]
    pub spin_yaw: f32,
    #[serde(default)]
    pub pitch: f32,
    #[serde(default)]
    pub roll: f32,
    #[serde(default)]
    pub spin_pitch: f32,
    #[serde(default)]
    pub spin_roll: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JumpPad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub force_x: f32,
    pub force_y: f32,
    pub cooldown: u8,
}

impl JumpPad {
    pub fn update(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
    }

    pub fn can_activate(&self) -> bool {
        self.cooldown == 0
    }

    pub fn activate(&mut self) {
        self.cooldown = 30;
    }

    pub fn check_collision(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y - 20.0 && py <= self.y + 20.0
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        use macroquad::prelude::*;

        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        let width = self.width;

        let time = crate::time::get_time() as f32;
        let pulse = (time * 1.5).sin() * 0.5 + 0.5;

        let height = 10.0;
        let inner_height = 6.0;

        draw_rectangle(
            screen_x,
            screen_y,
            width,
            height,
            Color::from_rgba(20, 20, 20, 255),
        );

        draw_rectangle(
            screen_x + 1.0,
            screen_y + 2.0,
            width - 2.0,
            height - 2.0,
            Color::from_rgba(35, 35, 35, 255),
        );

        let inner_glow = (pulse * 100.0 + 80.0) as u8;
        draw_rectangle(
            screen_x + 2.0,
            screen_y,
            width - 4.0,
            inner_height,
            Color::from_rgba(inner_glow, inner_glow / 2, 255, 200),
        );

        draw_rectangle(
            screen_x + 3.0,
            screen_y - inner_height / 2.0 + 1.0,
            width - 6.0,
            inner_height + 2.0,
            Color::from_rgba(0, 0, 20, (pulse * 120.0 + 100.0) as u8),
        );

        let top_line_brightness = (pulse * 80.0 + 100.0) as u8;
        draw_line(
            screen_x + 2.0,
            screen_y + 2.0,
            screen_x + width - 2.0,
            screen_y + 2.0,
            1.0,
            Color::from_rgba(top_line_brightness / 2, top_line_brightness / 2, 10, 200),
        );
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Teleporter {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub dest_x: f32,
    pub dest_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightSource {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub intensity: f32,
    pub flicker: bool,
}

#[derive(Clone, Debug)]
pub struct LinearLight {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub width: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Health25,
    Health50,
    Health100,
    Armor50,
    Armor100,
    Shotgun,
    GrenadeLauncher,
    RocketLauncher,
    LightningGun,
    Railgun,
    Plasmagun,
    BFG,
    Quad,
    Regen,
    Battle,
    Flight,
    Haste,
    Invis,
}

impl Map {
    pub fn new(name: &str) -> Self {
        if let Ok(map) = Self::load_from_file(name) {
            return map;
        }
        
        match name {
            "soldat" => Self::soldat_map(),
            "q3dm6" => Self::q3dm6(),
            _ => Self::soldat_map(),
        }
    }

    pub async fn new_async(name: &str) -> Self {
        if let Ok(map) = Self::load_from_file_async(name).await {
            return map;
        }

        match name {
            "soldat" => Self::soldat_map(),
            "q3dm6" => Self::q3dm6(),
            _ => Self::soldat_map(),
        }
    }

    pub fn load_from_file(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use super::map_loader::MapFile;
        let path = format!("maps/{}.json", name);
        let map_file = MapFile::load_from_file(&path)?;
        Ok(map_file.to_map())
    }

    pub async fn load_from_file_async(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use super::map_loader::MapFile;
        let path = format!("maps/{}.json", name);
        let map_file = MapFile::load_from_file_async(&path).await?;
        Ok(map_file.to_map())
    }

    pub fn soldat_map() -> Self {
        let width = 60;
        let height = 34;
        let mut tiles = vec![
            vec![
                Tile {
                    solid: false,
                    texture_id: 0,
                    shader_name: None,
                    detail_texture: None,
                    glow_texture: None,
                    blend_alpha: 1.0,
                };
                height
            ];
            width
        ];

        // Floor
        for x in 0..width {
            tiles[x][height - 2].solid = true;
        }

        // Top border
        for x in 0..width {
            tiles[x][0].solid = true;
        }

        // Side borders
        for y in 1..height - 1 {
            tiles[0][y].solid = true;
            tiles[width - 1][y].solid = true;
        }

        // Platforms from image (rescaled and adjusted)
        for x in 0..10 {
            tiles[x][8].solid = true;
        }
        for y in 9..14 {
            tiles[9][y].solid = true;
        }

        for x in width - 10..width {
            tiles[x][8].solid = true;
        }
        for y in 9..14 {
            tiles[width - 10][y].solid = true;
        }

        for x in 20..40 {
            tiles[x][12].solid = true;
        }

        for x in 5..20 {
            tiles[x][18].solid = true;
        }
        for x in width - 20..width - 5 {
            tiles[x][18].solid = true;
        }

        for x in 25..35 {
            tiles[x][24].solid = true;
        }

        for x in 0..8 {
            tiles[x][28].solid = true;
        }
        for x in width - 8..width {
            tiles[x][28].solid = true;
        }

        Self {
            width,
            height,
            tiles,
            spawn_points: vec![
                SpawnPoint {
                    x: 160.0,
                    y: 100.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 1120.0,
                    y: 100.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 640.0,
                    y: 160.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 320.0,
                    y: 420.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 960.0,
                    y: 420.0,
                    team: 0,
                },
            ],
            items: vec![
                Item {
                    x: 640.0,
                    y: 170.0,
                    item_type: ItemType::RocketLauncher,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 640.0,
                    y: 420.0,
                    item_type: ItemType::Armor100,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 80.0,
                    y: 100.0,
                    item_type: ItemType::Railgun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1200.0,
                    y: 100.0,
                    item_type: ItemType::Railgun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
            ],
            jumppads: vec![],
            teleporters: vec![],
            lights: vec![],
            background_elements: vec![],
        }
    }

    pub fn q3dm6() -> Self {
        let width = 120;
        let height = 50;
        let mut tiles = vec![
            vec![
                Tile {
                    solid: false,
                    texture_id: 0,
                    shader_name: None,
                    detail_texture: None,
                    glow_texture: None,
                    blend_alpha: 1.0,
                };
                height
            ];
            width
        ];

        const FLOOR: usize = 47;
        const LEVEL_1: usize = 44;
        const LEVEL_2: usize = 40;
        const LEVEL_3: usize = 34;
        const LEVEL_4: usize = 27;
        const LEVEL_5: usize = 20;

        for x in 0..width {
            tiles[x][FLOOR].solid = true;
            tiles[x][FLOOR].texture_id = 1;
            tiles[x][FLOOR + 1].solid = true;
            tiles[x][FLOOR + 1].texture_id = 1;
            tiles[x][FLOOR + 2].solid = true;
            tiles[x][FLOOR + 2].texture_id = 1;
        }

        for y in 0..height {
            tiles[0][y].solid = true;
            tiles[0][y].texture_id = 1;
            tiles[1][y].solid = true;
            tiles[1][y].texture_id = 1;
            tiles[width - 1][y].solid = true;
            tiles[width - 1][y].texture_id = 1;
            tiles[width - 2][y].solid = true;
            tiles[width - 2][y].texture_id = 1;
        }

        for x in 8..26 {
            tiles[x][LEVEL_1].solid = true;
            tiles[x][LEVEL_1].texture_id = 2;
            tiles[x][LEVEL_1 + 1].solid = true;
            tiles[x][LEVEL_1 + 1].texture_id = 2;
        }

        for x in 94..112 {
            tiles[x][LEVEL_1].solid = true;
            tiles[x][LEVEL_1].texture_id = 2;
            tiles[x][LEVEL_1 + 1].solid = true;
            tiles[x][LEVEL_1 + 1].texture_id = 2;
        }

        for x in 30..46 {
            tiles[x][LEVEL_2].solid = true;
            tiles[x][LEVEL_2].texture_id = 3;
            tiles[x][LEVEL_2 + 1].solid = true;
            tiles[x][LEVEL_2 + 1].texture_id = 3;
        }

        for x in 74..90 {
            tiles[x][LEVEL_2].solid = true;
            tiles[x][LEVEL_2].texture_id = 3;
            tiles[x][LEVEL_2 + 1].solid = true;
            tiles[x][LEVEL_2 + 1].texture_id = 3;
        }

        for x in 50..70 {
            tiles[x][LEVEL_3].solid = true;
            tiles[x][LEVEL_3].texture_id = 4;
            tiles[x][LEVEL_3 + 1].solid = true;
            tiles[x][LEVEL_3 + 1].texture_id = 4;
        }

        for x in 10..22 {
            tiles[x][LEVEL_4].solid = true;
            tiles[x][LEVEL_4].texture_id = 5;
            tiles[x][LEVEL_4 + 1].solid = true;
            tiles[x][LEVEL_4 + 1].texture_id = 5;
        }

        for x in 98..110 {
            tiles[x][LEVEL_4].solid = true;
            tiles[x][LEVEL_4].texture_id = 5;
            tiles[x][LEVEL_4 + 1].solid = true;
            tiles[x][LEVEL_4 + 1].texture_id = 5;
        }

        for x in 54..66 {
            tiles[x][LEVEL_5].solid = true;
            tiles[x][LEVEL_5].texture_id = 6;
            tiles[x][LEVEL_5 + 1].solid = true;
            tiles[x][LEVEL_5 + 1].texture_id = 6;
        }

        for x in 4..8 {
            for y in (FLOOR - 9)..(FLOOR) {
                tiles[x][y].solid = true;
                tiles[x][y].texture_id = 1;
            }
        }

        for x in 112..116 {
            for y in (FLOOR - 9)..(FLOOR) {
                tiles[x][y].solid = true;
                tiles[x][y].texture_id = 1;
            }
        }

        Self {
            width,
            height,
            tiles,
            spawn_points: vec![
                SpawnPoint {
                    x: 512.0,
                    y: 728.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 3328.0,
                    y: 728.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 512.0,
                    y: 424.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 3328.0,
                    y: 424.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 1216.0,
                    y: 632.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 2624.0,
                    y: 632.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 1920.0,
                    y: 536.0,
                    team: 0,
                },
                SpawnPoint {
                    x: 1920.0,
                    y: 312.0,
                    team: 0,
                },
            ],
            items: vec![
                Item {
                    x: 1920.0,
                    y: 520.0,
                    item_type: ItemType::Quad,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1920.0,
                    y: 296.0,
                    item_type: ItemType::Railgun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 512.0,
                    y: 680.0,
                    item_type: ItemType::RocketLauncher,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 3328.0,
                    y: 680.0,
                    item_type: ItemType::RocketLauncher,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1200.0,
                    y: 616.0,
                    item_type: ItemType::Plasmagun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 2640.0,
                    y: 616.0,
                    item_type: ItemType::Plasmagun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 288.0,
                    y: 680.0,
                    item_type: ItemType::Armor100,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 3552.0,
                    y: 680.0,
                    item_type: ItemType::Armor100,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1920.0,
                    y: 616.0,
                    item_type: ItemType::Health100,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 416.0,
                    y: 408.0,
                    item_type: ItemType::Health50,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 3424.0,
                    y: 408.0,
                    item_type: ItemType::Health50,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1792.0,
                    y: 520.0,
                    item_type: ItemType::Armor50,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 2048.0,
                    y: 520.0,
                    item_type: ItemType::Armor50,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1120.0,
                    y: 616.0,
                    item_type: ItemType::Shotgun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 2720.0,
                    y: 616.0,
                    item_type: ItemType::Shotgun,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1280.0,
                    y: 616.0,
                    item_type: ItemType::GrenadeLauncher,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 2560.0,
                    y: 616.0,
                    item_type: ItemType::GrenadeLauncher,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 448.0,
                    y: 680.0,
                    item_type: ItemType::Health25,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 3392.0,
                    y: 680.0,
                    item_type: ItemType::Health25,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 1360.0,
                    y: 616.0,
                    item_type: ItemType::Health25,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
                Item {
                    x: 2480.0,
                    y: 616.0,
                    item_type: ItemType::Health25,
                    respawn_time: 0,
                    active: true,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    dropped: false,
                    yaw: 0.0,
                    spin_yaw: 0.0,
                    pitch: 0.0,
                    roll: 0.0,
                    spin_pitch: 0.0,
                    spin_roll: 0.0,
                },
            ],
            jumppads: vec![
                JumpPad {
                    x: 128.0,
                    y: 752.0,
                    width: 64.0,
                    force_x: 0.0,
                    force_y: -6.5,
                    cooldown: 0,
                },
                JumpPad {
                    x: 3648.0,
                    y: 752.0,
                    width: 64.0,
                    force_x: 0.0,
                    force_y: -6.5,
                    cooldown: 0,
                },
                JumpPad {
                    x: 960.0,
                    y: 640.0,
                    width: 64.0,
                    force_x: 4.0,
                    force_y: -5.5,
                    cooldown: 0,
                },
                JumpPad {
                    x: 2816.0,
                    y: 640.0,
                    width: 64.0,
                    force_x: -4.0,
                    force_y: -5.5,
                    cooldown: 0,
                },
                JumpPad {
                    x: 1664.0,
                    y: 640.0,
                    width: 64.0,
                    force_x: 0.0,
                    force_y: -4.5,
                    cooldown: 0,
                },
                JumpPad {
                    x: 2112.0,
                    y: 640.0,
                    width: 64.0,
                    force_x: 0.0,
                    force_y: -4.5,
                    cooldown: 0,
                },
            ],
            teleporters: vec![
                Teleporter {
                    x: 1536.0,
                    y: 608.0,
                    width: 64.0,
                    height: 64.0,
                    dest_x: 1840.0,
                    dest_y: 296.0,
                },
                Teleporter {
                    x: 2240.0,
                    y: 608.0,
                    width: 64.0,
                    height: 64.0,
                    dest_x: 2000.0,
                    dest_y: 296.0,
                },
            ],
            lights: vec![],
            background_elements: vec![],
        }
    }

    #[inline]
    pub fn is_solid(&self, tile_x: i32, tile_y: i32) -> bool {
        if tile_x < 0 || tile_y < 0 || tile_x >= self.width as i32 || tile_y >= self.height as i32 {
            return true;
        }
        self.tiles[tile_x as usize][tile_y as usize].solid
    }

    pub fn map_width(&self) -> usize {
        self.width
    }

    pub fn map_height(&self) -> usize {
        self.height
    }

    pub fn has_line_of_sight(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 1.0 {
            return true;
        }

        let steps = (dist / 8.0).ceil() as i32;
        let step_x = dx / steps as f32;
        let step_y = dy / steps as f32;

        for i in 0..=steps {
            let check_x = x1 + step_x * i as f32;
            let check_y = y1 + step_y * i as f32;

            let tile_x = (check_x / 32.0) as i32;
            let tile_y = (check_y / 16.0) as i32;

            if self.is_solid(tile_x, tile_y) {
                return false;
            }
        }

        true
    }

    pub fn get_floor_below(&self, x: f32, start_y: f32) -> f32 {
        let tile_x = (x / 32.0) as i32;
        if tile_x < 0 || tile_x >= self.width as i32 {
            return 752.0;
        }
        let mut tile_y = (start_y / 16.0) as i32;
        if tile_y < 0 {
            tile_y = 0;
        }
        for y in tile_y..self.height as i32 {
            if self.is_solid(tile_x, y) {
                return (y * 16) as f32 - 16.0;
            }
        }
        752.0
    }

    pub fn find_safe_spawn_position(&self) -> (f32, f32) {
        if !self.spawn_points.is_empty() {
            let sp = &self.spawn_points[0];
            return (sp.x, sp.y);
        }
        (1920.0, 728.0)
    }

    pub fn render(&self, camera_x: f32, camera_y: f32, _lights: &[super::light::LightPulse]) {
        self.render_background(camera_x, camera_y);
        self.render_tiles(camera_x, camera_y);
    }

    pub fn render_background(&self, _camera_x: f32, _camera_y: f32) {
        clear_background(Color::from_rgba(18, 22, 28, 255));
    }

    pub fn render_tiles(&self, camera_x: f32, camera_y: f32) {
        let _w = screen_width();
        let _h = screen_height();

        let start_x = ((camera_x / 32.0).floor() as i32).max(0);
        let end_x = (((camera_x + screen_width()) / 32.0).ceil() as i32).min(self.width as i32);
        let start_y = ((camera_y / 16.0).floor() as i32).max(0);
        let end_y = (((camera_y + screen_height()) / 16.0).ceil() as i32).min(self.height as i32);

        for x in start_x..end_x {
            for y in start_y..end_y {
                let tile = &self.tiles[x as usize][y as usize];
                let screen_x = (x as f32 * 32.0) - camera_x;
                let screen_y = (y as f32 * 16.0) - camera_y;

                if tile.solid {
                    let left_tex = if x > 0 && self.tiles[(x - 1) as usize][y as usize].solid {
                        Some(self.tiles[(x - 1) as usize][y as usize].texture_id)
                    } else {
                        None
                    };
                    let right_tex = if x < (self.width as i32 - 1)
                        && self.tiles[(x + 1) as usize][y as usize].solid
                    {
                        Some(self.tiles[(x + 1) as usize][y as usize].texture_id)
                    } else {
                        None
                    };
                    let top_tex = if y > 0 && self.tiles[x as usize][(y - 1) as usize].solid {
                        Some(self.tiles[x as usize][(y - 1) as usize].texture_id)
                    } else {
                        None
                    };
                    let bottom_tex = if y < (self.height as i32 - 1)
                        && self.tiles[x as usize][(y + 1) as usize].solid
                    {
                        Some(self.tiles[x as usize][(y + 1) as usize].texture_id)
                    } else {
                        None
                    };

                    let left_same = left_tex.map_or(false, |t| t == tile.texture_id);
                    let right_same = right_tex.map_or(false, |t| t == tile.texture_id);
                    let top_same = top_tex.map_or(false, |t| t == tile.texture_id);
                    let bottom_same = bottom_tex.map_or(false, |t| t == tile.texture_id);

                    let world_tile_x = x as f32 * 32.0;
                    let world_tile_y = y as f32 * 16.0;

                    super::procedural_tiles::render_procedural_tile_simple(
                        screen_x,
                        screen_y,
                        32.0,
                        16.0,
                        tile.texture_id,
                        (left_same, right_same, top_same, bottom_same),
                        world_tile_x,
                        world_tile_y,
                    );
                }
            }
        }

        let time = crate::time::get_time() as f32;

        for teleporter in &self.teleporters {
            let screen_x = teleporter.x - camera_x;
            let screen_y = teleporter.y - camera_y;

            let time = crate::time::get_time() as f32;
            let left = screen_x;
            let right = screen_x + teleporter.width;

            // Side emitters (yellow/orange frames)
            let side_w = 8.0;
            draw_rectangle(
                left - 6.0,
                screen_y,
                side_w,
                teleporter.height,
                Color::from_rgba(210, 170, 60, 220),
            );
            draw_rectangle(
                right - side_w + 6.0,
                screen_y,
                side_w,
                teleporter.height,
                Color::from_rgba(210, 170, 60, 220),
            );
            for i in 0..5 {
                let a = 200 - i * 28;
                draw_rectangle(
                    left - 6.0 + i as f32,
                    screen_y,
                    1.0,
                    teleporter.height,
                    Color::from_rgba(255, 220, 100, a as u8),
                );
                draw_rectangle(
                    right + 6.0 - i as f32,
                    screen_y,
                    1.0,
                    teleporter.height,
                    Color::from_rgba(255, 220, 100, a as u8),
                );
            }

            // Inner portal body
            draw_rectangle(
                left + 6.0,
                screen_y,
                teleporter.width - 12.0,
                teleporter.height,
                Color::from_rgba(24, 48, 64, 230),
            );

            // Central horizontal bars moving upward
            let bar_gap = 10.0;
            let offset = (time * 40.0) % bar_gap;
            let inner_left = left + 12.0;
            let inner_right = right - 12.0;
            let bar_width = inner_right - inner_left;
            let mut y = screen_y + teleporter.height - offset;
            while y >= screen_y {
                draw_rectangle(
                    inner_left,
                    y - 2.0,
                    bar_width,
                    4.0,
                    Color::from_rgba(220, 230, 240, 210),
                );
                draw_rectangle(
                    inner_left,
                    y - 1.0,
                    bar_width,
                    2.0,
                    Color::from_rgba(255, 255, 255, 230),
                );
                y -= bar_gap;
            }

            // Core glow
            let glow = (time * 2.2).sin() * 0.25 + 0.75;
            draw_rectangle(
                inner_left,
                screen_y,
                bar_width,
                teleporter.height,
                Color::from_rgba(100, 150, 220, (25.0 * glow) as u8),
            );
        }

        for pad in &self.jumppads {
            let screen_x = pad.x - camera_x;
            let screen_y = pad.y - camera_y;

            let pulse = (time * 5.0).sin() * 0.25 + 0.75;
            let alpha = (pulse * 200.0) as u8;

            draw_rectangle(
                screen_x,
                screen_y,
                pad.width,
                12.0,
                Color::from_rgba(255, 180, 60, alpha),
            );

            draw_rectangle(
                screen_x + 4.0,
                screen_y + 2.0,
                pad.width - 8.0,
                8.0,
                Color::from_rgba(255, 200, 100, 220),
            );

            for i in 0..3 {
                let arrow_offset = ((time * 3.0 - i as f32 * 0.4).fract()) * 8.0;
                let arrow_alpha = ((1.0 - arrow_offset / 8.0) * 255.0) as u8;

                let cx = screen_x + pad.width * 0.5;
                let cy = screen_y + 6.0 - arrow_offset;

                draw_line(
                    cx - 4.0,
                    cy,
                    cx,
                    cy - 4.0,
                    2.0,
                    Color::from_rgba(255, 255, 100, arrow_alpha),
                );
                draw_line(
                    cx + 4.0,
                    cy,
                    cx,
                    cy - 4.0,
                    2.0,
                    Color::from_rgba(255, 255, 100, arrow_alpha),
                );
            }
        }
    }
}
