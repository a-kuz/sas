use macroquad::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AwardType {
    Excellent,
    Impressive,
    Humiliation,
    Perfect,
    Accuracy,
}

#[derive(Clone, Debug)]
pub struct Award {
    pub award_type: AwardType,
    pub player_id: u16,
    pub lifetime: f32,
    pub scale: f32,
}

impl Award {
    pub fn new(award_type: AwardType, player_id: u16) -> Self {
        Self {
            award_type,
            player_id,
            lifetime: 0.0,
            scale: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.lifetime += dt;
        
        if self.lifetime < 0.2 {
            self.scale = self.lifetime / 0.2;
        } else if self.lifetime > 2.5 {
            self.scale = ((3.0 - self.lifetime) / 0.5).max(0.0);
        } else {
            self.scale = 1.0;
        }
    }

    pub fn is_expired(&self) -> bool {
        self.lifetime > 3.0
    }
}

pub struct AwardTracker {
    pub last_kill_time: f32,
    pub kill_count_in_window: u32,
    pub last_victim_id: Option<u16>,
}

impl AwardTracker {
    pub fn new() -> Self {
        Self {
            last_kill_time: 0.0,
            kill_count_in_window: 0,
            last_victim_id: None,
        }
    }

    pub fn check_excellent(&mut self, current_time: f32) -> bool {
        let time_diff = current_time - self.last_kill_time;
        
        if time_diff < 2.0 {
            self.kill_count_in_window += 1;
            self.last_kill_time = current_time;
            
            if self.kill_count_in_window >= 2 {
                self.kill_count_in_window = 1;
                return true;
            }
        } else {
            self.kill_count_in_window = 1;
            self.last_kill_time = current_time;
        }
        
        false
    }

    pub fn reset(&mut self) {
        self.kill_count_in_window = 0;
        self.last_kill_time = 0.0;
        self.last_victim_id = None;
    }
}

pub struct AwardIconCache {
    pub excellent: Option<Texture2D>,
    pub impressive: Option<Texture2D>,
    pub humiliation: Option<Texture2D>,
    pub perfect: Option<Texture2D>,
    pub accuracy: Option<Texture2D>,
}

impl AwardIconCache {
    pub fn new() -> Self {
        Self {
            excellent: None,
            impressive: None,
            humiliation: None,
            perfect: None,
            accuracy: None,
        }
    }

    pub async fn load(&mut self) {
        self.excellent = load_texture("q3-resources/menu/medals/medal_excellent.png").await.ok();
        self.impressive = load_texture("q3-resources/menu/medals/medal_impressive.png").await.ok();
        self.humiliation = load_texture("q3-resources/menu/medals/medal_gauntlet.png").await.ok();
        self.perfect = load_texture("q3-resources/menu/medals/medal_victory.png").await.ok();
        self.accuracy = load_texture("q3-resources/menu/medals/medal_accuracy.png").await.ok();
    }

    pub fn get(&self, award_type: &AwardType) -> Option<&Texture2D> {
        match award_type {
            AwardType::Excellent => self.excellent.as_ref(),
            AwardType::Impressive => self.impressive.as_ref(),
            AwardType::Humiliation => self.humiliation.as_ref(),
            AwardType::Perfect => self.perfect.as_ref(),
            AwardType::Accuracy => self.accuracy.as_ref(),
        }
    }
}

pub struct TimeAnnouncement {
    pub time: f32,
    pub announced_5min: bool,
    pub announced_1min: bool,
    pub announced_fight: bool,
}

impl TimeAnnouncement {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            announced_5min: false,
            announced_1min: false,
            announced_fight: false,
        }
    }

    pub fn update(&mut self, match_time: f32, time_limit: f32) -> Option<&'static str> {
        let remaining = time_limit - match_time;
        
        if !self.announced_fight && match_time > 0.1 && match_time < 1.0 {
            self.announced_fight = true;
            return Some("fight");
        }
        
        if !self.announced_5min && remaining <= 300.0 && remaining > 299.0 {
            self.announced_5min = true;
            return Some("5_minute");
        }
        
        if !self.announced_1min && remaining <= 60.0 && remaining > 59.0 {
            self.announced_1min = true;
            return Some("1_minute");
        }
        
        None
    }

    pub fn reset(&mut self) {
        self.announced_5min = false;
        self.announced_1min = false;
        self.announced_fight = false;
    }
}

pub struct LeadAnnouncement {
    pub current_leader: Option<u16>,
}

impl LeadAnnouncement {
    pub fn new() -> Self {
        Self {
            current_leader: None,
        }
    }

