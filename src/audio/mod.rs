pub mod events;

use events::AudioEvent;
use rodio::{Decoder, OutputStream, Sink};
use std::collections::HashMap;

pub struct AudioSystem {
    sounds: HashMap<String, Vec<u8>>,
    enabled: bool,
    _stream: OutputStream,
    _stream_handle: rodio::OutputStreamHandle,
}

impl AudioSystem {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap_or_else(|_| {
            eprintln!("[AUDIO] Failed to create audio output stream");
            panic!("Audio initialization failed");
        });

        Self {
            sounds: HashMap::new(),
            enabled: true,
            _stream: stream,
            _stream_handle: stream_handle,
        }
    }

    pub async fn load_sound(&mut self, name: &str, path: &str) {
        if let Ok(file) = std::fs::read(path) {
            self.sounds.insert(name.to_string(), file);
        }
    }

    pub async fn load_player_sounds(&mut self, model: &str) {
        let base_path = format!("q3-resources/sound/player/{}", model);
        self.load_sound(
            &format!("pain_25_{}", model),
            &format!("{}/pain25_1.wav", base_path),
        )
        .await;
        self.load_sound(
            &format!("pain_50_{}", model),
            &format!("{}/pain50_1.wav", base_path),
        )
        .await;
        self.load_sound(
            &format!("pain_75_{}", model),
            &format!("{}/pain75_1.wav", base_path),
        )
        .await;
        self.load_sound(
            &format!("pain_100_{}", model),
            &format!("{}/pain100_1.wav", base_path),
        )
        .await;
        self.load_sound(
            &format!("death_{}", model),
            &format!("{}/death1.wav", base_path),
        )
        .await;
        self.load_sound(
            &format!("jump_{}", model),
            &format!("{}/jump1.wav", base_path),
        )
        .await;
    }

    pub fn play(&self, name: &str, volume: f32) {
        if !self.enabled {
            return;
        }

        if let Some(sound_data) = self.sounds.get(name) {
            let cursor = std::io::Cursor::new(sound_data.clone());
            if let Ok(source) = Decoder::new(cursor) {
                if let Ok(sink) = Sink::try_new(&self._stream_handle) {
                    sink.set_volume(volume);
                    sink.append(source);
                    sink.detach();
                } else {
                    eprintln!("[AUDIO] Failed to create sink for sound: {}", name);
                }
            }
        }
    }

    pub fn play_positional(&self, name: &str, volume: f32, x: f32, listener_x: f32) {
        if !self.enabled {
            return;
        }

        let distance = (x - listener_x).abs();
        let max_distance = 800.0;

        if distance > max_distance {
            return;
        }

        let distance_volume = 1.0 - (distance / max_distance).min(1.0);
        let final_volume = volume * distance_volume;

        if final_volume > 0.01 {
            self.play(name, final_volume);
        }
    }

    pub fn process_event(&self, event: &AudioEvent, listener_x: f32) {
        use crate::game::award::AwardType;
        use crate::game::weapon::Weapon;

        match event {
            AudioEvent::WeaponFire {
                weapon,
                x,
                has_quad,
            } => {
                if *has_quad {
                    self.play("quad_fire", 0.8);
                }

                let sound_name = match weapon {
                    Weapon::Gauntlet => "gauntlet",
                    Weapon::MachineGun => "mg_fire",
                    Weapon::Shotgun => "shotgun_fire",
                    Weapon::GrenadeLauncher => "grenade_fire",
                    Weapon::RocketLauncher => "rocket_fire",
                    Weapon::Lightning => "lightning_fire",
                    Weapon::Railgun => "railgun_fire",
                    Weapon::Plasmagun => "plasma_fire",
                    Weapon::BFG => "bfg_fire",
                };
                let volume = match weapon {
                    Weapon::MachineGun => 0.3,
                    Weapon::Lightning => 0.3,
                    Weapon::Gauntlet => 0.4,
                    Weapon::Plasmagun => 0.4,
                    Weapon::Shotgun => 0.5,
                    Weapon::GrenadeLauncher => 0.5,
                    Weapon::RocketLauncher => 0.6,
                    Weapon::Railgun => 0.7,
                    Weapon::BFG => 0.8,
                };
                self.play_positional(sound_name, volume, *x, listener_x);
            }
            AudioEvent::WeaponSwitch => self.play("weapon_switch", 0.4),
            AudioEvent::Explosion { x } => {
                self.play_positional("rocket_explode", 0.7, *x, listener_x);
            }
            AudioEvent::PlayerPain { health, x, model } => {
                let sound_base = if *health < 25 {
                    "pain_25"
                } else if *health < 50 {
                    "pain_50"
                } else if *health < 75 {
                    "pain_75"
                } else {
                    "pain_100"
                };
                let sound_name = format!("{}_{}", sound_base, model);
                self.play_positional(&sound_name, 0.5, *x, listener_x);
            }
            AudioEvent::PlayerDeath { x, model } => {
                let sound_name = format!("death_{}", model);
                self.play_positional(&sound_name, 0.6, *x, listener_x);
            }
            AudioEvent::PlayerGib { x } => {
                self.play_positional("gib", 0.7, *x, listener_x);
            }
            AudioEvent::PlayerJump { x, model } => {
                let sound_name = format!("jump_{}", model);
                self.play_positional(&sound_name, 0.3, *x, listener_x);
            }
            AudioEvent::PlayerLand { x } => {
                self.play_positional("land", 0.4, *x, listener_x);
            }
            AudioEvent::PlayerHit { damage } => {
                let sound_name = if *damage >= 100 {
                    "hit_100"
                } else if *damage >= 50 {
                    "hit_75"
                } else if *damage >= 25 {
                    "hit_50"
                } else {
                    "hit_25"
                };
                self.play(sound_name, 0.5);
            }
            AudioEvent::ItemPickup { x } => {
                self.play_positional("item_pickup", 0.5, *x, listener_x);
            }
            AudioEvent::ArmorPickup { x } => {
                self.play_positional("armor_pickup", 0.5, *x, listener_x);
            }
            AudioEvent::WeaponPickup { x } => {
                self.play_positional("weapon_pickup", 0.5, *x, listener_x);
            }
            AudioEvent::PowerupPickup { x } => {
                self.play_positional("powerup_pickup", 0.6, *x, listener_x);
            }
            AudioEvent::QuadDamage => {
                self.play("quad_damage", 0.9);
            }
            AudioEvent::TeleportIn { x } => {
                self.play_positional("teleport_in", 0.6, *x, listener_x);
            }
            AudioEvent::TeleportOut { x } => {
                self.play_positional("teleport_out", 0.6, *x, listener_x);
            }
            AudioEvent::JumpPad { x } => {
                self.play_positional("jumppad", 0.6, *x, listener_x);
            }
            AudioEvent::RailgunHit { x } => {
                self.play_positional("railgun_hit", 0.6, *x, listener_x);
            }
            AudioEvent::GrenadeBounce { x } => {
                let sound_name = if crate::compat_rand::gen_range(0, 2) == 0 {
                    "grenade_bounce1"
                } else {
                    "grenade_bounce2"
                };
                self.play_positional(sound_name, 0.4, *x, listener_x);
            }
            AudioEvent::Award { award_type } => {
                let sound_name = match award_type {
                    AwardType::Excellent => "excellent",
                    AwardType::Impressive => "impressive",
                    AwardType::Humiliation => "humiliation",
                    AwardType::Perfect => "perfect",
                    AwardType::Accuracy => "accuracy",
                };
                self.play(sound_name, 0.8);
            }
            AudioEvent::TimeAnnouncement { announcement } => {
                self.play(announcement, 0.8);
            }
            AudioEvent::LeadChange { announcement } => {
                println!("[AUDIO] Playing LeadChange sound: {}", announcement);
                self.play(announcement, 0.8);
            }
            AudioEvent::MatchStart => {
                self.play("fight", 0.8);
            }
            AudioEvent::MatchEnd => {
                self.play("match_end", 0.8);
            }
        }
    }
}

