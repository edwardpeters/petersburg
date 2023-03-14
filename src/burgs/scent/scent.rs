use super::super::*;
use clap::*;
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]

pub mod types {
    pub use super::Scentburg;
    pub use super::ScentburgArgs;
}

#[derive(Copy, Clone)]
pub struct ScentSquare {
    pub food: usize,
    pub home: usize,
    pub stuck: bool,
}
pub struct Scentburg {
    size: usize,
    grid: Arc<Mutex<WrappedGrid<ScentSquare>>>,
    draw_grid: Arc<Mutex<WrappedGrid<Color>>>,
}

#[derive(Args, Copy, Clone, Debug)]
pub struct ScentburgArgs {
    #[arg(long, short, default_value_t = 1024)]
    pub size: usize,
}

impl Petersburg for Scentburg {
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

impl Scentburg {
    pub fn new(c: ScentburgArgs) -> Self {
        let empty = ScentSquare {
            food: 0,
            home: 0,
            stuck: false,
        };
        let grid: WrappedGrid<ScentSquare> = WrappedGrid::new(c.size, c.size, empty);
        let draw_grid: WrappedGrid<Color> = WrappedGrid::new(c.size, c.size, color::BLACK);
        Scentburg {
            size: c.size,
            grid: Arc::new(Mutex::new(grid)),
            draw_grid: Arc::new(Mutex::new(draw_grid)),
        }
    }
    fn seek(&self, start: Point, time_step: &mut usize) -> Point {
        let mut grid = self.grid.lock().unwrap();
        let mut dir = Compass::rand();
        let mut p = start;
        let mut homesickness = self.size * self.size;
        'seek_food: loop {
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
}
