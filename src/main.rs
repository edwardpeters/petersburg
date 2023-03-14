#![allow(unused_labels)]

pub mod burgs;
pub mod constants;
pub mod genes;
pub mod geography;
pub mod maze;
mod run;
pub mod simulation;
pub mod utils;

fn main() {
    run::run();
}
