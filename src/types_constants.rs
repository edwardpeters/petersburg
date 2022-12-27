#![allow(unused_variables, dead_code)]

use std::time;
use general::draw_utils::*;

pub const SIZE: usize = 512;
// pub type Color = (f64, f64, f64);
// pub const BLACK: Color = (0.0, 0.0, 0.0);
// pub const WHITE: Color = (1.0, 1.0, 1.0);
// pub const RED: Color = (1.0, 0.0, 0.0);
// pub const GREEN: Color = (0.0, 1.0, 0.0);


pub const REFRESH: time::Duration = time::Duration::from_millis(500);
pub const ONE_SECOND: time::Duration = time::Duration::from_millis(1000);
pub const LONG_WAIT: time::Duration = time::Duration::from_millis(10000);
pub type DrawGrid = Vec<Vec<Color>>;
pub type Message = (usize, usize, Color);
pub const MAX_LIFE: usize = 10_000;

#[derive(Copy, Clone)]
pub struct ScentSquare {
    pub food: usize,
    pub home: usize,
    pub stuck: bool,
}
pub type ScentGrid = Vec<Vec<ScentSquare>>;