pub async fn init_audio() -> AudioSystem {
    let mut audio = AudioSystem::new();

    audio
        .load_sound(
            "mg_fire",
            "q3-resources/sound/weapons/machinegun/machgf1b.wav",
        )
        .await;
    audio
        .load_sound(
            "shotgun_fire",
            "q3-resources/sound/weapons/shotgun/sshotf1b.wav",
        )
        .await;
    audio
        .load_sound(
            "rocket_fire",
            "q3-resources/sound/weapons/rocket/rocklf1a.wav",
        )
        .await;
    audio
        .load_sound(
            "rocket_fly",
            "q3-resources/sound/weapons/rocket/rockfly.wav",
        )
        .await;
    audio
        .load_sound(
            "rocket_explode",
            "q3-resources/sound/weapons/rocket/rocklx1a.wav",
        )
        .await;
    audio
        .load_sound(
            "grenade_fire",
            "q3-resources/sound/weapons/grenade/grenlf1a.wav",
        )
        .await;
    audio
        .load_sound(
            "grenade_bounce1",
            "q3-resources/sound/weapons/grenade/hgrenb1a.wav",
        )
        .await;
    audio
        .load_sound(
            "grenade_bounce2",
            "q3-resources/sound/weapons/grenade/hgrenb2a.wav",
        )
        .await;
    audio
        .load_sound(
            "grenade_explode",
            "q3-resources/sound/weapons/rocket/rocklx1a.wav",
        )
        .await;
    audio
        .load_sound(
            "plasma_fire",
            "q3-resources/sound/weapons/plasma/hyprbf1a.wav",
        )
        .await;
    audio
        .load_sound(
            "plasma_explode",
            "q3-resources/sound/weapons/plasma/plasmx1a.wav",
        )
        .await;
    audio
        .load_sound(
            "railgun_fire",
            "q3-resources/sound/weapons/railgun/railgf1a.wav",
        )
        .await;
    audio
        .load_sound(
            "railgun_hit",
            "q3-resources/sound/weapons/plasma/plasmx1a.wav",
        )
        .await;
    audio
        .load_sound(
            "lightning_fire",
            "q3-resources/sound/weapons/lightning/lg_hum.wav",
        )
        .await;
    audio
        .load_sound("bfg_fire", "q3-resources/sound/weapons/bfg/bfg_fire.wav")
        .await;
    audio
        .load_sound(
            "bfg_explode",
            "q3-resources/sound/weapons/rocket/rocklx1a.wav",
        )
        .await;
    audio
        .load_sound("gauntlet", "q3-resources/sound/weapons/melee/fstatck.wav")
        .await;

    audio
        .load_sound("land", "q3-resources/sound/player/land1.wav")
        .await;
    audio
        .load_sound("gib", "q3-resources/sound/player/gibsplt1.wav")
        .await;

    audio
        .load_sound("footstep1", "q3-resources/sound/player/footsteps/step1.wav")
        .await;
    audio
        .load_sound("footstep2", "q3-resources/sound/player/footsteps/step2.wav")
        .await;
    audio
        .load_sound("footstep3", "q3-resources/sound/player/footsteps/step3.wav")
        .await;
    audio
        .load_sound("footstep4", "q3-resources/sound/player/footsteps/step4.wav")
        .await;

    audio
        .load_sound("weapon_switch", "q3-resources/sound/weapons/change.wav")
        .await;
    audio
        .load_sound("no_ammo", "q3-resources/sound/weapons/noammo.wav")
        .await;

    audio
        .load_sound("item_pickup", "q3-resources/sound/items/n_health.wav")
        .await;
    audio
        .load_sound("armor_pickup", "q3-resources/sound/items/s_health.wav")
        .await;
    audio
        .load_sound("weapon_pickup", "q3-resources/sound/misc/w_pkup.wav")
        .await;
    audio
        .load_sound("powerup_pickup", "q3-resources/sound/items/protect.wav")
        .await;
    audio
        .load_sound("quad_damage", "q3-resources/sound/items/quaddamage.wav")
        .await;
    audio
        .load_sound("quad_fire", "q3-resources/sound/items/quaddamage_fire.wav")
        .await;

    audio
        .load_sound("teleport_in", "q3-resources/sound/world/telein.wav")
        .await;
    audio
        .load_sound("teleport_out", "q3-resources/sound/world/teleout.wav")
        .await;
    audio
        .load_sound("jumppad", "q3-resources/sound/world/jumppad.wav")
        .await;

    audio
        .load_sound("hit_25", "q3-resources/sound/feedback/hit25.wav")
        .await;
    audio
        .load_sound("hit_50", "q3-resources/sound/feedback/hit50.wav")
        .await;
    audio
        .load_sound("hit_75", "q3-resources/sound/feedback/hit75.wav")
        .await;
    audio
        .load_sound("hit_100", "q3-resources/sound/feedback/hit100.wav")
        .await;

    audio
        .load_sound("excellent", "q3-resources/sound/feedback/excellent.wav")
        .await;
    audio
        .load_sound("impressive", "q3-resources/sound/feedback/impressive.wav")
        .await;
    audio
        .load_sound("humiliation", "q3-resources/sound/feedback/humiliation.wav")
        .await;
    audio
        .load_sound("perfect", "q3-resources/sound/feedback/perfect.wav")
        .await;
    audio
        .load_sound("accuracy", "q3-resources/sound/feedback/accuracy.wav")
        .await;

    audio
        .load_sound("fight", "q3-resources/sound/feedback/fight.wav")
        .await;
    audio
        .load_sound("5_minute", "q3-resources/sound/feedback/5_minute.wav")
        .await;
    audio
        .load_sound("1_minute", "q3-resources/sound/feedback/1_minute.wav")
        .await;
    audio
        .load_sound("prepare", "q3-resources/sound/feedback/prepare.wav")
        .await;

    audio
        .load_sound(
            "taken_the_lead",
            "q3-resources/sound/feedback/takenlead.wav",
        )
        .await;
    audio
        .load_sound(
            "tied_for_the_lead",
            "q3-resources/sound/feedback/tiedlead.wav",
        )
        .await;
    audio
        .load_sound("lost_the_lead", "q3-resources/sound/feedback/lostlead.wav")
        .await;

    audio
}
