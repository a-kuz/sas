pub fn gen_bool(prob: f64) -> bool {
    fastrand::f64() < prob
}

pub fn gen_f32() -> f32 {
    fastrand::f32()
}

pub fn gen_u8() -> u8 {
    fastrand::u8(..)
}

pub fn gen_range_f32(min: f32, max: f32) -> f32 {
    fastrand::f32() * (max - min) + min
}

pub fn gen_range_i32(min: i32, max: i32) -> i32 {
    fastrand::i32(min..max)
}

pub fn gen_range_usize(min: usize, max: usize) -> usize {
    fastrand::usize(min..max)
}

pub fn gen_range(min: usize, max: usize) -> usize {
    fastrand::usize(min..max)
}
