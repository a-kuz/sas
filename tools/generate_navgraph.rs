use std::collections::HashMap;
use std::fs;

#[path = "../src/game/light.rs"]
mod light;

#[path = "../src/game/procedural_tiles.rs"]
mod procedural_tiles;

#[path = "../src/game/shader.rs"]
mod shader;

#[path = "../src/game/map.rs"]
mod map;

#[path = "../src/game/file_loader.rs"]
mod file_loader;

#[path = "../src/game/map_loader.rs"]
mod map_loader;

#[path = "../src/game/nav_graph.rs"]
mod nav_graph;

#[path = "../src/game/nav_graph_generator.rs"]
mod nav_graph_generator;

use nav_graph_generator::NavGraphGenerator;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: generate_navgraph <map_name>");
        eprintln!("Example: generate_navgraph 0-arena");
        eprintln!("         generate_navgraph new_map");
        std::process::exit(1);
    }

    let map_name = &args[1];
    let map_path = format!("maps/{}.json", map_name);

    println!("Loading map: {}", map_path);
    let map = match map::Map::load_from_file(map_name) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load map: {}", e);
            std::process::exit(1);
        }
    };

    println!("Map dimensions: {}x{}", map.width, map.height);
    println!("Jump pads: {}", map.jumppads.len());
    println!("Teleporters: {}", map.teleporters.len());
    println!("Items: {}", map.items.len());
    println!();

    println!("Generating navigation graph...");
    let generator = NavGraphGenerator::new(map);
    let nav_graph = generator.generate();

    println!();
    println!("=== NAVIGATION GRAPH STATISTICS ===");
    println!("Total nodes: {}", nav_graph.nodes.len());
    println!("Total edges: {}", nav_graph.edges.len());
    println!();

    let node_types = nav_graph
        .nodes
        .iter()
        .fold(HashMap::new(), |mut acc, node| {
            *acc.entry(format!("{:?}", node.node_type)).or_insert(0) += 1;
            acc
        });

    println!("Node types:");
    for (node_type, count) in node_types {
        println!("  {}: {}", node_type, count);
    }
    println!();

    let edge_types = nav_graph
        .edges
        .iter()
        .fold(HashMap::new(), |mut acc, edge| {
            *acc.entry(format!("{:?}", edge.edge_type)).or_insert(0) += 1;
            acc
        });

    println!("Edge types:");
    for (edge_type, count) in edge_types {
        println!("  {}: {}", edge_type, count);
    }
    println!();

    let components = nav_graph.find_connected_components();
    println!("Connected components: {}", components.len());
    if components.len() > 1 {
        println!(
            "WARNING: Graph has {} disconnected components!",
            components.len()
        );
        for (i, component) in components.iter().enumerate() {
            println!("  Component {}: {} nodes", i, component.len());
        }
    } else {
        println!("✓ Graph is fully connected!");
    }
    println!();

    let output_path = format!("maps/{}_navgraph.json", map_name);
    let json = serde_json::to_string_pretty(&nav_graph).unwrap();

    fs::write(&output_path, json).expect("Failed to write navigation graph");
    println!("Saved navigation graph to: {}", output_path);
}
