use crate::game::weapon::Weapon;
use crate::game::award::AwardType;

#[derive(Clone, Debug)]
pub enum AudioEvent {
    WeaponFire { weapon: Weapon, x: f32, has_quad: bool },
    WeaponSwitch,
    Explosion { x: f32 },
    RailgunHit { x: f32 },
    PlayerPain { health: i32, x: f32, model: String },
    PlayerDeath { x: f32, model: String },
    PlayerGib { x: f32 },
    PlayerJump { x: f32, model: String },
    PlayerLand { x: f32 },
    PlayerHit { damage: i32 },
    ItemPickup { x: f32 },
    ArmorPickup { x: f32 },
    WeaponPickup { x: f32 },
    PowerupPickup { x: f32 },
    QuadDamage,
    TeleportIn { x: f32 },
    TeleportOut { x: f32 },
    JumpPad { x: f32 },
    GrenadeBounce { x: f32 },
    Award { award_type: AwardType },
    TimeAnnouncement { announcement: String },
    LeadChange { announcement: String },
    MatchStart,
    MatchEnd,
}

pub struct AudioEventQueue {
    pub events: Vec<AudioEvent>,
}

impl AudioEventQueue {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: AudioEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<AudioEvent> {
        self.events.drain(..).collect()
    }
}

