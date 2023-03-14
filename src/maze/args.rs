use clap::*;
use std::fmt::{Display, Formatter, Result};

#[derive(Args, Debug, Copy, Clone)]
pub struct MazeArgs {
    #[arg(long, default_value_t = 16)]
    pub num_squares: usize,
    #[arg(long, default_value_t = 0.1)]
    pub openness: f64,
    #[arg(long, value_enum, default_value_t = GenMethod::Pathed)]
    pub gen_method: GenMethod,
}

#[derive(ValueEnum, Debug, Copy, Clone)]
pub enum GenMethod {
    Pathed,
    Open,
}

impl Display for GenMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = match self {
            GenMethod::Pathed => "Pathed",
            GenMethod::Open => "Open",
        };
        write!(f, "{}", s)
    }
}
