use macroquad::prelude::*;

use super::map;

pub struct ItemIcons {
    pub health_small: Option<Texture2D>,
    pub health_medium: Option<Texture2D>,
    pub health_mega: Option<Texture2D>,
    pub armor_shard: Option<Texture2D>,
    pub armor_yellow: Option<Texture2D>,
    pub armor_red: Option<Texture2D>,
    pub weapon_shotgun: Option<Texture2D>,
    pub weapon_grenade: Option<Texture2D>,
    pub weapon_rocket: Option<Texture2D>,
    pub weapon_lightning: Option<Texture2D>,
    pub weapon_railgun: Option<Texture2D>,
    pub weapon_plasma: Option<Texture2D>,
    pub weapon_bfg: Option<Texture2D>,
    pub powerup_quad: Option<Texture2D>,
    pub powerup_regen: Option<Texture2D>,
    pub powerup_battle: Option<Texture2D>,
    pub powerup_flight: Option<Texture2D>,
    pub powerup_haste: Option<Texture2D>,
    pub powerup_invis: Option<Texture2D>,
}

impl ItemIcons {
    pub fn new() -> Self {
        ItemIcons {
            health_small: None,
            health_medium: None,
            health_mega: None,
            armor_shard: None,
            armor_yellow: None,
            armor_red: None,
            weapon_shotgun: None,
            weapon_grenade: None,
            weapon_rocket: None,
            weapon_lightning: None,
            weapon_railgun: None,
            weapon_plasma: None,
            weapon_bfg: None,
            powerup_quad: None,
            powerup_regen: None,
            powerup_battle: None,
            powerup_flight: None,
            powerup_haste: None,
            powerup_invis: None,
        }
    }

    pub async fn load_all(&mut self) {
        self.health_small = Self::load_icon("q3-resources/icons/iconh_green.png").await;
        self.health_medium = Self::load_icon("q3-resources/icons/iconh_yellow.png").await;
        self.health_mega = Self::load_icon("q3-resources/icons/iconh_mega.png").await;
        self.armor_shard = Self::load_icon("q3-resources/icons/iconr_shard.png").await;
        self.armor_yellow = Self::load_icon("q3-resources/icons/iconr_yellow.png").await;
        self.armor_red = Self::load_icon("q3-resources/icons/iconr_red.png").await;
        self.weapon_shotgun = Self::load_icon("q3-resources/icons/iconw_shotgun.png").await;
        self.weapon_grenade = Self::load_icon("q3-resources/icons/iconw_grenade.png").await;
        self.weapon_rocket = Self::load_icon("q3-resources/icons/iconw_rocket.png").await;
        self.weapon_lightning = Self::load_icon("q3-resources/icons/iconw_lightning.png").await;
        self.weapon_railgun = Self::load_icon("q3-resources/icons/iconw_railgun.png").await;
        self.weapon_plasma = Self::load_icon("q3-resources/icons/iconw_plasma.png").await;
        self.weapon_bfg = Self::load_icon("q3-resources/icons/iconw_bfg.png").await;
        self.powerup_quad = Self::load_icon("q3-resources/icons/quad.png").await;
        self.powerup_regen = Self::load_icon("q3-resources/icons/regen.png").await;
        self.powerup_battle = Self::load_icon("q3-resources/icons/envirosuit.png").await;
        self.powerup_flight = Self::load_icon("q3-resources/icons/flight.png").await;
        self.powerup_haste = Self::load_icon("q3-resources/icons/haste.png").await;
        self.powerup_invis = Self::load_icon("q3-resources/icons/invis.png").await;
    }

    async fn load_icon(path: &str) -> Option<Texture2D> {
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Linear);
                Some(texture)
            }
            Err(e) => {
                println!("[Editor] Failed to load icon {}: {}", path, e);
                None
            }
        }
    }

    pub fn get_icon(&self, item_type: &map::ItemType) -> Option<&Texture2D> {
        match item_type {
            map::ItemType::Health25 => self.health_small.as_ref(),
            map::ItemType::Health50 => self.health_medium.as_ref(),
            map::ItemType::Health100 => self.health_mega.as_ref(),
            map::ItemType::Armor50 => self.armor_yellow.as_ref(),
            map::ItemType::Armor100 => self.armor_red.as_ref(),
            map::ItemType::Shotgun => self.weapon_shotgun.as_ref(),
            map::ItemType::GrenadeLauncher => self.weapon_grenade.as_ref(),
            map::ItemType::RocketLauncher => self.weapon_rocket.as_ref(),
            map::ItemType::LightningGun => self.weapon_lightning.as_ref(),
            map::ItemType::Railgun => self.weapon_railgun.as_ref(),
            map::ItemType::Plasmagun => self.weapon_plasma.as_ref(),
            map::ItemType::BFG => self.weapon_bfg.as_ref(),
            map::ItemType::Quad => self.powerup_quad.as_ref(),
            map::ItemType::Regen => self.powerup_regen.as_ref(),
            map::ItemType::Battle => self.powerup_battle.as_ref(),
            map::ItemType::Flight => self.powerup_flight.as_ref(),
            map::ItemType::Haste => self.powerup_haste.as_ref(),
            map::ItemType::Invis => self.powerup_invis.as_ref(),
        }
    }
}
