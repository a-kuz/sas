#![allow(dead_code)]

use std::fs;

#[path = "../src/game/light.rs"]
mod light;

#[path = "../src/game/map.rs"]
mod map;

#[path = "../src/game/map_loader.rs"]
mod map_loader;

#[path = "../src/game/procedural_tiles.rs"]
mod procedural_tiles;

fn main() {
    fs::create_dir_all("maps").expect("Failed to create maps directory");
    
    println!("Exporting q3dm6...");
    let q3dm6 = map::Map::q3dm6();
    let map_file = map_loader::MapFile::from_map(&q3dm6, 32.0, 16.0, "q3dm6");
    map_file.save_to_file("maps/q3dm6.json").expect("Failed to save q3dm6");
    println!("✓ Exported q3dm6 to maps/q3dm6.json");
    
    println!("Exporting soldat...");
    let soldat = map::Map::soldat_map();
    let map_file = map_loader::MapFile::from_map(&soldat, 32.0, 16.0, "soldat");
    map_file.save_to_file("maps/soldat.json").expect("Failed to save soldat");
    println!("✓ Exported soldat to maps/soldat.json");
    
    println!("\nDone! Maps exported successfully.");
}

