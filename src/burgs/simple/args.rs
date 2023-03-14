
use clap::*;

#[derive(Args, Debug, Copy, Clone)]
pub struct SimpleArgs {
    #[arg(long, short, default_value_t = 1024)]
    pub size: usize,
    #[arg(long, default_value_t = 8)]
    pub num_threads: usize,
}