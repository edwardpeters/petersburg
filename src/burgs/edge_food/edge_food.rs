#![allow(unused_variables, dead_code, unreachable_code)]

use super::super::*;
use constants::*;
use std::sync::{Arc, Mutex};
use std::thread;
#[allow(unused_imports)]
use utils::*;

pub mod types {
    pub use super::EdgeFoodBurg;
    pub use super::EdgeFoodConfig;
}

#[derive(Copy, Clone)]
pub struct ScentSquare {
    pub food: usize,
    pub home: usize,
    pub stuck: bool,
}
pub struct EdgeFoodBurg {
    size: usize,
    grid: Arc<Mutex<WrappedGrid<ScentSquare>>>,
    draw_grid: Arc<Mutex<WrappedGrid<Color>>>,
}

pub struct EdgeFoodConfig {
    pub size: usize,
}

impl Petersburg for EdgeFoodBurg {
    type Config = EdgeFoodConfig;

    fn new(c: Self::Config) -> Self {
        let empty = ScentSquare {
            food: 0,
            home: 0,
            stuck: false,
        };
        let grid: WrappedGrid<ScentSquare> = WrappedGrid::new(c.size, c.size, empty);
        let draw_grid: WrappedGrid<Color> = WrappedGrid::new(c.size, c.size, color::BLACK);
        EdgeFoodBurg {
            size: c.size,
            grid: Arc::new(Mutex::new(grid)),
            draw_grid: Arc::new(Mutex::new(draw_grid)),
        }
    }

    fn run(&self) {
        let mut successes = 0;
        let center = Point(self.size / 2, self.size / 2);
        let mut grid = self.grid.lock().unwrap();
        grid.update(center, |s| ScentSquare { stuck: true, ..s });
        drop(grid);
        let mut time_step = 0;
        let mut start = center;
        loop {
            let start_time = time_step;
            start = self.seek(start, &mut time_step);
            successes = successes + 1;
            let mut grid = self.grid.lock().unwrap();
            grid.update(start, |s| ScentSquare { stuck: true, ..s });
            drop(grid);
            let mut draw_grid = self.draw_grid.lock().unwrap();
            draw_grid.set(start, color::WHITE);
            drop(draw_grid);
            //tx.send(new_point).unwrap();
            println!(
                "A particle made its way home in {} steps, sticking at {}",
                time_step - start_time,
                start
            );
            if time_step - start_time < 3 {
                break;
            };
        }
        println!("All done with a total time_step of {}", time_step);
    }

    fn draw(&self, context: &cairo::Context) {
        self.draw_grid.lock().unwrap().draw(context);
    }
}

impl EdgeFoodBurg {
    fn seek(&self, start: Point, time_step: &mut usize) -> Point {
        let mut grid = self.grid.lock().unwrap();
        let mut dir = Compass::rand();
        let mut p = start;
        let mut homesickness = self.size * self.size;
        let color = (
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        );
        'seek_food: loop {
            // if !grid[x][y].stuck {
            //     let mut draw_grid = draw_mut.lock().unwrap();
            //     draw_grid[x][y] = color;
            //     drop(draw_grid)
            // };
            homesickness = homesickness - 1;
            *time_step = *time_step + 1;
            if p.0 == 0 || p.1 == 0 || p.0 == self.size - 1 || p.1 == self.size - 1 {
                break 'seek_food;
            }
            let ScentSquare { food, home, stuck } = grid.get(p);
            //let home_scent = if (homesickness as usize) > *time_step  {0} else {*time_step - (homesickness as usize)};
            let home_scent = std::cmp::max(homesickness as usize, home);
            grid.set(
                p,
                ScentSquare {
                    food: food,
                    home: home_scent,
                    stuck: stuck,
                },
            );
            let drift = rand::random::<usize>() % 8;
            if drift == 0 {
                dir = dir.left()
            } else if drift == 7 {
                dir = dir.right()
            };
            p = grid.step(p, dir);
        }
        'seek_home: loop {
            // if !grid[x][y].stuck {
            //     let mut draw_grid = draw_mut.lock().unwrap();
            //     draw_grid[x][y] = color;
            //     drop(draw_grid)
            // };
            *time_step = *time_step + 1;
            let hood = grid.get_neighborhood(p);
            for neighbor in hood {
                if neighbor.stuck {
                    break 'seek_home;
                }
            }

            let spos = grid.step(p, dir);
            let rpos = grid.step(p, dir.right());
            let lpos = grid.step(p, dir.left());
            let (mut sweight, mut lweight, mut rweight) = (3, 1, 1);

            let sh = grid.get(spos).home;
            let rh = grid.get(rpos).home;
            let lh = grid.get(lpos).home;

            if sh > lh {
                sweight = sweight + 2
            } else if lh > sh {
                lweight = lweight + 2
            }
            if rh > lh {
                rweight = rweight + 2
            } else if lh > rh {
                lweight = lweight + 2
            }
            if sh > rh {
                sweight = sweight + 2
            } else if rh > sh {
                rweight = rweight + 2
            }
            let roll = rand::random::<usize>() % (lweight + rweight + sweight);
            if roll < sweight {
                p = spos;
            } else if roll < sweight + rweight {
                dir = dir.right();
                p = rpos;
            } else {
                dir = dir.left();
                p = lpos
            }
        }
        p
    }

    //fn steveburg(tx : Sender<Message>){
    pub fn steveburg(&mut self) {
        loop {
            thread::sleep(time::A_SECOND);
        }
        // let mut max = 0;
        // let mut min = usize::MAX;
        // for i in 0..self.size {
        //     for j in 0..self.size {
        //         if grid[i][j].home > max {
        //             max = grid[i][j].home
        //         }
        //         if grid[i][j].home != 0 && grid[i][j].home < min {
        //             min = grid[i][j].home
        //         }
        //     }
        // }
        // let range = max - min;
        // println!("Heat range is {}", range);
        // {
        //     let mut draw_grid = draw_grid_mut.lock().unwrap();
        //     for i in 0..self.size {
        //         for j in 0..self.size {
        //             if grid[i][j].home != 0 {
        //                 draw_grid[i][j] = heat_to_color(grid[i][j].home - min, range);
        //             }
        //             // tx.send((i, j, heat_to_color(grid[i][j].home, max))).unwrap();
        //         }
        //     }
        //     drop(draw_grid);
        // }
        // println!("ALl dropped?");
    }
}