    pub fn update(&mut self, players: &[super::player::Player], local_player_id: Option<u16>, killer_id: Option<u16>, old_leader: Option<u16>, was_local_in_lead_group: bool) -> Option<&'static str> {
        if players.is_empty() || local_player_id.is_none() {
            return None;
        }

        let local_id = local_player_id.unwrap();

        let mut sorted_players: Vec<_> = players.iter().collect();
        sorted_players.sort_by(|a, b| b.frags.cmp(&a.frags));

        let top_score = sorted_players[0].frags;
        let tied_count = sorted_players.iter().filter(|p| p.frags == top_score).count();

        let new_leader = if tied_count > 1 {
            None
        } else {
            Some(sorted_players[0].id)
        };

        let local_score = players.iter().find(|p| p.id == local_id).map(|p| p.frags).unwrap_or(0);
        
        let was_local_sole_leader = old_leader == Some(local_id);
        let is_local_leader = new_leader == Some(local_id);
        let is_local_tied_for_lead = tied_count > 1 && local_score == top_score;
        let is_local_in_lead_group = is_local_leader || is_local_tied_for_lead;

        println!("    LeadAnnouncement::update: local_id={}, killer_id={:?}", local_id, killer_id);
        println!("      old_leader={:?}, new_leader={:?}", old_leader, new_leader);
        println!("      was_local_in_lead_group={}, is_local_in_lead_group={}", was_local_in_lead_group, is_local_in_lead_group);
        println!("      was_local_sole_leader={}, is_local_leader={}, is_local_tied_for_lead={}", was_local_sole_leader, is_local_leader, is_local_tied_for_lead);
        println!("      top_score={}, tied_count={}, local_score={}", top_score, tied_count, local_score);

        let announcement = if let Some(killer) = killer_id {
            if killer == local_id {
                if !was_local_in_lead_group && is_local_leader {
                    println!("      -> Case: local killer became sole leader from behind");
                    Some("taken_the_lead")
                } else if !was_local_in_lead_group && is_local_tied_for_lead {
                    println!("      -> Case: local killer tied for lead");
                    Some("tied_for_the_lead")
                } else if was_local_in_lead_group && is_local_leader && !was_local_sole_leader {
                    println!("      -> Case: local killer became sole leader from tied");
                    Some("taken_the_lead")
                } else if was_local_in_lead_group && !is_local_in_lead_group {
                    println!("      -> Case: local lost lead group (suicide)");
                    Some("lost_the_lead")
                } else if was_local_sole_leader && is_local_tied_for_lead {
                    println!("      -> Case: local sole leader now tied (suicide)");
                    Some("tied_for_the_lead")
                } else {
                    println!("      -> Case: local killer no announcement");
                    None
                }
            } else {
                if was_local_in_lead_group && !is_local_in_lead_group {
                    println!("      -> Case: local lost lead group");
                    Some("lost_the_lead")
                } else if was_local_sole_leader && is_local_tied_for_lead {
                    println!("      -> Case: local sole leader now tied by other");
                    Some("tied_for_the_lead")
                } else {
                    println!("      -> Case: other killer no announcement");
                    None
                }
            }
        } else {
            None
        };

        self.current_leader = new_leader;

        announcement
    }

    pub fn reset(&mut self) {
        self.current_leader = None;
    }
}

pub struct TeamAdvantageAnnouncement {
    pub current_leading_player: Option<u16>,
    pub previous_leading_player: Option<u16>,
}

impl TeamAdvantageAnnouncement {
    pub fn new() -> Self {
        Self {
            current_leading_player: None,
            previous_leading_player: None,
        }
    }

    pub fn update(&mut self, players: &[super::player::Player]) -> Option<&'static str> {
        let mut sorted_players: Vec<_> = players.iter().filter(|p| !p.dead).collect();
        sorted_players.sort_by(|a, b| b.frags.cmp(&a.frags));

        if sorted_players.len() < 2 {
            return None;
        }

        let top_score = sorted_players[0].frags;
        let tied_count = sorted_players.iter().filter(|p| p.frags == top_score).count();

        let new_leader = if tied_count > 1 {
            None
        } else {
            Some(sorted_players[0].id)
        };

        let announcement = match (self.current_leading_player, new_leader) {
            (None, None) => None,
            (None, Some(_)) => Some("taken_the_lead"),
            (Some(_), None) => Some("tied_for_the_lead"),
            (Some(prev), Some(curr)) if prev != curr => Some("taken_the_lead"),
            _ => None,
        };

        self.previous_leading_player = self.current_leading_player;
        self.current_leading_player = new_leader;

        announcement
    }

    pub fn reset(&mut self) {
        self.current_leading_player = None;
        self.previous_leading_player = None;
    }
}

