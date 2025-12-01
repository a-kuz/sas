use macroquad::prelude::Color;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    static ref CVAR_REGISTRY: Arc<RwLock<CvarRegistry>> = Arc::new(RwLock::new(CvarRegistry::new()));
}

#[derive(Clone)]
pub struct Cvar {
    pub name: String,
    pub value: String,
    pub default_value: String,
    pub flags: u32,
    pub modified: bool,
}

pub const CVAR_ARCHIVE: u32 = 1;
pub const CVAR_USERINFO: u32 = 2;
pub const CVAR_SERVERINFO: u32 = 4;
pub const CVAR_SYSTEMINFO: u32 = 8;
pub const CVAR_INIT: u32 = 16;
pub const CVAR_LATCH: u32 = 32;
pub const CVAR_ROM: u32 = 64;
pub const CVAR_CHEAT: u32 = 128;

impl Cvar {
    pub fn new(name: &str, default_value: &str, flags: u32) -> Self {
        Self {
            name: name.to_string(),
            value: default_value.to_string(),
            default_value: default_value.to_string(),
            flags,
            modified: false,
        }
    }

    pub fn get_string(&self) -> String {
        self.value.clone()
    }

    pub fn get_integer(&self) -> i32 {
        self.value
            .parse::<i32>()
            .unwrap_or_else(|_| self.default_value.parse::<i32>().unwrap_or(0))
    }

    pub fn get_float(&self) -> f32 {
        self.value
            .parse::<f32>()
            .unwrap_or_else(|_| self.default_value.parse::<f32>().unwrap_or(0.0))
    }

    pub fn get_bool(&self) -> bool {
        self.get_integer() != 0
    }

    pub fn set(&mut self, value: &str) {
        if self.value != value {
            self.value = value.to_string();
            self.modified = true;
        }
    }

    pub fn set_integer(&mut self, value: i32) {
        self.set(&value.to_string());
    }

    pub fn set_float(&mut self, value: f32) {
        self.set(&value.to_string());
    }
}

pub struct CvarRegistry {
    cvars: HashMap<String, Cvar>,
}

impl CvarRegistry {
    pub fn new() -> Self {
        Self {
            cvars: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, default_value: &str, flags: u32) {
        let name_lower = name.to_lowercase();
        if !self.cvars.contains_key(&name_lower) {
            self.cvars
                .insert(name_lower.clone(), Cvar::new(name, default_value, flags));
        }
    }

    pub fn get(&self, name: &str) -> Option<Cvar> {
        let name_lower = name.to_lowercase();
        self.cvars.get(&name_lower).cloned()
    }

    pub fn set(&mut self, name: &str, value: &str) {
        let name_lower = name.to_lowercase();
        if let Some(cvar) = self.cvars.get_mut(&name_lower) {
            cvar.set(value);
        }
    }

    pub fn get_all_names(&self) -> Vec<String> {
        self.cvars.keys().cloned().collect()
    }

    pub fn find_matches(&self, prefix: &str) -> Vec<String> {
        let prefix_lower = prefix.to_lowercase();
        let mut matches: Vec<String> = self
            .cvars
            .keys()
            .filter(|name| name.starts_with(&prefix_lower))
            .cloned()
            .collect();
        matches.sort();
        matches
    }
}

pub fn register_cvar(name: &str, default_value: &str, flags: u32) {
    CVAR_REGISTRY
        .write()
        .unwrap()
        .register(name, default_value, flags);
}

pub fn get_cvar(name: &str) -> Option<Cvar> {
    CVAR_REGISTRY.read().unwrap().get(name)
}

pub fn set_cvar(name: &str, value: &str) {
    CVAR_REGISTRY.write().unwrap().set(name, value);
    save_config();
}

pub fn get_cvar_string(name: &str) -> String {
    get_cvar(name).map(|c| c.get_string()).unwrap_or_default()
}

pub fn get_cvar_integer(name: &str) -> i32 {
    get_cvar(name).map(|c| c.get_integer()).unwrap_or(0)
}

pub fn get_cvar_float(name: &str) -> f32 {
    get_cvar(name).map(|c| c.get_float()).unwrap_or(0.0)
}

pub fn get_cvar_bool(name: &str) -> bool {
    get_cvar(name).map(|c| c.get_bool()).unwrap_or(false)
}

pub fn find_cvar_matches(prefix: &str) -> Vec<String> {
    CVAR_REGISTRY.read().unwrap().find_matches(prefix)
}

pub fn init_default_cvars() {
    register_cvar("cg_drawFPS", "1", CVAR_ARCHIVE);
    register_cvar("cg_drawGun", "1", CVAR_ARCHIVE);
    register_cvar("cg_crosshairSize", "24", CVAR_ARCHIVE);

    register_cvar("net_showPackets", "0", CVAR_ARCHIVE);
    register_cvar("net_showDrop", "0", CVAR_ARCHIVE);
    register_cvar("net_showSync", "0", CVAR_ARCHIVE);
    register_cvar("net_showPhysics", "0", CVAR_ARCHIVE);
    register_cvar("net_showCollision", "0", CVAR_ARCHIVE);
    register_cvar("net_drawPrediction", "0", CVAR_ARCHIVE);
    register_cvar("cg_simpleItems", "0", CVAR_ARCHIVE);
    register_cvar("r_gamma", "1.0", CVAR_ARCHIVE);
    register_cvar("r_railWidth", "16", CVAR_ARCHIVE);
    register_cvar("cg_shadows", "1", CVAR_ARCHIVE);
    register_cvar("r_dynamiclight", "1", CVAR_ARCHIVE);
    register_cvar("cg_model", "sarge", CVAR_ARCHIVE);
    register_cvar("cg_model2", "visor", CVAR_ARCHIVE);
    register_cvar("sensitivity", "400.0", CVAR_ARCHIVE);
    register_cvar("m_pitch", "-1.0", CVAR_ARCHIVE);
    register_cvar("m_yaw", "-1.0", CVAR_ARCHIVE);
    register_cvar("cl_yawspeed", "1.0", CVAR_ARCHIVE);
    register_cvar("m_grab", "1", CVAR_ARCHIVE);
    register_cvar("m_show_cursor", "0", CVAR_ARCHIVE);

    register_cvar("cl_timeNudge", "0", CVAR_ARCHIVE);
    register_cvar("cl_autoNudge", "0", CVAR_ARCHIVE);

    load_config();
}

pub fn load_config() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(content) = std::fs::read_to_string("sas_config.cfg") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("//") {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "seta" {
                    set_cvar(parts[1], parts[2..].join(" ").trim_matches('"'));
                }
            }
        }
    }
}

pub fn save_config() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let registry = CVAR_REGISTRY.read().unwrap();
        let mut lines = Vec::new();

        for (name, cvar) in &registry.cvars {
            if (cvar.flags & CVAR_ARCHIVE) != 0 {
                lines.push(format!("seta {} \"{}\"", name, cvar.value));
            }
        }

        lines.sort();
        let content = lines.join("\n") + "\n";
        let _ = std::fs::write("sas_config.cfg", content);
    }
}

pub fn apply_gamma(mut color: Color) -> Color {
    let gamma = get_cvar_float("r_gamma");
    if (gamma - 1.0).abs() < 0.01 {
        return color;
    }

    let gamma_clamped = gamma.max(0.5).min(3.0);
    color.r = color.r.powf(1.0 / gamma_clamped);
    color.g = color.g.powf(1.0 / gamma_clamped);
    color.b = color.b.powf(1.0 / gamma_clamped);
    color
}
