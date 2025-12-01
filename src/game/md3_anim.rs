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
    pub both_death1: AnimRange,
    pub both_dead1: AnimRange,
    pub both_death2: AnimRange,
    pub both_dead2: AnimRange,
    pub both_death3: AnimRange,
    pub both_dead3: AnimRange,
    pub torso_gesture: AnimRange,
    pub torso_attack: AnimRange,
    pub torso_attack2: AnimRange,
    pub torso_drop: AnimRange,
    pub torso_raise: AnimRange,
    pub torso_stand: AnimRange,
    pub torso_stand2: AnimRange,
    pub legs_walkcr: AnimRange,
    pub legs_walk: AnimRange,
    pub legs_run: AnimRange,
    pub legs_back: AnimRange,
    pub legs_swim: AnimRange,
    pub legs_jump: AnimRange,
    pub legs_land: AnimRange,
    pub legs_jumpb: AnimRange,
    pub legs_landb: AnimRange,
    pub legs_idle: AnimRange,
    pub legs_idlecr: AnimRange,
    pub legs_turn: AnimRange,
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
                    let anim_range = AnimRange {
                        first_frame: first,
                        num_frames: num,
                        looping_frames: loop_frames,
                        fps,
                    };
                    anims.push(anim_range);
                }
            }
        }

        if anims.len() < 13 {
            return Err(format!("Not enough animation ranges in config: got {}, need at least 13", anims.len()));
        }

        let skip = if anims.len() > 13 {
            anims[13].first_frame.saturating_sub(anims[6].first_frame)
        } else {
            0
        };

        for i in 13..anims.len() {
            anims[i].first_frame = anims[i].first_frame.saturating_sub(skip);
        }

        println!("[MD3_ANIM] Loaded {} animations, skip={}", anims.len(), skip);

        Ok(AnimConfig {
            both_death1: anims[0].clone(),
            both_dead1: anims[1].clone(),
            both_death2: anims[2].clone(),
            both_dead2: anims[3].clone(),
            both_death3: anims[4].clone(),
            both_dead3: anims[5].clone(),
            torso_gesture: anims[6].clone(),
            torso_attack: anims[7].clone(),
            torso_attack2: anims[8].clone(),
            torso_drop: anims[9].clone(),
            torso_raise: anims[10].clone(),
            torso_stand: anims[11].clone(),
            torso_stand2: anims[12].clone(),
            legs_walkcr: anims[13].clone(),
            legs_walk: anims[14].clone(),
            legs_run: anims[15].clone(),
            legs_back: anims[16].clone(),
            legs_swim: anims[17].clone(),
            legs_jump: anims[18].clone(),
            legs_land: anims[19].clone(),
            legs_jumpb: anims[20].clone(),
            legs_landb: anims[21].clone(),
            legs_idle: anims[22].clone(),
            legs_idlecr: anims[23].clone(),
            legs_turn: if anims.len() > 24 { anims[24].clone() } else { anims[22].clone() },
        })
    }
}
