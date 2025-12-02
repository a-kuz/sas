use crate::game::map::Map;
use crate::game::player::Player;
use crate::game::weapon::Weapon;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct EnemySpawn {
    pub x: f32,
    pub y: f32,
    pub model: String,
    pub health: i32,
    pub armor: i32,
    pub weapons: [bool; 9],
    pub ammo: [u8; 9],
    pub weapon: Weapon,
    pub spawned: bool,
    pub is_boss: bool,
    pub boss_name: String,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct StoryLevel {
    pub name: String,
    pub description: String,
    pub briefing: String,
    pub map_name: String,
    pub enemy_spawns: Vec<EnemySpawn>,
    pub exit_x: f32,
    pub exit_y: f32,
    pub exit_radius: f32,
}

#[derive(Clone, Debug)]
pub struct StoryMode {
    pub active: bool,
    pub current_level: usize,
    pub levels: Vec<StoryLevel>,
    pub enemies_killed: u32,
    pub total_enemies: u32,
    pub level_complete: bool,
    pub level_complete_timer: f32,
    pub next_level_ready: bool,
    pub show_briefing: bool,
    pub briefing_timer: f32,
    pub campaign_complete: bool,
}

impl StoryMode {
    pub fn new() -> Self {
        let levels = Self::create_campaign();
        let total_enemies = if !levels.is_empty() {
            levels[0].enemy_spawns.len() as u32
        } else {
            0
        };

        Self {
            active: true,
            current_level: 0,
            levels,
            enemies_killed: 0,
            total_enemies,
            level_complete: false,
            level_complete_timer: 0.0,
            next_level_ready: false,
            show_briefing: true,
            briefing_timer: 6.0,
            campaign_complete: false,
        }
    }

    fn create_campaign() -> Vec<StoryLevel> {
        vec![
            StoryLevel {
                name: "TRAINING GROUND".to_string(),
                description: "First Contact".to_string(),
                briefing: "Vadrigar prison breached. Corrupted soldiers detected in sector 0."
                    .to_string(),
                map_name: "0-arena".to_string(),
                exit_x: 850.0,
                exit_y: 600.0,
                exit_radius: 120.0,
                enemy_spawns: vec![
                    EnemySpawn {
                        x: 620.0,
                        y: 170.0,
                        model: "grunt".to_string(),
                        health: 100,
                        armor: 0,
                        weapons: [true, true, false, false, false, false, false, false, false],
                        ammo: [255, 100, 0, 0, 0, 0, 0, 0, 0],
                        weapon: Weapon::MachineGun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 850.0,
                        y: 290.0,
                        model: "grunt".to_string(),
                        health: 100,
                        armor: 0,
                        weapons: [true, true, false, false, false, false, false, false, false],
                        ammo: [255, 100, 0, 0, 0, 0, 0, 0, 0],
                        weapon: Weapon::MachineGun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1350.0,
                        y: 285.0,
                        model: "sarge".to_string(),
                        health: 150,
                        armor: 25,
                        weapons: [true, true, true, false, false, false, false, false, false],
                        ammo: [255, 150, 80, 0, 0, 0, 0, 0, 0],
                        weapon: Weapon::Shotgun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                ],
            },
            StoryLevel {
                name: "BLOOD ARENA".to_string(),
                description: "The Spread".to_string(),
                briefing: "Corruption accelerating. Elite squads deployed to sector 1.".to_string(),
                map_name: "1-arena".to_string(),
                exit_x: 850.0,
                exit_y: 550.0,
                exit_radius: 120.0,
                enemy_spawns: vec![
                    EnemySpawn {
                        x: 600.0,
                        y: 350.0,
                        model: "razor".to_string(),
                        health: 180,
                        armor: 50,
                        weapons: [true, true, false, true, false, false, false, false, false],
                        ammo: [255, 100, 0, 150, 0, 0, 0, 0, 0],
                        weapon: Weapon::GrenadeLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1000.0,
                        y: 250.0,
                        model: "visor".to_string(),
                        health: 180,
                        armor: 50,
                        weapons: [true, true, true, false, false, false, false, false, false],
                        ammo: [255, 150, 100, 0, 0, 0, 0, 0, 0],
                        weapon: Weapon::Shotgun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1400.0,
                        y: 350.0,
                        model: "hunter".to_string(),
                        health: 200,
                        armor: 75,
                        weapons: [true, true, false, false, true, false, false, false, false],
                        ammo: [255, 150, 0, 0, 150, 0, 0, 0, 0],
                        weapon: Weapon::RocketLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1200.0,
                        y: 500.0,
                        model: "keel".to_string(),
                        health: 200,
                        armor: 50,
                        weapons: [true, true, true, true, false, false, false, false, false],
                        ammo: [255, 150, 100, 150, 0, 0, 0, 0, 0],
                        weapon: Weapon::GrenadeLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                ],
            },
            StoryLevel {
                name: "DIMENSIONAL RIFT".to_string(),
                description: "Reality Fractures".to_string(),
                briefing: "WARNING: Dimensional tear expanding. Boss-class entity approaching!"
                    .to_string(),
                map_name: "new_map".to_string(),
                exit_x: 750.0,
                exit_y: 250.0,
                exit_radius: 120.0,
                enemy_spawns: vec![
                    EnemySpawn {
                        x: 600.0,
                        y: 300.0,
                        model: "bones".to_string(),
                        health: 220,
                        armor: 75,
                        weapons: [true, true, true, true, false, false, false, false, false],
                        ammo: [255, 200, 150, 150, 0, 0, 0, 0, 0],
                        weapon: Weapon::GrenadeLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 800.0,
                        y: 400.0,
                        model: "uriel".to_string(),
                        health: 250,
                        armor: 75,
                        weapons: [true, true, false, false, true, false, false, true, false],
                        ammo: [255, 200, 0, 0, 150, 0, 0, 200, 0],
                        weapon: Weapon::Plasmagun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1200.0,
                        y: 300.0,
                        model: "crash".to_string(),
                        health: 250,
                        armor: 100,
                        weapons: [true, true, true, false, true, false, true, false, false],
                        ammo: [255, 200, 150, 0, 150, 0, 100, 0, 0],
                        weapon: Weapon::Railgun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1000.0,
                        y: 500.0,
                        model: "orbb".to_string(),
                        health: 250,
                        armor: 100,
                        weapons: [true, true, true, true, true, false, true, false, false],
                        ammo: [255, 200, 150, 150, 150, 0, 150, 0, 0],
                        weapon: Weapon::RocketLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 900.0,
                        y: 250.0,
                        model: "doom".to_string(),
                        health: 700,
                        armor: 200,
                        weapons: [true, true, true, true, true, false, true, true, false],
                        ammo: [255, 255, 200, 200, 200, 0, 200, 200, 0],
                        weapon: Weapon::RocketLauncher,
                        spawned: false,
                        is_boss: true,
                        boss_name: "DOOM INCARNATE".to_string(),
                        scale: 1.4,
                    },
                ],
            },
            StoryLevel {
                name: "DARK TOURNAMENT".to_string(),
                description: "Champions Fall".to_string(),
                briefing: "CRITICAL: Multiple champion signatures! Dimensional overlords detected!"
                    .to_string(),
                map_name: "2-arena".to_string(),
                exit_x: 900.0,
                exit_y: 500.0,
                exit_radius: 120.0,
                enemy_spawns: vec![
                    EnemySpawn {
                        x: 700.0,
                        y: 300.0,
                        model: "slash".to_string(),
                        health: 300,
                        armor: 120,
                        weapons: [true, true, true, true, true, false, true, false, false],
                        ammo: [255, 255, 200, 200, 200, 0, 180, 0, 0],
                        weapon: Weapon::RocketLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 900.0,
                        y: 450.0,
                        model: "lucy".to_string(),
                        health: 300,
                        armor: 120,
                        weapons: [true, true, true, true, true, false, true, true, false],
                        ammo: [255, 255, 200, 200, 200, 0, 180, 200, 0],
                        weapon: Weapon::Plasmagun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1300.0,
                        y: 300.0,
                        model: "anarki".to_string(),
                        health: 300,
                        armor: 120,
                        weapons: [true, true, true, true, true, false, true, false, false],
                        ammo: [255, 255, 200, 200, 200, 0, 180, 0, 0],
                        weapon: Weapon::GrenadeLauncher,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1500.0,
                        y: 450.0,
                        model: "biker".to_string(),
                        health: 350,
                        armor: 150,
                        weapons: [true, true, true, true, true, false, true, true, false],
                        ammo: [255, 255, 200, 200, 200, 0, 200, 200, 0],
                        weapon: Weapon::Railgun,
                        spawned: false,
                        is_boss: false,
                        boss_name: String::new(),
                        scale: 1.0,
                    },
                    EnemySpawn {
                        x: 1100.0,
                        y: 250.0,
                        model: "xaero".to_string(),
                        health: 800,
                        armor: 280,
                        weapons: [true, true, true, true, true, false, true, true, false],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 0],
                        weapon: Weapon::Railgun,
                        spawned: false,
                        is_boss: true,
                        boss_name: "XAERO THE DESTROYER".to_string(),
                        scale: 1.5,
                    },
                    EnemySpawn {
                        x: 1100.0,
                        y: 550.0,
                        model: "major".to_string(),
                        health: 800,
                        armor: 280,
                        weapons: [true, true, true, true, true, false, true, true, false],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 0],
                        weapon: Weapon::RocketLauncher,
                        spawned: false,
                        is_boss: true,
                        boss_name: "MAJOR DEVASTATION".to_string(),
                        scale: 1.5,
                    },
                ],
            },
            StoryLevel {
                name: "VOID NEXUS".to_string(),
                description: "THE SOURCE".to_string(),
                briefing: "FINAL OPERATION: Breach core located. Destroy the Void Keeper!"
                    .to_string(),
                map_name: "my_arena".to_string(),
                exit_x: 1000.0,
                exit_y: 700.0,
                exit_radius: 130.0,
                enemy_spawns: vec![
                    EnemySpawn {
                        x: 700.0,
                        y: 250.0,
                        model: "sorlag".to_string(),
                        health: 900,
                        armor: 350,
                        weapons: [true, true, true, true, true, false, true, true, true],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 220],
                        weapon: Weapon::BFG,
                        spawned: false,
                        is_boss: true,
                        boss_name: "SORLAG DIMENSION LORD".to_string(),
                        scale: 1.6,
                    },
                    EnemySpawn {
                        x: 1300.0,
                        y: 250.0,
                        model: "klesk".to_string(),
                        health: 900,
                        armor: 350,
                        weapons: [true, true, true, true, true, false, true, true, true],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 220],
                        weapon: Weapon::Plasmagun,
                        spawned: false,
                        is_boss: true,
                        boss_name: "KLESK VOID WARRIOR".to_string(),
                        scale: 1.6,
                    },
                    EnemySpawn {
                        x: 900.0,
                        y: 500.0,
                        model: "mynx".to_string(),
                        health: 900,
                        armor: 350,
                        weapons: [true, true, true, true, true, false, true, true, true],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 220],
                        weapon: Weapon::Railgun,
                        spawned: false,
                        is_boss: true,
                        boss_name: "MYNX SHADOW QUEEN".to_string(),
                        scale: 1.6,
                    },
                    EnemySpawn {
                        x: 1100.0,
                        y: 500.0,
                        model: "ranger".to_string(),
                        health: 1000,
                        armor: 400,
                        weapons: [true, true, true, true, true, false, true, true, true],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 255],
                        weapon: Weapon::RocketLauncher,
                        spawned: false,
                        is_boss: true,
                        boss_name: "RANGER ETERNAL".to_string(),
                        scale: 1.7,
                    },
                    EnemySpawn {
                        x: 1000.0,
                        y: 350.0,
                        model: "tankjr".to_string(),
                        health: 2500,
                        armor: 600,
                        weapons: [true, true, true, true, true, false, true, true, true],
                        ammo: [255, 255, 255, 255, 255, 0, 255, 255, 255],
                        weapon: Weapon::BFG,
                        spawned: false,
                        is_boss: true,
                        boss_name: "THE VOID KEEPER".to_string(),
                        scale: 2.2,
                    },
                ],
            },
        ]
    }

    pub fn update(
        &mut self,
        dt: f32,
        players: &mut Vec<Player>,
        _map: &Map,
    ) -> (Vec<Player>, bool) {
        if !self.active || self.current_level >= self.levels.len() {
            return (Vec::new(), false);
        }

        if self.show_briefing {
            self.briefing_timer -= dt;
            if self.briefing_timer <= 0.0 {
                self.show_briefing = false;
            }
            return (Vec::new(), false);
        }

        let mut new_enemies = Vec::new();
        let mut should_change_level = false;

        let level = &mut self.levels[self.current_level];

        for spawn in &mut level.enemy_spawns {
            if !spawn.spawned {
                let enemy = Self::create_enemy_from_spawn(spawn);
                new_enemies.push(enemy);
                spawn.spawned = true;
            }
        }

        let alive_enemies = players
            .iter()
            .filter(|p| p.is_bot && !p.dead && !p.gibbed)
            .count();

        if alive_enemies == 0 && level.enemy_spawns.iter().all(|s| s.spawned) {
            if !self.level_complete {
                self.level_complete = true;
                self.level_complete_timer = 0.0;
            }
        }

        if self.level_complete {
            self.level_complete_timer += dt;
        }

        if let Some(player) = players.iter().find(|p| !p.is_bot) {
            let dx = player.x - level.exit_x;
            let dy = player.y - level.exit_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < level.exit_radius && self.level_complete {
                self.next_level_ready = true;
                should_change_level = true;
            }
        }

        (new_enemies, should_change_level)
    }

    fn create_enemy_from_spawn(spawn: &EnemySpawn) -> Player {
        let enemy_id = (2000 + crate::compat_rand::gen_range(0, 1000)) as u16;
        let mut enemy = Player::new(enemy_id, spawn.model.clone(), true);
        enemy.model = spawn.model.clone();
        enemy.x = spawn.x;
        enemy.y = spawn.y;
        enemy.health = spawn.health;
        enemy.armor = spawn.armor;
        enemy.has_weapon = spawn.weapons;
        enemy.ammo = spawn.ammo;
        enemy.weapon = spawn.weapon;
        enemy
    }

    pub fn advance_to_next_level(&mut self) -> Option<String> {
        self.current_level += 1;
        self.level_complete = false;
        self.level_complete_timer = 0.0;
        self.next_level_ready = false;
        self.enemies_killed = 0;
        self.show_briefing = true;
        self.briefing_timer = 6.0;

        if self.current_level < self.levels.len() {
            self.total_enemies = self.levels[self.current_level].enemy_spawns.len() as u32;
            Some(self.levels[self.current_level].map_name.clone())
        } else {
            self.active = false;
            self.campaign_complete = true;
            None
        }
    }

    pub fn get_current_level(&self) -> Option<&StoryLevel> {
        self.levels.get(self.current_level)
    }

    pub fn on_enemy_killed(&mut self) {
        self.enemies_killed += 1;
    }

    pub fn render_hud(&self) {
        if !self.active {
            if self.campaign_complete {
                self.render_victory_screen();
            }
            return;
        }

        if self.current_level >= self.levels.len() {
            return;
        }

        let level = &self.levels[self.current_level];
        let screen_w = screen_width();
        let screen_h = screen_height();

        if self.show_briefing {
            let bg_alpha = (self.briefing_timer / 6.0 * 220.0).min(220.0) as u8;
            draw_rectangle(
                0.0,
                screen_h * 0.2,
                screen_w,
                screen_h * 0.6,
                Color::from_rgba(0, 0, 0, bg_alpha),
            );

            draw_rectangle_lines(
                screen_w * 0.1,
                screen_h * 0.22,
                screen_w * 0.8,
                screen_h * 0.56,
                3.0,
                Color::from_rgba(255, 150, 0, bg_alpha),
            );

            let mission = format!("- MISSION {} -", self.current_level + 1);
            draw_text(
                &mission,
                screen_w / 2.0 - 100.0,
                screen_h * 0.32,
                28.0,
                Color::from_rgba(255, 150, 0, 255),
            );

            draw_text(
                &level.name,
                screen_w / 2.0 - 200.0,
                screen_h * 0.40,
                48.0,
                Color::from_rgba(255, 200, 100, 255),
            );

            draw_text(
                &level.description,
                screen_w / 2.0 - 150.0,
                screen_h * 0.48,
                24.0,
                Color::from_rgba(200, 200, 255, 255),
            );

            draw_text(
                &level.briefing,
                screen_w / 2.0 - 400.0,
                screen_h * 0.60,
                20.0,
                Color::from_rgba(255, 255, 255, 255),
            );

            let enemies_text = format!("TARGETS: {}", self.total_enemies);
            draw_text(
                &enemies_text,
                screen_w / 2.0 - 80.0,
                screen_h * 0.68,
                18.0,
                Color::from_rgba(200, 200, 200, 255),
            );

            let timer_text = format!("Starting in {:.0}...", self.briefing_timer.ceil());
            draw_text(
                &timer_text,
                screen_w / 2.0 - 100.0,
                screen_h * 0.74,
                20.0,
                Color::from_rgba(255, 255, 0, 255),
            );
            return;
        }

        let level_progress = format!(
            "MISSION {}/{}: {}",
            self.current_level + 1,
            self.levels.len(),
            level.name
        );
        draw_text(
            &level_progress,
            screen_w / 2.0 - 250.0,
            35.0,
            28.0,
            Color::from_rgba(255, 200, 100, 255),
        );

        let objective = if self.level_complete {
            ">>> PROCEED TO EXIT <<<".to_string()
        } else {
            format!("ELIMINATED: {}/{}", self.enemies_killed, self.total_enemies)
        };

        let obj_color = if self.level_complete {
            let pulse = ((crate::time::get_time() * 2.0).sin() * 0.3 + 0.7) as f32;
            Color::from_rgba(100, 255, 100, (pulse * 255.0) as u8)
        } else {
            WHITE
        };

        draw_text(&objective, screen_w / 2.0 - 140.0, 68.0, 22.0, obj_color);

        let boss_count = level
            .enemy_spawns
            .iter()
            .filter(|s| s.is_boss && !s.boss_name.is_empty())
            .count();
        if boss_count > 0 && !self.level_complete {
            let boss_text = if boss_count == 1 {
                "! BOSS DETECTED !"
            } else if boss_count == 2 {
                "!! DUAL BOSS ENCOUNTER !!"
            } else {
                "!!! MULTIPLE BOSSES !!!"
            };
            let pulse = ((crate::time::get_time() * 4.0).sin() * 0.5 + 0.5) as f32;
            let alpha = (100.0 + pulse * 155.0) as u8;
            let color = Color::from_rgba(255, 50, 50, alpha);
            draw_text(boss_text, screen_w / 2.0 - 150.0, 100.0, 20.0, color);
        }

        if self.current_level == 0 && self.enemies_killed == 0 && !self.show_briefing {
            draw_text(
                "Collect weapons and items from the arena to survive!",
                15.0,
                screen_height() - 90.0,
                16.0,
                Color::from_rgba(200, 220, 255, 220),
            );
        }
    }

    fn render_victory_screen(&self) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::from_rgba(0, 0, 0, 240));

        let pulse = ((crate::time::get_time() * 2.0).sin() * 0.2 + 0.8) as f32;

        let title = "CAMPAIGN COMPLETE";
        draw_text(
            title,
            screen_w / 2.0 - 280.0,
            screen_h * 0.25,
            56.0,
            Color::from_rgba(255, 200, 100, (pulse * 255.0) as u8),
        );

        let subtitle = "THE BREACH IS SEALED";
        draw_text(
            subtitle,
            screen_w / 2.0 - 240.0,
            screen_h * 0.35,
            40.0,
            Color::from_rgba(100, 255, 100, 255),
        );

        let story_line1 = "The dimensional tear collapses into nothingness.";
        draw_text(
            story_line1,
            screen_w / 2.0 - 320.0,
            screen_h * 0.48,
            24.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        let story_line2 = "The corrupted warriors fade back to their dimensions.";
        draw_text(
            story_line2,
            screen_w / 2.0 - 340.0,
            screen_h * 0.54,
            24.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        let story_line3 = "The Arenas stand silent once more.";
        draw_text(
            story_line3,
            screen_w / 2.0 - 260.0,
            screen_h * 0.60,
            24.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        let champion = "YOU ARE THE CHAMPION OF CHAMPIONS!";
        draw_text(
            champion,
            screen_w / 2.0 - 300.0,
            screen_h * 0.72,
            28.0,
            Color::from_rgba(255, 255, 100, 255),
        );

        let press_esc = "Press ESC to return to menu";
        draw_text(
            press_esc,
            screen_w / 2.0 - 180.0,
            screen_h * 0.85,
            20.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }

    pub fn render_exit_portal(&self, camera_x: f32, camera_y: f32) {
        if self.current_level >= self.levels.len() || self.show_briefing {
            return;
        }

        let level = &self.levels[self.current_level];
        let screen_x = level.exit_x - camera_x;
        let screen_y = level.exit_y - camera_y;

        if !self.level_complete {
            let marker_pulse = ((crate::time::get_time() * 2.0).sin() * 0.5 + 0.5) as f32;
            let marker_alpha = (100.0 + marker_pulse * 100.0) as u8;
            draw_circle(
                screen_x,
                screen_y,
                15.0,
                Color::from_rgba(255, 255, 100, marker_alpha),
            );
            draw_circle_lines(
                screen_x,
                screen_y,
                20.0,
                2.0,
                Color::from_rgba(255, 255, 0, marker_alpha),
            );
            return;
        }

        let time = crate::time::get_time() as f32;
        let pulse = (time * 3.0).sin() * 0.3 + 0.7;
        let radius = level.exit_radius * pulse;

        let rotation = time * 1.5;
        for i in 0..12 {
            let angle = rotation + (i as f32 * std::f32::consts::PI / 6.0);
            let r = radius * 1.1;
            let x = screen_x + angle.cos() * r;
            let y = screen_y + angle.sin() * r;
            draw_circle(x, y, 4.0, Color::from_rgba(100, 255, 255, 150));
        }

        draw_circle(
            screen_x,
            screen_y,
            radius,
            Color::from_rgba(100, 255, 100, 60),
        );
        draw_circle(
            screen_x,
            screen_y,
            radius * 0.75,
            Color::from_rgba(100, 255, 255, 90),
        );
        draw_circle(
            screen_x,
            screen_y,
            radius * 0.5,
            Color::from_rgba(255, 255, 100, 140),
        );
        draw_circle(
            screen_x,
            screen_y,
            radius * 0.25,
            Color::from_rgba(255, 255, 255, 180),
        );

        draw_circle_lines(
            screen_x,
            screen_y,
            radius,
            4.0,
            Color::from_rgba(100, 255, 100, 220),
        );
        draw_circle_lines(
            screen_x,
            screen_y,
            radius * 0.6,
            3.0,
            Color::from_rgba(255, 255, 100, 200),
        );

        for i in 0..8 {
            let angle = -time * 2.5 + (i as f32 * std::f32::consts::PI / 4.0);
            let x = screen_x + angle.cos() * radius * 0.4;
            let y = screen_y + angle.sin() * radius * 0.4;
            draw_circle(x, y, 7.0, Color::from_rgba(255, 255, 200, 200));
        }

        let exit_text = "EXIT";
        draw_text(
            exit_text,
            screen_x - 28.0,
            screen_y + 7.0,
            28.0,
            Color::from_rgba(0, 0, 0, 200),
        );
        draw_text(
            exit_text,
            screen_x - 30.0,
            screen_y + 5.0,
            28.0,
            Color::from_rgba(255, 255, 255, 255),
        );
    }
}
