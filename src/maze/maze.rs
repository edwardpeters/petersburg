use super::*;
use crate::geography::{wrapped::*, Cardinal::*, *};
use crate::utils::*;

#[derive(Debug)]
pub struct Maze {
    pub(super) num_squares: usize,
    pub(super) vbars: Vec<Vec<bool>>,
    pub(super) hbars: Vec<Vec<bool>>,
    pub(super) wrapped: bool,
    pub(super) scale: usize,
}

/**
 * So the issue is that maze needs to no how many squares it has, and for things to work, that needs to be a factor of the total number of squares
 * Could be done at type level by specifying size of square and number of squares, but that's problematic.
 * So need a check - here or in sim (here is probably better?)
 */
impl Maze {
    pub fn new(size: usize, wrapped: bool, args: MazeArgs) -> Self {
        let scale = if size % args.num_squares == 0 {
            size / args.num_squares
        } else {
            panic!(
                "Maze squares must be an exact fraction of total size: {} % {} = {}",
                size,
                args.num_squares,
                size % args.num_squares
            )
        };
        let hbars = vec![vec!(true; args.num_squares); args.num_squares];
        let vbars = vec![vec!(true; args.num_squares); args.num_squares];
        let mut maze = Maze {
            num_squares: args.num_squares,
            vbars,
            hbars,
            wrapped,
            scale,
        };
        match args.gen_method {
            args::GenMethod::Pathed => maze.random_pathed(),
            args::GenMethod::Open => maze.random_open(),
        }
        maze.remove_walls(args.openness);
        maze
    }
    fn random_pathed(&mut self) {
        let mut connected = WrappedGrid::new(self.num_squares, self.num_squares, false);
        connected.set(connected.rand(), true);
        let mut num_to_connect = self.num_squares * self.num_squares - 1;
        'connect: loop {
            if num_to_connect == 0 {
                break 'connect;
            }
            let mut p = connected.rand();
            if !connected.get(p) {
                continue 'connect;
            }
            'path: loop {
                use rand::prelude::*;
                let Point(x, y) = p;
                let all_dirs = Cardinal::all();
                let dir = all_dirs
                    .iter()
                    .filter(|dir| {
                        (self.wrapped
                            || (x != 0 || **dir != W)
                                && (x != self.num_squares - 1 || **dir != E)
                                && (y != 0 || **dir != N)
                                && (y != self.num_squares - 1 || **dir != S))
                            && !connected.get(connected.step(p, **dir))
                    })
                    .choose(&mut rand::thread_rng());
                match dir {
                    None => break 'path,
                    Some(dir_for_realsies) => {
                        let neighbor = connected.step(p, *dir_for_realsies);
                        connected.set(neighbor, true);
                        num_to_connect = num_to_connect - 1;

                        match dir_for_realsies {
                            N => self.hbars[x][y] = false,
                            S => self.hbars[neighbor.0][neighbor.1] = false,
                            W => self.vbars[x][y] = false,
                            E => self.vbars[neighbor.0][neighbor.1] = false,
                        }
                        p = neighbor;
                    }
                }
            }
        }
    }
    fn random_open(&mut self) {
        let mut connected = WrappedGrid::new(self.num_squares, self.num_squares, false);
        let mut num_to_connect = self.num_squares * self.num_squares - 1;
        connected.set(Point(0, 0), true);
        'connect: loop {
            if num_to_connect == 0 {
                break 'connect;
            }
            let x = roll::usize(self.num_squares);
            let y = roll::usize(self.num_squares);
            if roll::bool() {
                if !self.wrapped && y == 0 {
                    continue 'connect;
                }
                if self.hbars[x][y] == true {
                    let below = Point(x, y);
                    let above = connected.step(below, Cardinal::N);
                    if connected.get(below) ^ connected.get(above) {
                        num_to_connect = num_to_connect - 1;
                        connected.set(above, true);
                        connected.set(below, true);
                        self.hbars[x][y] = false;
                    }
                }
            } else {
                if self.vbars[x][y] == true {
                    if !self.wrapped && x == 0 {
                        continue 'connect;
                    }
                    let right = Point(x, y);
                    let left = connected.step(right, Cardinal::W);
                    if connected.get(right) ^ connected.get(left) {
                        num_to_connect = num_to_connect - 1;
                        connected.set(right, true);
                        connected.set(left, true);
                        self.vbars[x][y] = false;
                    }
                }
            }
        }
    }
    fn remove_walls(&mut self, factor: f64) {
        for i in 0..self.num_squares {
            for j in 0..self.num_squares {
                if roll::under(factor) && (self.wrapped || j != 0) {
                    self.hbars[i][j] = false
                };
                if roll::under(factor) && (self.wrapped || i != 0) {
                    self.vbars[i][j] = false;
                };
            }
        }
    }
    pub fn is_wall(&self, p: Point) -> bool {
        let Point(x, y) = p;
        let scale = self.scale;
        if x % self.scale != 0 && y % self.scale != 0 {
            false
        } else if x % scale == 0 && y % scale != 0 {
            //We're on line with a wall
            self.vbars[x / scale][y / scale]
        } else if x % scale != 0 && y % scale == 0 {
            //We're on line with a wall
            self.hbars[x / scale][y / scale]
        } else {
            self.vbars[x / scale][y / scale]
                || self.hbars[x / scale][y / scale]
                || self.hbars[modulo((x / scale) as i32 - 1, self.num_squares)][y / scale]
                || self.vbars[x / scale][modulo((y / scale) as i32 - 1, self.num_squares)]
        }
    }
}
