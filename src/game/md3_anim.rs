use std::fs;

#[derive(Clone, Debug)]
pub struct AnimRange {
    pub first_frame: usize,
    pub num_frames: usize,
    pub looping_frames: usize,
    pub fps: usize,
}

#[derive(Clone, Debug)]
pub struct AnimConfig {
    pub death1: AnimRange,
    pub torso_attack: AnimRange,
    pub torso_stand: AnimRange,
    pub legs_run: AnimRange,
    pub legs_jump: AnimRange,
    pub legs_idle: AnimRange,
    pub legs_walkcr: AnimRange,
    pub legs_idlecr: AnimRange,
}

impl AnimConfig {
    pub fn load(model_name: &str) -> Result<Self, String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = format!("q3-resources/models/players/{}/animation.cfg", model_name);
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read animation.cfg: {}", e))?;
            Self::parse_content(&content)
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            Err("Use load_async for WASM".to_string())
        }
    }
    
    pub async fn load_async(model_name: &str) -> Result<Self, String> {
        let path = format!("q3-resources/models/players/{}/animation.cfg", model_name);
        let content = super::file_loader::load_file_string(&path).await?;
        Self::parse_content(&content)
    }
    
    fn parse_content(content: &str) -> Result<Self, String> {
        let mut anims: Vec<AnimRange> = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with("sex") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(first), Ok(num), Ok(loop_frames), Ok(fps)) = (
                    parts[0].parse::<usize>(),
                    parts[1].parse::<usize>(),
                    parts[2].parse::<usize>(),
                    parts[3].parse::<usize>(),
                ) {
                    anims.push(AnimRange {
                        first_frame: first,
                        num_frames: num,
                        looping_frames: loop_frames,
                        fps,
                    });
                }
            }
        }
        
        if anims.len() < 22 {
            return Err("Not enough animation ranges in config".to_string());
        }
        if anims.len() >= 25 {
            let torso_gesture_first = anims[6].first_frame;
            let legs_walkcr_first = anims[13].first_frame;
            let skip = legs_walkcr_first.saturating_sub(torso_gesture_first);
            for i in 13..25 {
                anims[i].first_frame = anims[i].first_frame.saturating_sub(skip);
            }
        }
        
        Ok(AnimConfig {
            death1: anims[0].clone(),
            torso_attack: anims[7].clone(),
            torso_stand: anims[11].clone(),
            legs_walkcr: anims[13].clone(),
            legs_run: anims[15].clone(),
            legs_jump: anims[18].clone(),
            legs_idle: anims[22].clone(),
            legs_idlecr: anims[23].clone(),
        })
    }
}

