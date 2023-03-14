#![allow(dead_code)]

pub fn pseudo_average_update(previous: f32, new: f32, length: usize) -> f32 {
    (length as f32 * previous - 1.0 + new) / (length as f32)
}

#[inline(always)]
pub fn modulo(a: i32, b: usize) -> usize {
    //a.rem_euclid(b as i32) as usize
    (((a % b as i32) + b as i32) % b as i32) as usize
}

pub mod roll {
    #[inline(always)]
    pub fn usize(bound: usize) -> usize {
        rand::random::<usize>() % bound
    }
    #[inline(always)]
    pub fn i32(lower: i32, upper: i32) -> i32 {
        rand::random::<i32>().abs() as i32 % (upper - lower) + lower
    }
    #[inline(always)]
    pub fn bool() -> bool {
        rand::random::<bool>()
    }
    #[inline(always)]
    pub fn under(threshold: f64) -> bool {
        rand::random::<f64>() < threshold
    }
}
