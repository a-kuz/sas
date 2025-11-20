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
    let icon_data = include_bytes!("../assets/logo-alpha.png");
    let icon = match image::load_from_memory(icon_data) {
        Ok(img) => {
            let small_img = img.resize_exact(16, 16, image::imageops::FilterType::Lanczos3);
            let medium_img = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
            let big_img = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3);
            
            let small_rgba = small_img.to_rgba8();
            let medium_rgba = medium_img.to_rgba8();
            let big_rgba = big_img.to_rgba8();
            
            let mut small = [0u8; 1024];
            let mut medium = [0u8; 4096];
            let mut big = [0u8; 16384];
            
            small.copy_from_slice(&small_rgba);
            medium.copy_from_slice(&medium_rgba);
            big.copy_from_slice(&big_rgba);
            
            Some(miniquad::conf::Icon {
                small,
                medium,
                big,
            })
        }
        Err(_) => None,
    };
    
    Conf {
        window_title: "SAS III - Still Alive Somehow??".to_string(),
        window_resizable: true,
        fullscreen: true,
        high_dpi: true,
        icon,
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
