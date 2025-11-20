use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use super::map::{Map, Tile, SpawnPoint, Item, ItemType, JumpPad, Teleporter, LightSource, BackgroundElement};
use super::file_loader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapFile {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub tile_width: f32,
    pub tile_height: f32,
    pub tile_data: Vec<TileRow>,
    pub spawn_points: Vec<SpawnPointData>,
    pub items: Vec<ItemData>,
    pub jumppads: Vec<JumpPadData>,
    pub teleporters: Vec<TeleporterData>,
    pub lights: Vec<LightData>,
    #[serde(default)]
    pub background_elements: Option<Vec<BackgroundElement>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileRow {
    pub y: usize,
    pub tiles: Vec<TileData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileData {
    pub x_start: usize,
    pub x_end: usize,
    pub solid: bool,
    pub texture_id: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnPointData {
    pub tile_x: f32,
    pub tile_y: f32,
    pub team: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemData {
    pub tile_x: f32,
    pub tile_y: f32,
    pub item_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JumpPadData {
    pub tile_x: f32,
    pub tile_y: f32,
    pub width_tiles: f32,
    pub force_x: f32,
    pub force_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeleporterData {
    pub tile_x: f32,
    pub tile_y: f32,
    pub width_tiles: f32,
    pub height_tiles: f32,
    pub dest_tile_x: f32,
    pub dest_tile_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightData {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(default = "default_intensity")]
    pub intensity: f32,
    #[serde(default)]
    pub flicker: bool,
}

fn default_intensity() -> f32 { 1.0 }

impl MapFile {
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let map_file: MapFile = serde_json::from_reader(reader)?;
        Ok(map_file)
    }
    
    pub async fn load_from_file_async(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = file_loader::load_file_string(path).await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, e)) as Box<dyn std::error::Error>)?;
        let map_file: MapFile = serde_json::from_str(&content)?;
        Ok(map_file)
    }

    pub fn to_map(&self) -> Map {
        let mut tiles = vec![vec![Tile { 
            solid: false, 
            texture_id: 0,
            shader_name: None,
            detail_texture: None,
            glow_texture: None,
            blend_alpha: 1.0,
        }; self.height]; self.width];

        for row in &self.tile_data {
            let y = row.y;
            if y >= self.height {
                continue;
            }
            for tile_data in &row.tiles {
                for x in tile_data.x_start..=tile_data.x_end.min(self.width - 1) {
                    tiles[x][y] = Tile {
                        solid: tile_data.solid,
                        texture_id: tile_data.texture_id,
                        shader_name: None,
                        detail_texture: None,
                        glow_texture: None,
                        blend_alpha: 1.0,
                    };
                }
            }
        }

        let spawn_points = self.spawn_points
            .iter()
            .map(|sp| SpawnPoint {
                x: sp.tile_x * self.tile_width,
                y: sp.tile_y * self.tile_height,
                team: sp.team,
            })
            .collect();

        let items = self.items
            .iter()
            .filter_map(|item| {
                let item_type = match item.item_type.as_str() {
                    "Health25" => ItemType::Health25,
                    "Health50" => ItemType::Health50,
                    "Health100" => ItemType::Health100,
                    "Armor50" => ItemType::Armor50,
                    "Armor100" => ItemType::Armor100,
                    "Shotgun" => ItemType::Shotgun,
                    "GrenadeLauncher" => ItemType::GrenadeLauncher,
                    "RocketLauncher" => ItemType::RocketLauncher,
                    "Railgun" => ItemType::Railgun,
                    "Plasmagun" => ItemType::Plasmagun,
                    "BFG" => ItemType::BFG,
                    "Quad" => ItemType::Quad,
                    "Regen" => ItemType::Regen,
                    "Battle" => ItemType::Battle,
                    "Flight" => ItemType::Flight,
                    "Haste" => ItemType::Haste,
                    "Invis" => ItemType::Invis,
                    _ => return None,
                };
                Some(Item {
                    x: item.tile_x * self.tile_width,
                    y: item.tile_y * self.tile_height,
                    item_type,
                    respawn_time: 0,
                    active: true,
                })
            })
            .collect();

        let jumppads = self.jumppads
            .iter()
            .map(|jp| JumpPad {
                x: jp.tile_x * self.tile_width,
                y: jp.tile_y * self.tile_height,
                width: jp.width_tiles * self.tile_width,
                force_x: jp.force_x,
                force_y: jp.force_y,
                cooldown: 0,
            })
            .collect();

        let teleporters = self.teleporters
            .iter()
            .map(|tp| Teleporter {
                x: tp.tile_x * self.tile_width,
                y: tp.tile_y * self.tile_height,
                width: tp.width_tiles * self.tile_width,
                height: tp.height_tiles * self.tile_height,
                dest_x: tp.dest_tile_x * self.tile_width,
                dest_y: tp.dest_tile_y * self.tile_height,
            })
            .collect();

        let lights = self.lights.iter().map(|l| LightSource { 
            x: l.x, 
            y: l.y, 
            radius: l.radius, 
            r: l.r, 
            g: l.g, 
            b: l.b, 
            intensity: l.intensity,
            flicker: l.flicker,
        }).collect();

        Map {
            width: self.width,
            height: self.height,
            tiles,
            spawn_points,
            items,
            jumppads,
            teleporters,
            lights,
            background_elements: self.background_elements.clone().unwrap_or_default(),
        }
    }

    #[allow(dead_code)]
    pub fn from_map(map: &Map, tile_width: f32, tile_height: f32, name: &str) -> Self {
        let mut tile_data = Vec::new();

        for y in 0..map.height {
            let mut row_tiles = Vec::new();
            let mut x = 0;

            while x < map.width {
                let tile = &map.tiles[x][y];
                if tile.solid || tile.texture_id != 0 {
                    let x_start = x;
                    let texture_id = tile.texture_id;
                    let solid = tile.solid;

                    while x < map.width 
                        && map.tiles[x][y].solid == solid 
                        && map.tiles[x][y].texture_id == texture_id {
                        x += 1;
                    }

                    row_tiles.push(TileData {
                        x_start,
                        x_end: x - 1,
                        solid,
                        texture_id,
                    });
                } else {
                    x += 1;
                }
            }

            if !row_tiles.is_empty() {
                tile_data.push(TileRow { y, tiles: row_tiles });
            }
        }

        let spawn_points = map.spawn_points
            .iter()
            .map(|sp| SpawnPointData {
                tile_x: sp.x / tile_width,
                tile_y: sp.y / tile_height,
                team: sp.team,
            })
            .collect();

        let items = map.items
            .iter()
            .map(|item| {
                let item_type = match item.item_type {
                    ItemType::Health25 => "Health25",
                    ItemType::Health50 => "Health50",
                    ItemType::Health100 => "Health100",
                    ItemType::Armor50 => "Armor50",
                    ItemType::Armor100 => "Armor100",
                    ItemType::Shotgun => "Shotgun",
                    ItemType::GrenadeLauncher => "GrenadeLauncher",
                    ItemType::RocketLauncher => "RocketLauncher",
                    ItemType::Railgun => "Railgun",
                    ItemType::Plasmagun => "Plasmagun",
                    ItemType::BFG => "BFG",
                    ItemType::Quad => "Quad",
                    ItemType::Regen => "Regen",
                    ItemType::Battle => "Battle",
                    ItemType::Flight => "Flight",
                    ItemType::Haste => "Haste",
                    ItemType::Invis => "Invis",
                };
                ItemData {
                    tile_x: item.x / tile_width,
                    tile_y: item.y / tile_height,
                    item_type: item_type.to_string(),
                }
            })
            .collect();

        let jumppads = map.jumppads
            .iter()
            .map(|jp| JumpPadData {
                tile_x: jp.x / tile_width,
                tile_y: jp.y / tile_height,
                width_tiles: jp.width / tile_width,
                force_x: jp.force_x,
                force_y: jp.force_y,
            })
            .collect();

        let teleporters = map.teleporters
            .iter()
            .map(|tp| TeleporterData {
                tile_x: tp.x / tile_width,
                tile_y: tp.y / tile_height,
                width_tiles: tp.width / tile_width,
                height_tiles: tp.height / tile_height,
                dest_tile_x: tp.dest_x / tile_width,
                dest_tile_y: tp.dest_y / tile_height,
            })
            .collect();

        let lights = map.lights.iter().map(|l| LightData { 
            x: l.x, 
            y: l.y, 
            radius: l.radius, 
            r: l.r, 
            g: l.g, 
            b: l.b,
            intensity: l.intensity,
            flicker: l.flicker,
        }).collect();

        MapFile {
            name: name.to_string(),
            width: map.width,
            height: map.height,
            tile_width,
            tile_height,
            tile_data,
            spawn_points,
            items,
            jumppads,
            teleporters,
            lights,
            background_elements: Some(map.background_elements.clone()),
        }
    }
}

