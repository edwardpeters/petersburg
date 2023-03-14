use crate::simulation::*; //I'd rather this be simulation::*
use crate::utils::*;
use clap::Args;
use std::{thread, time};

pub mod types {
    pub use super::{Unburg, UnburgArgs};
}

#[derive(Args, Debug, Copy, Clone)]
pub struct UnburgArgs {
    #[arg(long, short, default_value_t = 3)]
    pub size: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Unburg {
    pub size: usize,
    pub words: &'static str,
    pub color: Color,
}

impl Petersburg for Unburg {
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

impl Unburg {
    pub fn new(args: UnburgArgs) -> Self {
        Self {
            size: args.size,
            words: "Some words exist here",
            color: color::BLUE,
        }
    }

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
