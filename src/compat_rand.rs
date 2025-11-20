pub fn gen_bool(prob: f64) -> bool {
    (macroquad::rand::rand() as f64 / u32::MAX as f64) < prob
}

pub fn gen_f32() -> f32 {
    macroquad::rand::rand() as f32 / u32::MAX as f32
}

pub fn gen_u8() -> u8 {
    macroquad::rand::gen_range(0, 256) as u8
}

pub fn gen_range_f32(min: f32, max: f32) -> f32 {
    min + (max - min) * gen_f32()
}

pub fn gen_range_i32(min: i32, max: i32) -> i32 {
    macroquad::rand::gen_range(min, max)
}

pub fn gen_range_usize(min: usize, max: usize) -> usize {
    macroquad::rand::gen_range(min, max)
}

