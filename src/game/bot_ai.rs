use crate::game::player::Player;
use crate::game::weapon::Weapon;
use crate::game::map::{Map, ItemType};
use crate::game::nav_graph::NavGraph;
use crate::compat_rand::*;

#[derive(Clone, Debug)]
pub struct BotAI {
    pub think_time: u32,
    pub target_player: Option<u16>,
    pub target_item: Option<(f32, f32)>,
    pub move_direction: f32,
    pub want_jump: bool,
    pub want_shoot: bool,
    pub want_weapon: Option<Weapon>,
    pub evade_direction: f32,
    pub evade_timer: u32,
    pub rocket_jump_timer: u32,
    pub search_direction: f32,
    pub search_timer: u32,
    pub last_position: (f32, f32),
    pub stuck_counter: u32,
    pub current_path: Vec<usize>,
    pub path_target: Option<(f32, f32)>,
    pub path_update_timer: u32,
    pub afk: bool,
}

impl BotAI {
    pub fn new() -> Self {
        Self {
            think_time: 0,
            target_player: None,
            target_item: None,
            move_direction: 0.0,
            want_jump: false,
            want_shoot: false,
            want_weapon: None,
            evade_direction: 0.0,
            evade_timer: 0,
            rocket_jump_timer: 0,
            search_direction: 1.0,
            search_timer: 0,
            last_position: (0.0, 0.0),
            stuck_counter: 0,
            current_path: Vec::new(),
            path_target: None,
            path_update_timer: 0,
            afk: false,
        }
    }

