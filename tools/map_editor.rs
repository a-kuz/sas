#![allow(dead_code)]

use macroquad::prelude::*;

#[path = "../src/game/light.rs"]
mod light;

#[path = "../src/game/light_grid.rs"]
mod light_grid;

#[path = "../src/game/lightmap.rs"]
mod lightmap;

#[path = "../src/game/map.rs"]
mod map;

#[path = "../src/game/map_loader.rs"]
mod map_loader;

#[path = "../src/game/file_loader.rs"]
mod file_loader;

#[path = "../src/game/shader.rs"]
mod shader;

#[path = "../src/game/deferred_renderer.rs"]
mod deferred_renderer;

#[path = "../src/game/procedural_tiles.rs"]
mod procedural_tiles;

#[path = "../src/game/nav_graph.rs"]
mod nav_graph;

#[path = "../src/game/nav_graph_generator.rs"]
mod nav_graph_generator;

#[path = "map_editor/shaders.rs"]
mod shaders;

#[path = "map_editor/textures.rs"]
mod textures;

#[path = "map_editor/icons.rs"]
mod icons;

#[path = "map_editor/tools.rs"]
mod tools;

#[path = "map_editor/input.rs"]
mod input;

#[path = "map_editor/help.rs"]
mod help;

#[path = "map_editor/ui.rs"]
mod ui;

#[path = "map_editor/state.rs"]
pub mod state;

#[path = "map_editor/map_selector.rs"]
mod map_selector;

use map_selector::MapSelector;
use state::EditorState;

#[cfg(feature = "profiler")]
mod profiler {
    pub fn scope(_name: &str) -> () {
        ()
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "SAS Map Editor".to_string(),
        window_width: 1600,
        window_height: 900,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let map_name = args[1].clone();
        println!("SAS Map Editor");
        println!("Loading map: {}", map_name);

        let mut editor = EditorState::new(&map_name);

        println!("Loading textures...");
        editor.load_textures().await;
        println!("Textures loaded!");

        loop {
            editor.handle_input();
            editor.render();

            next_frame().await;
        }
    } else {
        println!("SAS Map Editor - Map Selector");

        let mut selector = MapSelector::new().await;
        let mut previews_loaded = false;

        loop {
            if is_key_pressed(KeyCode::Escape) {
                break;
            }

            if !previews_loaded {
                selector.ensure_previews_loaded().await;
                previews_loaded = true;
            }

            if let Some(map_name) = selector.handle_input_and_render() {
                println!("Selected map: {}", map_name);

                let mut editor = EditorState::new(&map_name);
                editor.load_textures().await;

                loop {
                    if is_key_pressed(KeyCode::Escape)
                        && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
                    {
                        break;
                    }

                    editor.handle_input();
                    editor.render();

                    next_frame().await;
                }
            }

            next_frame().await;
        }
    }
}
