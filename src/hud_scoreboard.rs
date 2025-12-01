use crate::game::GameState;
use crate::input::Input;
use crate::render;
use crate::render::Camera;
use macroquad::prelude::*;

pub struct HudScoreboard;

impl HudScoreboard {
    pub fn render_hud(game_state: &GameState) {
        let leader_frags = game_state
            .players
            .iter()
            .map(|p| p.frags)
            .max()
            .unwrap_or(0);

        if let Some(ref story) = game_state.story_mode {
            story.render_hud();
        }

        if game_state.is_local_multiplayer {
            Self::render_local_multiplayer_hud(game_state, leader_frags);
        } else {
            Self::render_single_player_hud(game_state);
        }
    }

    fn render_local_multiplayer_hud(game_state: &GameState, leader_frags: i32) {
        if let Some(player1) = game_state.players.get(0) {
            let competitor_frags = game_state.players.get(1).map(|p| p.frags).unwrap_or(0);
            render::draw_hud(
                player1.health,
                player1.armor,
                player1.ammo[player1.weapon as usize],
                player1.weapon.name(),
                player1.frags,
                player1.weapon as u8,
                competitor_frags,
                &player1.has_weapon,
                &player1.ammo,
                game_state.match_time,
                game_state.time_limit,
            );
        }

        if let Some(player2) = game_state.players.get(1) {
            render::draw_hud_player2(
                player2.health,
                player2.armor,
                player2.ammo[player2.weapon as usize],
                player2.weapon.name(),
                player2.frags,
                player2.weapon as u8,
                leader_frags,
                &player2.has_weapon,
                &player2.ammo,
            );
        }
    }

    fn render_single_player_hud(game_state: &GameState) {
        if let Some(player) = game_state.players.get(0) {
            let competitor_frags = game_state
                .players
                .iter()
                .filter(|p| p.id != player.id && !p.dead)
                .map(|p| p.frags)
                .max()
                .unwrap_or(0);

            render::draw_hud(
                player.health,
                player.armor,
                player.ammo[player.weapon as usize],
                player.weapon.name(),
                player.frags,
                player.weapon as u8,
                competitor_frags,
                &player.has_weapon,
                &player.ammo,
                game_state.match_time,
                game_state.time_limit,
            );
        }
    }