pub struct GameResults {
    pub show: bool,
    pub start_time: f32,
    pub winner_id: Option<u16>,
    pub winner_weapon: Option<super::weapon::Weapon>,
    pub scores: Vec<(u16, String, i32, u32, u32)>,
    pub player_models: Vec<(u16, String, usize, usize)>,
}

impl GameResults {
    pub fn new() -> Self {
        Self {
            show: false,
            start_time: 0.0,
            winner_id: None,
            winner_weapon: None,
            scores: Vec::new(),
            player_models: Vec::new(),
        }
    }

    pub fn trigger(&mut self, players: &[super::player::Player], current_time: f32) {
        self.show = true;
        self.start_time = current_time;
        
        let mut player_scores: Vec<_> = players
            .iter()
            .map(|p| (p.id, p.name.clone(), p.frags, p.excellent_count, p.impressive_count))
            .collect();
        
        player_scores.sort_by(|a, b| b.2.cmp(&a.2));
        
        self.winner_id = player_scores.first().map(|s| s.0);
        self.winner_weapon = self.winner_id.and_then(|id| {
            players.iter().find(|p| p.id == id).map(|p| p.weapon.clone())
        });
        self.scores = player_scores;
        
        self.player_models = players
            .iter()
            .map(|p| (p.id, p.model.clone(), p.upper_frame, p.lower_frame))
            .collect();
    }

