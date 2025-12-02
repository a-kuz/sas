use std::collections::HashMap;

const MAX_HISTORY: usize = 60;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static SHADER_COUNTS: std::cell::RefCell<HashMap<String, usize>> = std::cell::RefCell::new(HashMap::new());
    static SHADER_DISPLAY: std::cell::RefCell<HashMap<String, usize>> = std::cell::RefCell::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct DrawCallStats {
    pub md3_models: u32,
    pub sprites: u32,
    pub particles: u32,
    pub tiles: u32,
    pub ui_elements: u32,
    pub weapon_effects: u32,
    pub shadows: u32,
    pub total: u32,
}

impl DrawCallStats {
    pub fn new() -> Self {
        Self {
            md3_models: 0,
            sprites: 0,
            particles: 0,
            tiles: 0,
            ui_elements: 0,
            weapon_effects: 0,
            shadows: 0,
            total: 0,
        }
    }

    pub fn add(&mut self, category: &str, count: u32) {
        match category {
            "md3_models" => self.md3_models += count,
            "sprites" => self.sprites += count,
            "particles" => self.particles += count,
            "tiles" => self.tiles += count,
            "ui_elements" => self.ui_elements += count,
            "weapon_effects" => self.weapon_effects += count,
            "shadows" => self.shadows += count,
            _ => {}
        }
        self.total += count;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

pub struct ProfileScope {
    name: &'static str,
    shader: Option<&'static str>,
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
    #[cfg(target_arch = "wasm32")]
    start: f64,
}

impl ProfileScope {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            shader: None,
            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),
            #[cfg(target_arch = "wasm32")]
            start: crate::time::get_time(),
        }
    }

    pub fn with_shader(name: &'static str, shader: &'static str) -> Self {
        Self {
            name,
            shader: Some(shader),
            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),
            #[cfg(target_arch = "wasm32")]
            start: crate::time::get_time(),
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let elapsed = self.start.elapsed().as_secs_f64() * 1000.0;
        #[cfg(target_arch = "wasm32")]
        let elapsed = (crate::time::get_time() - self.start) * 1000.0;

        #[cfg(not(target_arch = "wasm32"))]
        PROFILER.with(|p| {
            p.borrow_mut().record(self.name, elapsed, self.shader);
        });
    }
}

#[derive(Clone)]
struct SampleData {
    current: f64,
    avg: f64,
    min: f64,
    max: f64,
    history: Vec<f64>,
}

impl SampleData {
    fn new() -> Self {
        Self {
            current: 0.0,
            avg: 0.0,
            min: f64::MAX,
            max: 0.0,
            history: Vec::new(),
        }
    }

    fn record(&mut self, value: f64) {
        self.current = value;
        self.history.push(value);

        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }

        if !self.history.is_empty() {
            self.avg = self.history.iter().sum::<f64>() / self.history.len() as f64;
            self.min = self.history.iter().copied().fold(f64::MAX, f64::min);
            self.max = self.history.iter().copied().fold(0.0, f64::max);
        }
    }
}

pub struct Profiler {
    samples: HashMap<&'static str, SampleData>,
    frame_samples: HashMap<&'static str, f64>,
    _draw_calls: DrawCallStats,
    frame_draw_calls: DrawCallStats,
    enabled: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
            frame_samples: HashMap::new(),
            _draw_calls: DrawCallStats::new(),
            frame_draw_calls: DrawCallStats::new(),
            enabled: false,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn record(&mut self, name: &'static str, time_ms: f64, shader: Option<&'static str>) {
        if !self.enabled {
            return;
        }

        *self.frame_samples.entry(name).or_insert(0.0) += time_ms;

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shader_name) = shader {
            SHADER_COUNTS.with(|s| {
                let mut counts = s.borrow_mut();
                *counts.entry(shader_name.to_string()).or_insert(0) += 1;
            });
        }
    }

    pub fn record_draw_call(&mut self, category: &str, count: u32) {
        if !self.enabled {
            return;
        }

        self.frame_draw_calls.add(category, count);
    }

    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }

        for (name, time) in self.frame_samples.drain() {
            self.samples
                .entry(name)
                .or_insert_with(SampleData::new)
                .record(time);
        }
    }

    pub fn get_samples(&self) -> Vec<(&'static str, f64, f64, f64, f64)> {
        let mut result: Vec<_> = self
            .samples
            .iter()
            .map(|(name, data)| (*name, data.current, data.avg, data.min, data.max))
            .collect();

        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn get_history(&self, name: &'static str) -> Option<&[f64]> {
        self.samples.get(name).map(|s| s.history.as_slice())
    }
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static PROFILER: std::cell::RefCell<Profiler> = std::cell::RefCell::new(Profiler::new());
}

pub fn scope(name: &'static str) -> ProfileScope {
    ProfileScope::new(name)
}

pub fn toggle() {
    #[cfg(not(target_arch = "wasm32"))]
    PROFILER.with(|p| p.borrow_mut().toggle());
}

pub fn is_enabled() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return PROFILER.with(|p| p.borrow().is_enabled());
    #[cfg(target_arch = "wasm32")]
    return false;
}

pub fn end_frame() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        PROFILER.with(|p| p.borrow_mut().end_frame());

        SHADER_COUNTS.with(|counts| {
            SHADER_DISPLAY.with(|display| {
                *display.borrow_mut() = counts.borrow().clone();
            });
        });

        SHADER_COUNTS.with(|s| {
            s.borrow_mut().clear();
        });
    }
}

pub fn get_samples() -> Vec<(&'static str, f64, f64, f64, f64)> {
    #[cfg(not(target_arch = "wasm32"))]
    return PROFILER.with(|p| p.borrow().get_samples());
    #[cfg(target_arch = "wasm32")]
    return Vec::new();
}

pub fn get_history(name: &'static str) -> Vec<f64> {
    #[cfg(not(target_arch = "wasm32"))]
    return PROFILER.with(|p| {
        p.borrow()
            .get_history(name)
            .map(|h| h.to_vec())
            .unwrap_or_default()
    });
    #[cfg(target_arch = "wasm32")]
    return Vec::new();
}

pub fn count_shader(_shader: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    SHADER_COUNTS.with(|s| {
        let mut counts = s.borrow_mut();
        *counts.entry(_shader.to_string()).or_insert(0) += 1;
    });
}

pub fn get_shader_stats() -> Vec<(String, usize)> {
    #[cfg(not(target_arch = "wasm32"))]
    return SHADER_DISPLAY.with(|d| {
        let mut stats: Vec<_> = d.borrow().iter().map(|(k, v)| (k.clone(), *v)).collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats
    });
    #[cfg(target_arch = "wasm32")]
    return Vec::new();
}

pub fn print_shader_stats() {
    let stats = get_shader_stats();
    let total: usize = stats.iter().map(|(_, count)| count).sum();

    if total > 0 {
        println!("\n=== DRAW CALL BREAKDOWN ===");
        println!("Total draw calls: {}", total);
        println!("\nBy shader:");
        for (name, count) in stats.iter().take(20) {
            let percentage = (*count as f32 / total as f32 * 100.0) as usize;
            println!("  {:30} {:5} ({:2}%)", name, count, percentage);
        }
        println!("========================\n");
    }
}

#[macro_export]
macro_rules! count_shader {
    ($shader:expr) => {
        #[cfg(feature = "profiler")]
        crate::profiler::count_shader($shader);
    };
}
