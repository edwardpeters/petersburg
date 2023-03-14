use crate::maze::*;
use clap::*;

#[derive(Args, Debug, Copy, Clone)]
pub struct MazeburgArgs {
    #[arg(long, short, default_value_t = 1024)]
    pub size: usize,
    #[arg(long, default_value_t = 8)]
    pub num_threads: usize,
    #[arg(long, default_value_t = 6)]
    pub num_species: usize,
    #[arg(long, default_value_t = false)]
    pub wrapped: bool,
    #[arg(long, default_value_t = false)]
    pub show_lines: bool,
    #[command(flatten)]
    pub maze_args: MazeArgs,
}