    pub fn think(&mut self, bot: &Player, players: &[Player], map: &Map, projectiles: &[super::projectile::Projectile], nav_graph: Option<&NavGraph>) {
        if self.afk {
            self.move_direction = 0.0;
            self.want_jump = false;
            self.want_shoot = false;
            return;
        }
        
        self.think_time += 1;
        
        if self.evade_timer > 0 {
            self.evade_timer -= 1;
        }
        
        if self.think_time % 10 != 0 {
            return;
        }
        
        let pos_diff_x = (bot.x - self.last_position.0).abs();
        let pos_diff_y = (bot.y - self.last_position.1).abs();
        
        if pos_diff_x < 5.0 && pos_diff_y < 5.0 && self.move_direction.abs() > 0.1 {
            self.stuck_counter += 10;
        } else if self.stuck_counter > 0 {
            self.stuck_counter = self.stuck_counter.saturating_sub(5);
        }
        
        self.last_position = (bot.x, bot.y);
        
        if self.think_time % 60 == 0 {
            // println!("[Bot {}] stuck={} search_timer={} evade_timer={} rocket_timer={} move_dir={:.1}", 
            //     bot.id, self.stuck_counter, self.search_timer, self.evade_timer, 
            //     self.rocket_jump_timer, self.move_direction);
        }

        if self.path_update_timer > 0 {
            self.path_update_timer -= 1;
        }

        let mut closest_enemy: Option<(&Player, f32)> = None;
        let mut closest_dist = 10000.0;

        for player in players {
            if player.id != bot.id && !player.dead {
                let dx = player.x - bot.x;
                let dy = player.y - bot.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < closest_dist {
                    closest_dist = dist;
                    closest_enemy = Some((player, dist));
                }
            }
        }
        
        if self.rocket_jump_timer > 0 {
            self.rocket_jump_timer -= 1;
        }
        
        if self.search_timer > 0 {
            self.search_timer -= 1;
        }
        
        for pad in &map.jumppads {
            let near_pad = bot.x >= pad.x - 20.0
                && bot.x <= pad.x + pad.width + 20.0
                && bot.y >= pad.y - 40.0
                && bot.y <= pad.y + 20.0;
                
            if near_pad {
                if let Some((enemy, _)) = closest_enemy {
                    let dy = enemy.y - bot.y;
                    if dy < -50.0 {
                        let dx_to_pad = (pad.x + pad.width / 2.0) - bot.x;
                        self.move_direction = dx_to_pad.signum();
                    }
                }
            }
        }

        for proj in projectiles {
            if proj.active && proj.owner_id != bot.id {
                let dx = proj.x - bot.x;
                let dy = proj.y - bot.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < 120.0 {
                    let incoming = (proj.vel_x * dx + proj.vel_y * dy) < 0.0;
                    
                    if incoming {
                        self.evade_direction = if gen_range_i32(0, 2) == 0 { -1.0 } else { 1.0 };
                        self.evade_timer = 20;
                        self.want_jump = true;
                    }
                }
            }
        }

        let needs_health = bot.health < 50;
        let needs_armor = bot.armor < 50;
        
        let has_shotgun = bot.has_weapon[2] && bot.ammo[2] > 0;
        let has_grenade = bot.has_weapon[3] && bot.ammo[3] > 0;
        let has_rocket = bot.has_weapon[4] && bot.ammo[4] > 0;
        let has_rail = bot.has_weapon[6] && bot.ammo[6] > 0;
        let has_plasma = bot.has_weapon[7] && bot.ammo[7] > 0;
        let has_bfg = bot.has_weapon[8] && bot.ammo[8] > 0;
        
        let has_good_weapon = has_shotgun || has_grenade || has_rocket || has_rail || has_plasma || has_bfg;
        let needs_weapon = !has_good_weapon;

        let mut best_item: Option<(f32, f32, f32)> = None;
        let mut best_item_priority = 0;
        
        if needs_health || needs_armor || needs_weapon {
            for item in &map.items {
                if !item.active {
                    continue;
                }
                
                let (is_useful, priority) = match item.item_type {
                    ItemType::Health25 | ItemType::Health50 | ItemType::Health100 => (needs_health, 2),
                    ItemType::Armor50 | ItemType::Armor100 => (needs_armor, 3),
                    ItemType::RocketLauncher | ItemType::Railgun => (needs_weapon, 5),
                    ItemType::Plasmagun | ItemType::Shotgun | ItemType::GrenadeLauncher => (needs_weapon, 4),
                    ItemType::Quad | ItemType::Haste | ItemType::Regen => (true, 6),
                    _ => (false, 0),
                };
                
                if is_useful {
                    let dx = item.x - bot.x;
                    let dy = item.y - bot.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if best_item.is_none() || priority > best_item_priority || (priority == best_item_priority && dist < best_item.unwrap().2) {
                        best_item = Some((item.x, item.y, dist));
                        best_item_priority = priority;
                    }
                }
            }
        }

        if let Some((enemy, dist)) = closest_enemy {
            let has_los = map.has_line_of_sight(bot.x, bot.y - 24.0, enemy.x, enemy.y - 24.0);
            
            if !has_los {
                self.want_shoot = false;
            }
            
            self.target_player = Some(enemy.id);

            let dx = enemy.x - bot.x;
            let dy = enemy.y - bot.y;
            let dx_abs = dx.abs();
            
            let mut found_navigation_aid = false;
            let is_stuck = self.stuck_counter >= 50;
            
            if let Some(nav_graph) = nav_graph {
                if !has_los || self.stuck_counter > 20 || dy.abs() > 80.0 || (dx_abs < 100.0 && dy.abs() > 50.0) || (dx_abs < 32.0 && dy.abs() > 40.0) {
                    if self.use_nav_graph(bot, (enemy.x, enemy.y), nav_graph, map) {
                        found_navigation_aid = true;
                        // if self.think_time % 120 == 0 {
                        //     println!("[Bot {}] Using nav graph to reach enemy", bot.id);
                        // }
                    }
                }
            }
            
            if !found_navigation_aid && is_stuck {
                if self.search_timer == 0 {
                    self.search_direction = -self.move_direction;
                    if self.search_direction == 0.0 {
                        self.search_direction = if gen_range_i32(0, 2) == 0 { 1.0 } else { -1.0 };
                    }
                    self.search_timer = 100;
                    self.want_jump = true;
                    self.stuck_counter = 0;
                    // println!("[Bot {}] STUCK! Reversing direction to {:.1}", bot.id, self.search_direction);
                }
                self.move_direction = self.search_direction;
                found_navigation_aid = true;
            }
            
            if !found_navigation_aid && (dy < -50.0 && dx_abs < 100.0) {
                for pad in &map.jumppads {
                    let dx_to_pad = pad.x + pad.width / 2.0 - bot.x;
                    let dy_to_pad = pad.y - bot.y;
                    let dist_to_pad = (dx_to_pad * dx_to_pad + dy_to_pad * dy_to_pad).sqrt();
                    
                    if dist_to_pad < 300.0 {
                        self.move_direction = dx_to_pad.signum();
                        found_navigation_aid = true;
                        // if self.think_time % 60 == 0 {
                        //     println!("[Bot {}] Enemy above, moving to jumppad at ({:.0},{:.0})", bot.id, pad.x, pad.y);
                        // }
                        break;
                    }
                }
                
                if !found_navigation_aid {
                    for teleporter in &map.teleporters {
                        let dx_to_tele = teleporter.x + teleporter.width / 2.0 - bot.x;
                        let dy_to_tele = teleporter.y + teleporter.height / 2.0 - bot.y;
                        let dist_to_tele = (dx_to_tele * dx_to_tele + dy_to_tele * dy_to_tele).sqrt();
                        
                        if dist_to_tele < 300.0 {
                            let dx_after = enemy.x - teleporter.dest_x;
                            let dy_after = enemy.y - teleporter.dest_y;
                            let dist_after = (dx_after * dx_after + dy_after * dy_after).sqrt();
                            
                            if dist_after < dist - 50.0 {
                                self.move_direction = dx_to_tele.signum();
                                found_navigation_aid = true;
                                // if self.think_time % 60 == 0 {
                                //     println!("[Bot {}] Enemy above, moving to teleporter", bot.id);
                                // }
                                break;
                            }
                        }
                    }
                }
                
                if !found_navigation_aid && !has_los {
                    let on_ground = map.is_solid(((bot.x) / 32.0) as i32, ((bot.y + 16.0) / 16.0) as i32);
                    
                    if dy < -100.0 && dx_abs < 50.0 && bot.has_weapon[4] && bot.ammo[4] > 5 && on_ground && self.rocket_jump_timer == 0 {
                        if gen_range_i32(0, 100) < 60 {
                            self.rocket_jump_timer = 200;
                            found_navigation_aid = true;
                            // if self.think_time % 60 == 0 {
                            //     println!("[Bot {}] Enemy directly above, rocket jumping!", bot.id);
                            // }
                        }
                    }
                }
                
                if !found_navigation_aid && dx_abs < 50.0 {
                    if self.search_timer == 0 {
                        self.search_direction = if gen_range_i32(0, 2) == 0 { 1.0 } else { -1.0 };
                        self.search_timer = 150;
                        // if self.think_time % 60 == 0 {
                        //     println!("[Bot {}] Enemy above, searching for way up, dir={:.1}", bot.id, self.search_direction);
                        // }
                    }
                    self.move_direction = self.search_direction;
                    self.want_jump = true;
                    found_navigation_aid = true;
                }
            }
            
            if !found_navigation_aid && (dy > 50.0 && dx_abs < 100.0) {
                for teleporter in &map.teleporters {
                    let dx_to_tele = teleporter.x + teleporter.width / 2.0 - bot.x;
                    let dy_to_tele = teleporter.y + teleporter.height / 2.0 - bot.y;
                    let dist_to_tele = (dx_to_tele * dx_to_tele + dy_to_tele * dy_to_tele).sqrt();
                    
                    if dist_to_tele < 300.0 {
                        let dx_after = enemy.x - teleporter.dest_x;
                        let dy_after = enemy.y - teleporter.dest_y;
                        let dist_after = (dx_after * dx_after + dy_after * dy_after).sqrt();
                        
                        if dist_after < dist - 50.0 {
                            self.move_direction = dx_to_tele.signum();
                            found_navigation_aid = true;
                            // if self.think_time % 60 == 0 {
                            //     println!("[Bot {}] Enemy below, moving to teleporter", bot.id);
                            // }
                            break;
                        }
                    }
                }
                
                if !found_navigation_aid && dx_abs < 50.0 {
                    if self.search_timer == 0 {
                        self.search_direction = if gen_range_i32(0, 2) == 0 { 1.0 } else { -1.0 };
                        self.search_timer = 150;
                        // if self.think_time % 60 == 0 {
                        //     println!("[Bot {}] Enemy below, searching for way down, dir={:.1}", bot.id, self.search_direction);
                        // }
                    }
                    self.move_direction = self.search_direction;
                    found_navigation_aid = true;
                }
            }
            
            if self.evade_timer > 0 {
                self.move_direction = self.evade_direction;
            } else if self.search_timer > 0 && !found_navigation_aid {
                self.move_direction = self.search_direction;
                // if self.think_time % 60 == 0 {
                //     println!("[Bot {}] Following search path, timer={}", bot.id, self.search_timer);
                // }
            } else if !found_navigation_aid {
                if let Some((item_x, item_y, item_dist)) = best_item {
                    let must_get_item = (needs_health && bot.health < 30) 
                        || needs_armor 
                        || needs_weapon
                        || best_item_priority >= 6;
                    
                    if must_get_item || item_dist < 150.0 {
                        self.target_item = Some((item_x, item_y));
                        let item_dx = item_x - bot.x;
                        self.move_direction = item_dx.signum();
                        // if self.think_time % 60 == 0 {
                        //     println!("[Bot {}] Going for item (priority={})", bot.id, best_item_priority);
                        // }
                    } else {
                        self.target_item = None;
                        self.search_timer = 0;
                        if dist > 50.0 && dx_abs > 10.0 {
                            self.move_direction = dx.signum();
                                // if self.think_time % 60 == 0 {
                                //     println!("[Bot {}] Moving toward enemy (normal)", bot.id);
                                // }
                        } else if dist < 100.0 && has_los {
                            self.move_direction = -dx.signum();
                            // if self.think_time % 60 == 0 {
                            //     println!("[Bot {}] Backing away (too close)", bot.id);
                            // }
                        } else if dx_abs <= 10.0 {
                            self.move_direction = dx.signum();
                            // if self.think_time % 60 == 0 {
                            //     println!("[Bot {}] Aligned horizontally, minor adjustment", bot.id);
                            // }
                        } else {
                            self.move_direction = 0.0;
                            // if self.think_time % 60 == 0 {
                                // println!("[Bot {}] STANDING STILL - dist={:.0} dx_abs={:.0} dy={:.0}", bot.id, dist, dx_abs, dy.abs());
                            // }
                        }
                    }
                } else {
                    self.target_item = None;
                    self.search_timer = 0;
                    if dist > 50.0 && dx_abs > 10.0 {
                        self.move_direction = dx.signum();
                    } else if dist < 100.0 && has_los {
                        self.move_direction = -dx.signum();
                    } else if dx_abs <= 10.0 {
                        self.move_direction = dx.signum();
                    } else {
                        self.move_direction = 0.0;
                    }
                }
            }

            if !found_navigation_aid {
                let on_ground = map.is_solid(((bot.x) / 32.0) as i32, ((bot.y + 16.0) / 16.0) as i32);

                if on_ground {
                    if dy < -100.0 && dist > 300.0 && bot.has_weapon[4] && bot.ammo[4] > 3 && self.rocket_jump_timer == 0 && gen_range_i32(0, 100) < 15 {
                        self.rocket_jump_timer = 200;
                    } else if dy < -32.0 {
                        self.want_jump = true;
                    } else if self.evade_timer > 0 {
                        self.want_jump = true;
                    } else {
                        if gen_range_i32(0, 50) == 0 {
                            self.want_jump = true;
                        } else {
                            self.want_jump = false;
                        }
                    }
                } else {
                    self.want_jump = false;
                }

                let check_ahead_x = bot.x + self.move_direction * 16.0;
                let check_ahead_tile_x = (check_ahead_x / 32.0) as i32;
                let check_ahead_tile_y = ((bot.y) / 16.0) as i32;

                if on_ground && map.is_solid(check_ahead_tile_x, check_ahead_tile_y) {
                    self.want_jump = true;
                }
            }

            if dist < 800.0 && dist > 80.0 && has_los {
                self.want_shoot = true;

                if bot.weapon == Weapon::RocketLauncher || bot.weapon == Weapon::GrenadeLauncher {
                    let lead_factor = (dist / 15.0).clamp(1.0, 15.0);
                    let predicted_x = enemy.x + enemy.vel_x * lead_factor;
                    let predicted_y = enemy.y + enemy.vel_y * lead_factor;
                    let dx_pred = predicted_x - bot.x;
                    let dy_pred = predicted_y - bot.y;
                    let _angle_to_predicted = dy_pred.atan2(dx_pred);
                }

                let best_weapon = if dist > 400.0 && bot.has_weapon[4] && bot.ammo[4] > 0 {
                    Some(Weapon::RocketLauncher)
                } else if dist > 300.0 && bot.has_weapon[6] && bot.ammo[6] > 0 {
                    Some(Weapon::Railgun)
                } else if dist < 200.0 && bot.has_weapon[2] && bot.ammo[2] > 0 {
                    Some(Weapon::Shotgun)
                } else if dist < 150.0 && bot.has_weapon[7] && bot.ammo[7] > 0 {
                    Some(Weapon::Plasmagun)
                } else if bot.has_weapon[3] && bot.ammo[3] > 0 {
                    Some(Weapon::GrenadeLauncher)
                } else if bot.has_weapon[1] && bot.ammo[1] > 0 {
                    Some(Weapon::MachineGun)
                } else if bot.ammo[bot.weapon as usize] > 0 || bot.weapon as u8 == 0 {
                    None
                } else {
                    Some(Weapon::Gauntlet)
                };
                
                if let Some(weapon) = best_weapon {
                    self.want_weapon = Some(weapon);
                }
            } else if dist < 80.0 {
                self.want_shoot = false;
            } else {
                self.want_shoot = false;
            }
        } else if let Some((item_x, item_y, _)) = best_item {
            self.target_item = Some((item_x, item_y));
            
            let mut used_nav = false;
            if let Some(nav_graph) = nav_graph {
                if self.use_nav_graph(bot, (item_x, item_y), nav_graph, map) {
                    used_nav = true;
                }
            }
            
            if !used_nav {
            let item_dx = item_x - bot.x;
            self.move_direction = item_dx.signum();
            
            let on_ground = map.is_solid(((bot.x) / 32.0) as i32, ((bot.y + 16.0) / 16.0) as i32);
            if on_ground {
                let item_dy = item_y - bot.y;
                if item_dy < -32.0 {
                    self.want_jump = true;
                } else {
                    self.want_jump = false;
                    }
                }
            }
        } else {
            self.move_direction = if gen_range_i32(0, 2) == 0 { -1.0 } else { 1.0 };
            self.want_shoot = false;
            self.target_item = None;
            
            if gen_range_i32(0, 20) == 0 {
                self.want_jump = true;
            }
        }
    }
    
