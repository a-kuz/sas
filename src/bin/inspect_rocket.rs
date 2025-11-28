use sas::game::md3::MD3Model;

fn main() {
    let model = MD3Model::load("q3-resources/models/ammo/rocket/rocket.md3").unwrap();
    
    println!("=== Rocket Model Info ===");
    println!("Number of meshes: {}", model.meshes.len());
    println!("Number of tag frames: {}", model.tags.len());
    println!("Number of bone frames: {}", model.header.num_bone_frames);
    
    if !model.tags.is_empty() {
        println!("\nTags in frame 0:");
        for tag in &model.tags[0] {
            let name = String::from_utf8_lossy(&tag.name).trim_end_matches('\0').to_string();
            if !name.is_empty() {
                println!("  Tag '{}': pos=({:.2}, {:.2}, {:.2})", 
                         name, tag.position[0], tag.position[1], tag.position[2]);
            }
        }
    }
    
    println!("\nMeshes:");
    for (i, mesh) in model.meshes.iter().enumerate() {
        let name = String::from_utf8_lossy(&mesh.header.name).trim_end_matches('\0').to_string();
        println!("  Mesh {}: '{}' - {} triangles, {} vertices", 
                 i, name, mesh.triangles.len(), 
                 if mesh.vertices.is_empty() { 0 } else { mesh.vertices[0].len() });
        
        if !mesh.vertices.is_empty() && !mesh.vertices[0].is_empty() {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;
            
            for v in &mesh.vertices[0] {
                let x = v.vertex[0] as f32;
                let y = v.vertex[1] as f32;
                let z = v.vertex[2] as f32;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                min_z = min_z.min(z);
                max_z = max_z.max(z);
            }
            
            println!("    Bounds: X[{:.1}, {:.1}] Y[{:.1}, {:.1}] Z[{:.1}, {:.1}]", 
                     min_x, max_x, min_y, max_y, min_z, max_z);
            println!("    Center: ({:.1}, {:.1}, {:.1})", 
                     (min_x + max_x) / 2.0, (min_y + max_y) / 2.0, (min_z + max_z) / 2.0);
        }
    }
}

