use std::f32::consts::PI;

fn main() {
    println!("Verifying Flip Animation Math (Hypothesis 2)");

    // Constants
    let scale = 1.0;
    let screen_x = 100.0;
    let screen_y = 100.0;
    let torso_tag_pos = [10.0, 0.0, 0.0]; // Offset tag to check attachment

    // Simulation parameters
    let angles = [0.0, 45.0, 90.0];
    
    for &angle_deg in &angles {
        let angle = angle_deg * PI / 180.0;
        println!("\nAngle: {} deg", angle_deg);

        for &flip_x in &[false, true] {
            let dir = if flip_x { "LEFT" } else { "RIGHT" };
            let x_mult = if flip_x { -1.0 } else { 1.0 };

            // --- LEGS MATH (Standard MD3) ---
            // effective_legs_roll logic: if flip_x { angle } else { -angle }
            let legs_roll = if flip_x { angle } else { -angle };
            
            // Render Legs Tip (Forward vector [10, 0, 0])
            let v_x = 10.0 * x_mult; 
            let v_z = 0.0;
            
            let cos_r = legs_roll.cos();
            let sin_r = legs_roll.sin();
            
            let legs_rx = v_x * cos_r - v_z * sin_r;
            let legs_rz = v_x * sin_r + v_z * cos_r;
            
            let legs_tip_x = screen_x + legs_rx;
            let legs_tip_y = screen_y - legs_rz;

            // --- ATTACHMENT POINT (Where the tag IS on the legs) ---
            // Tag is at [10, 0, 0] relative to legs origin
            let tag_x = torso_tag_pos[0] * scale * x_mult;
            let tag_z = torso_tag_pos[2] * scale;
            
            let tag_rx = tag_x * cos_r - tag_z * sin_r;
            let tag_rz = tag_x * sin_r + tag_z * cos_r;
            
            let attachment_x = screen_x + tag_rx;
            let attachment_y = screen_y - tag_rz;

            // --- TORSO ORIGIN MATH (My implementation) ---
            // Should match attachment_x/y exactly
            let torso_origin_x = attachment_x;
            let torso_origin_y = attachment_y;

            // --- TORSO ROTATION MATH (Pivot Function) ---
            // Hypothesis: if flip_x { -angle } else { angle }
            let torso_roll = if flip_x { -angle } else { angle };
            
            let t_cos = torso_roll.cos();
            let t_sin = torso_roll.sin();
            
            // Render Torso Tip (Forward vector [10, 0, 0])
            let t_v_x = 10.0 * x_mult;
            let t_v_z = 0.0;
            
            let dx = t_v_x;
            let dy = -t_v_z;
            
            let torso_rx = dx * t_cos - dy * t_sin;
            let torso_ry = dx * t_sin + dy * t_cos;
            
            let torso_tip_x = torso_origin_x + torso_rx;
            let torso_tip_y = torso_origin_y + torso_ry;
            
            println!("  [{}] Legs Tip: ({:.1}, {:.1}) | Torso Tip: ({:.1}, {:.1}) | Attachment Match: YES", 
                dir, legs_tip_x, legs_tip_y, torso_tip_x, torso_tip_y);
                
            // Verify Tip Relative to Origin
            let legs_rel_x = legs_tip_x - screen_x;
            let legs_rel_y = legs_tip_y - screen_y;
            let torso_rel_x = torso_tip_x - torso_origin_x;
            let torso_rel_y = torso_tip_y - torso_origin_y;
            
            if (legs_rel_x - torso_rel_x).abs() > 0.1 || (legs_rel_y - torso_rel_y).abs() > 0.1 {
                 println!("     >>> MISMATCH! Rel Legs: ({:.1}, {:.1}) vs Rel Torso: ({:.1}, {:.1})", 
                    legs_rel_x, legs_rel_y, torso_rel_x, torso_rel_y);
            }
        }
    }
}
