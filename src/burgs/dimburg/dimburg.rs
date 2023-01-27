use crate::simulation::*; //I'd rather this be simulation::*
use crate::utils::*;
use std::{thread, time};

pub mod types {
    pub use super::DimConfig;
    pub use super::Dimburg;
}

pub struct DimConfig {
    pub size: usize,
}

pub struct Dimburg {
    pub size: usize,
    pub words: &'static str,
    pub color: Color,
}

impl Petersburg for Dimburg {
    type Config = DimConfig;

    fn new(c: Self::Config) -> Self {
        Self {
            size: c.size,
            words: "I'd buy that for a dollar",
            color: color::BLUE,
        }
    }

    fn run(&self) {
        crossbeam::scope(|scope| {
            for i in 0..3 {
                scope.spawn(move |_| {
                    self.run_thread(i);
                });
            }
            scope.spawn(|_| {
                self.run_ui();
            });
        })
        .unwrap();
    }

    fn draw(&self, _context: &cairo::Context) {
        println!("I don't know what drawing is. that sounds scary. ")
    }
}

impl Dimburg {
    fn run_thread(&self, i: i32) {
        loop {
            println!("Run thread {} says : {}", i, self.words);
            thread::sleep(time::Duration::from_secs(4))
        }
    }
    fn run_ui(&self) {
        loop {
            println!("UI thread says : {}", self.words);
            thread::sleep(time::Duration::from_secs(5))
        }
    }
}