    pub fn draw(&self, current_time: f32, model_cache: &mut super::model_cache::ModelCache, award_icon_cache: &AwardIconCache, weapon_model_cache: &super::weapon_model_cache::WeaponModelCache, _camera_x: f32, _camera_y: f32) {
        if !self.show {
            return;
        }

        let alpha = ((current_time - self.start_time) * 2.0).min(1.0);
        
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.85 * alpha));
        
        let title = "MATCH RESULTS";
        let title_size = 60.0;
        let title_w = crate::render::measure_q3_banner_string(title, title_size);
        crate::render::draw_q3_banner_string(
            title,
            screen_w / 2.0 - title_w / 2.0,
            screen_h * 0.15,
            title_size,
            Color::new(1.0, 0.8, 0.2, alpha),
        );
        
        let podium_y = screen_h * 0.65;
        let model_scale = screen_h * 0.7 / 120.0;
        
        for (i, (id, name, frags, excellent, impressive)) in self.scores.iter().enumerate() {
            let x_offset = match i {
                0 => screen_w * 0.5,
                1 => screen_w * 0.25,
                2 => screen_w * 0.75,
                _ => continue,
            };
            
            if let Some((_, model_name, _upper_frame, _lower_frame)) = self.player_models.iter().find(|m| m.0 == *id) {
                if let Some(model) = model_cache.get_mut(model_name) {
                    let model_color = if Some(*id) == self.winner_id {
                        Color::new(1.0, 0.9, 0.5, alpha)
                    } else {
                        Color::new(1.0, 1.0, 1.0, alpha * 0.85)
                    };
                    
                    let model_y_offset = match i {
                        0 => -30.0,
                        1 => -10.0,
                        2 => -10.0,
                        _ => 0.0,
                    };
                    
                    let (lower_frame, upper_frame) = if let Some(config) = &model.anim_config {
                        let time_elapsed = current_time - self.start_time;
                        
                        if i == 0 {
                            if let Some(victory_anim) = &config.both_victory {
                                let fps = victory_anim.fps as f32;
                                let frame_in_anim = (time_elapsed * fps) as usize;
                                let frame_offset = if victory_anim.looping_frames > 0 {
                                    frame_in_anim % victory_anim.looping_frames
                                } else {
                                    frame_in_anim.min(victory_anim.num_frames.saturating_sub(1))
                                };
                                let frame = (victory_anim.first_frame + frame_offset).min(190);
                                (frame, frame)
                            } else {
                                let legs_fps = config.legs_idle.fps as f32;
                                let legs_frame_in_anim = (time_elapsed * legs_fps) as usize;
                                let legs_frame_offset = if config.legs_idle.looping_frames > 0 {
                                    legs_frame_in_anim % config.legs_idle.looping_frames
                                } else {
                                    legs_frame_in_anim.min(config.legs_idle.num_frames.saturating_sub(1))
                                };
                                let lower = (config.legs_idle.first_frame + legs_frame_offset).min(190);
                                
                                let torso_fps = config.torso_stand.fps as f32;
                                let torso_frame_in_anim = (time_elapsed * torso_fps) as usize;
                                let torso_frame_offset = if config.torso_stand.looping_frames > 0 {
                                    torso_frame_in_anim % config.torso_stand.looping_frames
                                } else {
                                    torso_frame_in_anim.min(config.torso_stand.num_frames.saturating_sub(1))
                                };
                                let upper = (config.torso_stand.first_frame + torso_frame_offset).min(152);
                                
                                (lower, upper)
                            }
                        } else {
                            let (legs_anim, torso_anim) = match i {
                                1 => {
                                    (&config.legs_idle, &config.torso_stand)
                                }
                                2 => {
                                    (&config.death1, &config.death1)
                                }
                                _ => (&config.legs_idle, &config.torso_stand),
                            };
                            
                            let legs_fps = legs_anim.fps as f32;
                            let legs_frame_in_anim = (time_elapsed * legs_fps) as usize;
                            let legs_frame_offset = if legs_anim.looping_frames > 0 {
                                legs_frame_in_anim % legs_anim.looping_frames
                            } else {
                                legs_frame_in_anim.min(legs_anim.num_frames.saturating_sub(1))
                            };
                            let lower = (legs_anim.first_frame + legs_frame_offset).min(190);
                            
                            let torso_fps = torso_anim.fps as f32;
                            let torso_frame_in_anim = (time_elapsed * torso_fps) as usize;
                            let torso_frame_offset = if torso_anim.looping_frames > 0 {
                                torso_frame_in_anim % torso_anim.looping_frames
                            } else {
                                torso_frame_in_anim.min(torso_anim.num_frames.saturating_sub(1))
                            };
                            let upper = (torso_anim.first_frame + torso_frame_offset).min(152);
                            
                            (lower, upper)
                        }
                    } else {
                        let frame = ((current_time * 10.0) as usize) % 40;
                        (frame, frame)
                    };
                    
                    let is_winner = Some(*id) == self.winner_id;
                    let weapon_model = if is_winner {
                        self.winner_weapon.and_then(|w| weapon_model_cache.get(w))
                    } else {
                        None
                    };
                    
                    model.render_simple(
                        x_offset,
                        podium_y + model_y_offset,
                        model_color,
                        model_scale,
                        false,
                        0.0,
                        0.0,
                        lower_frame,
                        upper_frame,
                        weapon_model,
                        false,
                        None,
                        -std::f32::consts::PI / 2.0,
                        0.0,
                        0.0,
                        is_winner,
                        0.0,
                    );
                }
            }
            
            let name_y = podium_y + 80.0;
            let name_w = measure_text(name, None, 28, 1.0).width;
            let name_color = if Some(*id) == self.winner_id {
                Color::new(1.0, 0.8, 0.0, alpha)
            } else {
                Color::new(0.9, 0.9, 0.9, alpha)
            };
            draw_text(name, x_offset - name_w / 2.0, name_y, 28.0, name_color);
            
            let score_text = format!("{} frags", frags);
            let score_w = measure_text(&score_text, None, 24, 1.0).width;
            draw_text(&score_text, x_offset - score_w / 2.0, name_y + 30.0, 24.0, Color::new(0.95, 0.95, 0.95, alpha));
            
            if *excellent > 0 || *impressive > 0 {
                let icon_size = 32.0;
                let icon_y = name_y + 50.0;
                let mut award_x = x_offset;
                
                let total_width = if *excellent > 0 && *impressive > 0 {
                    icon_size * 2.0 + 50.0 + 20.0
                } else if *excellent > 0 {
                    icon_size + 30.0
                } else {
                    icon_size + 30.0
                };
                award_x -= total_width / 2.0;
                
                if *excellent > 0 {
                    if let Some(excellent_tex) = award_icon_cache.excellent.as_ref() {
                        draw_texture_ex(
                            excellent_tex,
                            award_x,
                            icon_y,
                            Color::new(1.0, 1.0, 1.0, alpha),
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(icon_size, icon_size)),
                                ..Default::default()
                            },
                        );
                        award_x += icon_size + 5.0;
                        draw_text(
                            &format!("{}", excellent),
                            award_x,
                            icon_y + icon_size - 6.0,
                            24.0,
                            Color::new(1.0, 0.85, 0.4, alpha),
                        );
                        award_x += 25.0 + 20.0;
                    }
                }
                
                if *impressive > 0 {
                    if let Some(impressive_tex) = award_icon_cache.impressive.as_ref() {
                        draw_texture_ex(
                            impressive_tex,
                            award_x,
                            icon_y,
                            Color::new(1.0, 1.0, 1.0, alpha),
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(icon_size, icon_size)),
                                ..Default::default()
                            },
                        );
                        award_x += icon_size + 5.0;
                        draw_text(
                            &format!("{}", impressive),
                            award_x,
                            icon_y + icon_size - 6.0,
                            24.0,
                            Color::new(1.0, 0.85, 0.4, alpha),
                        );
                    }
                }
            }
        }
    }
}





