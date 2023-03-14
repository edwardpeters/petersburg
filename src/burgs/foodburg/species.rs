use super::super::*;
use std::fmt;
use std::sync::Mutex;

pub mod types {
    pub use super::{Species, SpeciesID};
}

pub type SpeciesID = usize;
pub struct Species {
    pub index: usize,
    pub color: Color,
    pub root: Point,
    pub queued_count: Mutex<usize>,
    pub active_count: Mutex<usize>,
}

impl std::hash::Hash for Species {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.index.hash(state)
    }
}

impl PartialEq for Species {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl fmt::Display for Species {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = (
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
        );
        let colored = format!("Species {}", self.index).truecolor(r, g, b);
        write!(f, "{}", colored)
    }
}
impl Eq for Species {}

impl Species {
    pub fn new(index: usize, color: Color, root: Point) -> Self {
        let queued_count = Mutex::new(1);
        let active_count = Mutex::new(0);
        Species {
            index,
            color,
            root,
            queued_count,
            active_count,
        }
    }
}