    pub fn render_crosshair(game_state: &GameState, camera: &Camera, input: &Input) {
        let crosshair_size = crate::cvar::get_cvar_float("cg_crosshairSize");
        if crosshair_size <= 0.0 {
            return;
        }

        if !game_state.is_local_multiplayer {
            let player_index = if game_state.is_multiplayer {
                if let Some(ref network_client) = game_state.network_client {
                    if let Some(player_id) = network_client.player_id() {
                        game_state
                            .players
                            .iter()
                            .position(|p| p.id == player_id)
                            .unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            };

            if let Some(player) = game_state.players.get(player_index) {
                if player.dead {
                    return;
                }

                let angle = if game_state.is_multiplayer {
                    player.angle
                } else {
                    input.aim_angle
                };

                let flip = angle.abs() > std::f32::consts::PI / 2.0;
                let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                let mut rel_angle = angle - base_dir;
                while rel_angle > std::f32::consts::PI {
                    rel_angle -= 2.0 * std::f32::consts::PI;
                }
                while rel_angle < -std::f32::consts::PI {
                    rel_angle += 2.0 * std::f32::consts::PI;
                }
                let pitch = rel_angle;

                let weapon_model = game_state.weapon_model_cache.get(player.weapon);
                let (barrel_x, barrel_y) =
                    if let Some(player_model) = game_state.model_cache.get(&player.model) {
                        player_model.get_barrel_position(
                            player.x,
                            player.y,
                            flip,
                            pitch,
                            angle,
                            player.lower_frame,
                            player.upper_frame,
                            weapon_model,
                        )
                    } else {
                        let weapon_offset = 20.0;
                        (
                            player.x + angle.cos() * weapon_offset,
                            player.y - 24.0 + angle.sin() * weapon_offset,
                        )
                    };

                render::draw_crosshair(barrel_x, barrel_y, camera.x, camera.y, angle);
            }
        }
    }

    pub fn render_crosshair_local_mp(
        game_state: &GameState,
        camera: &Camera,
        local_mp_input: &crate::input::LocalMultiplayerInput,
    ) {
        let crosshair_size = crate::cvar::get_cvar_float("cg_crosshairSize");
        if crosshair_size <= 0.0 {
            return;
        }

        if game_state.is_local_multiplayer {
            for (player_idx, player_input) in [&local_mp_input.player1, &local_mp_input.player2]
                .iter()
                .enumerate()
            {
                if let Some(player) = game_state.players.get(player_idx) {
                    if player.dead {
                        continue;
                    }

                    let angle = player_input.aim_angle;

                    let flip = angle.abs() > std::f32::consts::PI / 2.0;
                    let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                    let mut rel_angle = angle - base_dir;
                    while rel_angle > std::f32::consts::PI {
                        rel_angle -= 2.0 * std::f32::consts::PI;
                    }
                    while rel_angle < -std::f32::consts::PI {
                        rel_angle += 2.0 * std::f32::consts::PI;
                    }
                    let pitch = rel_angle;

                    let weapon_model = game_state.weapon_model_cache.get(player.weapon);
                    let (barrel_x, barrel_y) =
                        if let Some(player_model) = game_state.model_cache.get(&player.model) {
                            player_model.get_barrel_position(
                                player.x,
                                player.y,
                                flip,
                                pitch,
                                angle,
                                player.lower_frame,
                                player.upper_frame,
                                weapon_model,
                            )
                        } else {
                            let weapon_offset = 20.0;
                            (
                                player.x + angle.cos() * weapon_offset,
                                player.y - 24.0 + angle.sin() * weapon_offset,
                            )
                        };

                    render::draw_crosshair(barrel_x, barrel_y, camera.x, camera.y, angle);
                }
            }
        }
    }

    pub fn render_scoreboard(game_state: &GameState) {
        if is_key_down(KeyCode::Tab) {
            let board_width = 460.0;
            let board_height = 300.0;
            let board_x = screen_width() / 2.0 - board_width / 2.0;
            let board_y = screen_height() / 2.0 - board_height / 2.0;

            draw_rectangle(
                board_x,
                board_y,
                board_width,
                board_height,
                Color::from_rgba(0, 0, 0, 200),
            );
            draw_rectangle_lines(board_x, board_y, board_width, board_height, 2.0, WHITE);

            draw_text("SCOREBOARD", board_x + 120.0, board_y + 30.0, 24.0, YELLOW);
            draw_text("Player", board_x + 20.0, board_y + 60.0, 20.0, WHITE);
            draw_text("Frags", board_x + 200.0, board_y + 60.0, 20.0, WHITE);
            draw_text("Deaths", board_x + 260.0, board_y + 60.0, 20.0, WHITE);
            draw_text("Awards", board_x + 330.0, board_y + 60.0, 20.0, WHITE);

            let mut sorted_players = game_state.players.clone();
            sorted_players.sort_by(|a, b| b.frags.cmp(&a.frags));

            for (i, player) in sorted_players.iter().enumerate() {
                let y = board_y + 90.0 + (i as f32 * 30.0);
                let color = if player.is_bot { RED } else { GREEN };
                draw_text(&player.name, board_x + 20.0, y, 18.0, color);
                draw_text(
                    &format!("{}", player.frags),
                    board_x + 210.0,
                    y,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("{}", player.deaths),
                    board_x + 270.0,
                    y,
                    18.0,
                    WHITE,
                );

                let mut award_x = board_x + 330.0;
                let icon_size = 20.0;
                let icon_y = y - 16.0;

                if player.excellent_count > 0 {
                    if let Some(excellent_tex) = game_state.award_icon_cache.excellent.as_ref() {
                        draw_texture_ex(
                            excellent_tex,
                            award_x,
                            icon_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(icon_size, icon_size)),
                                ..Default::default()
                            },
                        );
                        award_x += icon_size + 2.0;
                        draw_text(
                            &format!("{}", player.excellent_count),
                            award_x,
                            y,
                            16.0,
                            Color::from_rgba(255, 220, 100, 255),
                        );
                        award_x += 25.0;
                    }
                }

                if player.impressive_count > 0 {
                    if let Some(impressive_tex) = game_state.award_icon_cache.impressive.as_ref() {
                        draw_texture_ex(
                            impressive_tex,
                            award_x,
                            icon_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(icon_size, icon_size)),
                                ..Default::default()
                            },
                        );
                        award_x += icon_size + 2.0;
                        draw_text(
                            &format!("{}", player.impressive_count),
                            award_x,
                            y,
                            16.0,
                            Color::from_rgba(255, 220, 100, 255),
                        );
                    }
                }
            }
        }
    }

    pub fn render_debug_info(
        game_state: &GameState,
        fps_display: i32,
        perf_mode: bool,
        dt: f32,
        frame_time: f32,
    ) {
        if fps_display > 0 {
            render::draw_text_outlined(
                &format!("FPS: {}", fps_display),
                10.0,
                22.0,
                18.0,
                Color::from_rgba(200, 200, 210, 255),
            );
        }

        if game_state.debug_md3 {
            render::draw_text_outlined(
                "MD3 DEBUG ON (F7)",
                10.0,
                44.0,
                18.0,
                Color::from_rgba(255, 200, 120, 255),
            );
        }

        if perf_mode {
            render::draw_text_outlined(
                &format!(
                    "PERF (F9) dt={:.1}ms frame={:.1}ms",
                    dt * 1000.0,
                    frame_time
                ),
                10.0,
                42.0,
                16.0,
                Color::from_rgba(255, 200, 100, 255),
            );
        }
    }
}
