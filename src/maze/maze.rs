use crate::geography::{wrapped::*, Cardinal::*, *};
use crate::utils::*;
pub struct Maze {
    pub(super) height: usize,
    pub(super) width: usize,
    pub(super) vbars: Vec<Vec<bool>>,
    pub(super) hbars: Vec<Vec<bool>>,
    pub(super) wrap: bool,
    pub(super) scale: usize,
}

impl Maze {
    pub fn random(height: usize, width: usize, scale: usize, wrap: bool) -> Self {
        let mut hbars = vec![vec!(false; width); height];
        let mut vbars = vec![vec!(false; width); height];
        for i in 0..width {
            for j in 0..height {
                hbars[i][j] = rand::random::<bool>();
                vbars[i][j] = rand::random::<bool>();
            }
        }
        Self {
            height: height,
            width: width,
            vbars: vbars,
            hbars: hbars,
            wrap: wrap,
            scale: scale,
        }
    }
    pub fn random_pathed(height: usize, width: usize, scale: usize, wrap: bool) -> Self {
        let mut hbars = vec![vec!(true; width); height];
        let mut vbars = vec![vec!(true; width); height];
        let mut connected = WrappedGrid::new(height, width, false);
        connected.set(connected.rand(), true);
        let mut num_to_connect = height * width - 1;
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
                        (wrap
                            || (x != 0 || **dir != W)
                                && (x != width - 1 || **dir != E)
                                && (y != 0 || **dir != N)
                                && (y != height - 1 || **dir != S))
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
                            N => hbars[x][y] = false,
                            S => hbars[neighbor.0][neighbor.1] = false,
                            W => vbars[x][y] = false,
                            E => vbars[neighbor.0][neighbor.1] = false,
                        }
                        p = neighbor;
                    }
                }
            }
        }
        Self {
            height: height,
            width: width,
            vbars: vbars,
            hbars: hbars,
            wrap: wrap,
            scale: scale,
        }
    }
    pub fn random_open(height: usize, width: usize, scale: usize, wrap: bool) -> Self {
        let mut hbars = vec![vec!(true; width); height];
        let mut vbars = vec![vec!(true; width); height];
        let mut connected = WrappedGrid::new(height, width, false);
        let mut num_to_connect = height * width - 1;
        connected.set(Point(0, 0), true);
        'connect: loop {
            //println!("Still in the loop");
            if num_to_connect == 0 {
                break 'connect;
            }
            let x = roll::usize(width);
            let y = roll::usize(height);
            if roll::bool() {
                if !wrap && y == 0 {
                    continue 'connect;
                }
                if hbars[x][y] == true {
                    let below = Point(x, y);
                    let above = connected.step(below, Cardinal::N);
                    if connected.get(below) ^ connected.get(above) {
                        num_to_connect = num_to_connect - 1;
                        connected.set(above, true);
                        connected.set(below, true);
                        hbars[x][y] = false;
                    }
                }
            } else {
                if vbars[x][y] == true {
                    if !wrap && x == 0 {
                        continue 'connect;
                    }
                    let right = Point(x, y);
                    let left = connected.step(right, Cardinal::W);
                    if connected.get(right) ^ connected.get(left) {
                        num_to_connect = num_to_connect - 1;
                        connected.set(right, true);
                        connected.set(left, true);
                        vbars[x][y] = false;
                    }
                }
            }
        }
        Self {
            height: height,
            width: width,
            vbars: vbars,
            hbars: hbars,
            wrap: wrap,
            scale: scale,
        }
    }
    pub fn remove_walls(&mut self, factor: f64) {
        for i in 0..self.width {
            for j in 0..self.height {
                if roll::under(factor) && (self.wrap || j != 0) {
                    self.hbars[i][j] = false
                };
                if roll::under(factor) && (self.wrap || i != 0) {
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
                || self.hbars[modulo((x / scale) as i32 - 1, self.width)][y / scale]
                || self.vbars[x / scale][modulo((y / scale) as i32 - 1, self.height)]
        }
    }
}
