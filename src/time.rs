use std::time::Instant;

static mut START_TIME: Option<Instant> = None;

pub fn init() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}

pub fn get_time() -> f64 {
    unsafe {
        if let Some(start) = START_TIME {
            start.elapsed().as_secs_f64()
        } else {
            START_TIME = Some(Instant::now());
            0.0
        }
    }
}

pub fn get_time_millis() -> u64 {
    unsafe {
        if let Some(start) = START_TIME {
            start.elapsed().as_millis() as u64
        } else {
            START_TIME = Some(Instant::now());
            0
        }
    }
}

