use sas::*;
mod menu;
mod game_loop;
mod weapon_handler;
mod bot_handler;
mod hud_scoreboard;
mod app;

use app::App;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "SAS III - Still Alive Somehow??".to_string(),
        window_resizable: true,
        fullscreen: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    show_mouse(false);
    
    let mut app = App::new();
    app.initialize().await;
    app.run().await;
}
