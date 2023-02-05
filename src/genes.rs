use crate::geography::*;
use crate::utils::*;
use std::fmt::Display;
use std::string::String;

const MIN_STEPS: usize = 4;

pub mod types {
    pub use super::GeneStep;
    pub use super::GenoType;
}
pub use types::*;

#[derive(Copy, Clone, Debug)]
pub struct GeneStep {
    pub time_to_next: usize,
    pub dir: Compass,
}

impl GeneStep {
    fn rand() -> Self {
        let length: usize = 16 * (1.25 as f32).powi(roll::usize(24) as i32) as usize;
        Self {
            time_to_next: length,
            dir: Compass::rand(),
        }
    }
    fn mutate(&self) -> Self {
        let Self { time_to_next, dir } = *self;
        match roll::usize(4) {
            0 => Self {
                time_to_next,
                dir: dir.left(),
            },
            1 => Self {
                time_to_next,
                dir: dir.right(),
            },
            2 => Self {
                time_to_next: (time_to_next as f32 * 0.8) as usize,
                dir,
            },
            3 => Self {
                time_to_next: (time_to_next as f32 * 1.2) as usize,
                dir,
            },
            _ => panic!("Unreachable"),
        }
    }
}
impl Display for GeneStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.time_to_next, self.dir)
    }
}

#[derive(Clone, Debug)]
pub struct GenoType(pub Vec<GeneStep>, pub String);
impl GenoType {
    pub fn new(length: usize, name: String) -> Self {
        let steps = (0..length).map(|_i| GeneStep::rand()).collect();
        Self(steps, name)
    }
    pub fn mutate(&self, child_num: usize) -> Self {
        let mut steps = self.0.clone();
        let index = roll::usize(steps.len());
        match roll::usize(8) {
            0 => {
                if steps.len() > MIN_STEPS {
                    steps.swap_remove(index);
                }
            }
            1 => steps.insert(index, GeneStep::rand()),
            _ => steps[index] = steps[index].mutate(),
        }
        let new_name = format!("{}-{}", self.1, child_num);
        Self(steps, new_name)
    }
}

impl Display for GenoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenoType : {}\n", self.1).unwrap();
        for step in self.0.iter() {
            write!(f, "\t{}\n", step)?
        }
        write!(f, "I guess it needs an end?")
    }
}
