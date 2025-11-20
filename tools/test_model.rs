use std::env;
use std::path::Path;

fn check_model(model_name: &str, base_path: &str) -> bool {
    println!("\n=== Testing model: {} ===", model_name);
    
    let model_dir = format!("{}/{}", base_path, model_name);
    if !Path::new(&model_dir).exists() {
        println!("✗ Directory not found: {}", model_dir);
        return false;
    }
    println!("✓ Directory exists: {}", model_dir);
    
    let mut parts_ok = true;
    
    for part in &["lower.md3", "upper.md3", "head.md3"] {
        let path = format!("{}/{}", model_dir, part);
        if Path::new(&path).exists() {
            let metadata = std::fs::metadata(&path).unwrap();
            println!("✓ {} ({} bytes)", part, metadata.len());
        } else {
            println!("✗ {} - NOT FOUND", part);
            parts_ok = false;
        }
    }
    
    let anim_cfg = format!("{}/animation.cfg", model_dir);
    if Path::new(&anim_cfg).exists() {
        println!("✓ animation.cfg");
    } else {
        println!("⚠ animation.cfg - NOT FOUND (optional but recommended)");
    }
    
    let skin_files: Vec<_> = std::fs::read_dir(&model_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "skin")
                .unwrap_or(false)
        })
        .collect();
    
    if !skin_files.is_empty() {
        println!("✓ Found {} skin file(s):", skin_files.len());
        for skin in skin_files.iter().take(5) {
            println!("  - {}", skin.file_name().to_string_lossy());
        }
    } else {
        println!("⚠ No .skin files found (textures may not load)");
    }
    
    parts_ok
}

fn list_all_models(base_path: &str) -> Vec<String> {
    let mut models = Vec::new();
    
    if let Ok(dir) = std::fs::read_dir(base_path) {
        for entry in dir.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let lower = format!("{}/{}/lower.md3", base_path, name);
                    let upper = format!("{}/{}/upper.md3", base_path, name);
                    let head = format!("{}/{}/head.md3", base_path, name);
                    if Path::new(&lower).exists()
                        && Path::new(&upper).exists()
                        && Path::new(&head).exists()
                    {
                        models.push(name);
                    }
                }
            }
        }
    }
    
    models.sort();
    models
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let base_path = "q3-resources/models/players";
    
    println!("═══════════════════════════════════════");
    println!("    Player Model Testing Utility");
    println!("═══════════════════════════════════════\n");
    
    if args.len() > 1 && args[1] == "list" {
        println!("Available models:");
        let models = list_all_models(base_path);
        for (i, model) in models.iter().enumerate() {
            println!("{:3}. {}", i + 1, model);
        }
        println!("\nTotal: {} models", models.len());
        return;
    }
    
    if args.len() > 1 {
        let model_name = &args[1];
        let success = check_model(model_name, base_path);
        
        if success {
            println!("\n✓✓✓ Model '{}' is ready to use!", model_name);
            println!("\nTo use this model:");
            println!("  export NFK_PLAYER_MODEL={}", model_name);
            println!("  cargo run");
        } else {
            println!("\n✗✗✗ Model '{}' has issues!", model_name);
            std::process::exit(1);
        }
    } else {
        println!("Usage:");
        println!("  cargo run --bin test_model list");
        println!("  cargo run --bin test_model <model_name>");
        println!("\nExamples:");
        println!("  cargo run --bin test_model sarge");
        println!("  cargo run --bin test_model pm");
        println!("\nChecking all models...\n");
        
        let models = list_all_models(base_path);
        let mut ok_count = 0;
        let mut total = 0;
        
        for model in &models {
            total += 1;
            if check_model(model, base_path) {
                ok_count += 1;
            }
        }
        
        println!("\n═══════════════════════════════════════");
        println!("Summary: {}/{} models OK", ok_count, total);
        println!("═══════════════════════════════════════");
    }
}

