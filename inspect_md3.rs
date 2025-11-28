use std::fs::File;
use std::io::Read;

#[repr(C)]
#[derive(Debug)]
struct MD3Header {
    id: [u8; 4],
    version: i32,
    filename: [u8; 68],
    num_bone_frames: i32,
    num_tags: i32,
    num_meshes: i32,
    num_max_skins: i32,
    header_length: i32,
    tag_start: i32,
    tag_end: i32,
    file_size: i32,
}

fn main() {
    let paths = vec![
        "q3-resources/models/weapons2/machinegun/machinegun.md3",
        "q3-resources/models/weapons2/machinegun/machinegun_barrel.md3",
    ];
    
    for path in paths {
        println!("\n=== {} ===", path);
        let mut file = File::open(path).expect("Failed to open file");
        
        let mut header_bytes = [0u8; 108];
        file.read_exact(&mut header_bytes).expect("Failed to read header");
        
        let header = unsafe { std::ptr::read(header_bytes.as_ptr() as *const MD3Header) };
        
        println!("ID: {:?}", std::str::from_utf8(&header.id).unwrap());
        println!("Version: {}", header.version);
        println!("Filename: {:?}", std::str::from_utf8(&header.filename).unwrap_or("invalid"));
        println!("Bone frames: {}", header.num_bone_frames);
        println!("Tags: {}", header.num_tags);
        println!("Meshes: {}", header.num_meshes);
        
        for _ in 0..header.num_bone_frames {
            let mut frame_bytes = [0u8; 56];
            file.read_exact(&mut frame_bytes).expect("Failed to read bone frame");
        }
        
        for frame_idx in 0..header.num_bone_frames {
            println!("\n  Frame {}:", frame_idx);
            for tag_idx in 0..header.num_tags {
                let mut tag_bytes = [0u8; 112];
                file.read_exact(&mut tag_bytes).expect("Failed to read tag");
                
                let mut name = [0u8; 64];
                name.copy_from_slice(&tag_bytes[0..64]);
                let tag_name = std::str::from_utf8(&name)
                    .unwrap_or("invalid")
                    .trim_end_matches('\0');
                
                let mut position = [0f32; 3];
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        tag_bytes[64..76].as_ptr(),
                        position.as_mut_ptr() as *mut u8,
                        12
                    );
                }
                
                let mut rotation = [[0f32; 3]; 3];
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        tag_bytes[76..112].as_ptr(),
                        rotation.as_mut_ptr() as *mut u8,
                        36
                    );
                }
                
                println!("    Tag {}: '{}'", tag_idx, tag_name);
                println!("      Position: [{:.3}, {:.3}, {:.3}]", position[0], position[1], position[2]);
                println!("      Rotation matrix:");
                println!("        [{:.3}, {:.3}, {:.3}]", rotation[0][0], rotation[0][1], rotation[0][2]);
                println!("        [{:.3}, {:.3}, {:.3}]", rotation[1][0], rotation[1][1], rotation[1][2]);
                println!("        [{:.3}, {:.3}, {:.3}]", rotation[2][0], rotation[2][1], rotation[2][2]);
            }
        }
        
        for mesh_idx in 0..header.num_meshes {
            let mut mesh_header_bytes = [0u8; 108];
            file.read_exact(&mut mesh_header_bytes).expect("Failed to read mesh header");
            
            let mut name = [0u8; 68];
            name.copy_from_slice(&mesh_header_bytes[4..72]);
            let mesh_name = std::str::from_utf8(&name)
                .unwrap_or("invalid")
                .trim_end_matches('\0');
            
            let num_vertices = i32::from_le_bytes([
                mesh_header_bytes[80], mesh_header_bytes[81], 
                mesh_header_bytes[82], mesh_header_bytes[83]
            ]);
            let num_triangles = i32::from_le_bytes([
                mesh_header_bytes[84], mesh_header_bytes[85], 
                mesh_header_bytes[86], mesh_header_bytes[87]
            ]);
            
            println!("\n  Mesh {}: '{}'", mesh_idx, mesh_name);
            println!("    Vertices: {}", num_vertices);
            println!("    Triangles: {}", num_triangles);
            
            break;
        }
    }
}