    fn find_reachable_node(x: f32, y: f32, nav_graph: &NavGraph, _map: &Map) -> Option<usize> {
        let feet_y = y + 24.0;
        
        let mut best = None;
        let mut best_dist = f32::MAX;
        
        for (idx, node) in nav_graph.nodes.iter().enumerate() {
            let dy = node.y - feet_y;
            let dx = (node.x - x).abs();
            
            if dy > 48.0 || dy < -64.0 {
                continue;
            }
            
            if dx > 96.0 {
                continue;
            }
            
            let dist = dx * dx + dy * dy;
            if dist < best_dist {
                best_dist = dist;
                best = Some(idx);
            }
        }
        
        best.or_else(|| nav_graph.find_nearest_node(x, feet_y))
    }

    pub fn use_nav_graph(&mut self, bot: &Player, target: (f32, f32), nav_graph: &NavGraph, map: &Map) -> bool {
        let target_changed = match self.path_target {
            Some((tx, ty)) => (tx - target.0).abs() > 64.0 || (ty - target.1).abs() > 64.0,
            None => true,
        };
        
        if target_changed || self.path_update_timer == 0 || self.current_path.is_empty() || self.current_path.len() <= 1 {
            self.path_target = Some(target);
            self.path_update_timer = 30;
            
            let start_node = Self::find_reachable_node(bot.x, bot.y, nav_graph, map);
            
            if let Some(start) = start_node {
                // let start_node_info = &nav_graph.nodes[start];
                // println!("[Bot {}] Bot at ({:.0},{:.0}), nearest reachable node {} at ({:.0},{:.0})", 
                    // bot.id, bot.x, bot.y, start, start_node_info.x, start_node_info.y);
                
                let mut candidates: Vec<(usize, f32)> = nav_graph.nodes
                    .iter()
                    .enumerate()
                    .map(|(idx, node)| {
                        let dx = node.x - target.0;
                        let dy = node.y - target.1;
                        let dist = dx * dx + dy * dy;
                        (idx, dist)
                    })
                    .collect();
                
                candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                
                for (goal, _dist) in candidates.iter().take(10) {
                    if let Some(path) = nav_graph.find_path(start, *goal) {
                        self.current_path = path;
                        let _goal_node = &nav_graph.nodes[*goal];
                        // println!("[Bot {}] Found path: {} waypoints from node {} ({:.0},{:.0}) to node {} ({:.0},{:.0}), target=({:.0},{:.0})", 
                        //     bot.id, self.current_path.len(), 
                        //     start, nav_graph.nodes[start].x, nav_graph.nodes[start].y,
                        //     goal, goal_node.x, goal_node.y,
                        //     target.0, target.1);
                        
                        // for i in 0..self.current_path.len().min(5) {
                        //     let node = &nav_graph.nodes[self.current_path[i]];
                        //     let edge_info = if i + 1 < self.current_path.len() {
                        //         nav_graph.edges.iter()
                        //             .find(|e| e.from == self.current_path[i] && e.to == self.current_path[i+1])
                        //             .map(|e| format!("{:?}", e.edge_type))
                        //             .unwrap_or("?".to_string())
                        //     } else {
                        //         "END".to_string()
                        //     };
                        //     println!("  [{}] node {} at ({:.0},{:.0}) -> {}", i, self.current_path[i], node.x, node.y, edge_info);
                        // }
                        
                        return true;
                    }
                }
                
                // println!("[Bot {}] No path found from node {} to any of 10 nearest nodes to target", bot.id, start);
            } else {
                // println!("[Bot {}] Could not find start node near ({:.0},{:.0})", bot.id, bot.x, bot.y);
            }
            return false;
        }
        
        if self.current_path.len() > 1 {
            let current_node_id = self.current_path[0];
            let next_node_id = self.current_path[1];
            let next_node = &nav_graph.nodes[next_node_id];
            
            let edge_to_next = nav_graph.edges.iter().find(|e| 
                e.from == current_node_id && e.to == next_node_id
            );
            
            let dx = next_node.x - bot.x;
            let dy = next_node.y - bot.y;
            let dx_abs = dx.abs();
            let dist = (dx * dx + dy * dy).sqrt();
            
            if self.think_time % 120 == 0 {
                let _edge_type = edge_to_next.map(|e| format!("{:?}", e.edge_type)).unwrap_or("Unknown".to_string());
                // println!("[Bot {}] Path: {}/{} waypoints, next at ({:.0},{:.0}), type={}, dist={:.0}, bot at ({:.0},{:.0})", 
                //     bot.id, self.current_path.len() - 1, self.current_path.len(), 
                //     next_node.x, next_node.y, edge_type, dist, bot.x, bot.y);
            }
            
            let on_ground = map.is_solid(((bot.x) / 32.0) as i32, ((bot.y + 24.0) / 16.0) as i32);
            
            let reached = if let Some(edge) = edge_to_next {
                match edge.edge_type {
                    super::nav_graph::NavEdgeType::Walk => dx_abs < 48.0 && dy.abs() < 32.0,
                    super::nav_graph::NavEdgeType::Jump => dx_abs < 48.0 && dy < 32.0 && !on_ground,
                    super::nav_graph::NavEdgeType::FallOff | super::nav_graph::NavEdgeType::JumpDown => 
                        dx_abs < 64.0 && dy > -32.0,
                    super::nav_graph::NavEdgeType::JumpGap => dx_abs < 64.0 && dy.abs() < 32.0 && !on_ground,
                    super::nav_graph::NavEdgeType::JumpPad | super::nav_graph::NavEdgeType::Teleport => 
                        dx_abs < 32.0 && dy.abs() < 48.0,
                    _ => dx_abs < 64.0 && dy.abs() < 64.0,
                }
            } else {
                dx_abs < 48.0 && dy.abs() < 32.0
            };
            
            if reached {
                self.current_path.remove(0);
                self.stuck_counter = 0;
                if self.current_path.len() > 1 {
                    let new_next = &nav_graph.nodes[self.current_path[1]];
                    let new_dx = new_next.x - bot.x;
                    self.move_direction = new_dx.signum();
                    // println!("[Bot {}] Reached waypoint! {} remaining, next at ({:.0}, {:.0})", 
                    //     bot.id, self.current_path.len() - 1, new_next.x, new_next.y);
                } else {
                    // println!("[Bot {}] Reached final waypoint!", bot.id);
                    self.current_path.clear();
                }
            } else {
                if self.stuck_counter > 30 && dist > 100.0 {
                    // println!("[Bot {}] Stuck while following path, skipping waypoint", bot.id);
                    self.current_path.remove(0);
                    self.stuck_counter = 0;
                    if self.current_path.len() <= 1 {
                        self.current_path.clear();
                        return false;
                    }
                }
                
                self.move_direction = dx.signum();
                
                let is_next_jumppad = next_node.node_type == super::nav_graph::NavNodeType::JumpPad;
                
                if let Some(edge) = edge_to_next {
                    match edge.edge_type {
                        super::nav_graph::NavEdgeType::Jump | 
                        super::nav_graph::NavEdgeType::JumpGap => {
                            if !is_next_jumppad && dy <= -4.0 {
                                self.want_jump = true;
                            } else if self.think_time % 120 == 0 && dy <= -4.0 {
                                // println!("[Bot {}] NOT jumping: is_next_jumppad={}, next_node_type={:?}", 
                                //     bot.id, is_next_jumppad, next_node.node_type);
                            }
                        },
                        super::nav_graph::NavEdgeType::JumpDown => {
                            if !is_next_jumppad && dx_abs > 32.0 {
                                self.want_jump = true;
                            }
                        },
                        super::nav_graph::NavEdgeType::AirControl => {
                            if dy > 16.0 && dx_abs < 64.0 {
                                self.move_direction = dx.signum();
                            }
                        },
                        super::nav_graph::NavEdgeType::RocketJump => {
                            if dy < -48.0 && bot.weapon as u8 == 4 {
                                self.want_jump = true;
                                if self.rocket_jump_timer == 0 {
                                    self.want_shoot = true;
                                    self.rocket_jump_timer = 15;
                                }
                            }
                        },
                        super::nav_graph::NavEdgeType::JumpPad => {
                            if dx_abs < 48.0 {
                                self.move_direction = 0.0;
                            }
                        },
                        super::nav_graph::NavEdgeType::FallOff => {
                            if dx_abs < 32.0 {
                                self.move_direction = dx.signum();
                            }
                        },
                        _ => {}
                    }
                } else if !is_next_jumppad && dy < -32.0 {
                    self.want_jump = true;
                }
            }
            
            return true;
        } else if !self.current_path.is_empty() {
            // println!("[Bot {}] Path has only {} nodes, not following", bot.id, self.current_path.len());
        }
        
        false
    }
}

