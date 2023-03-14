use cairo::Context;

#[allow(unused_imports)]
use super::{super::*, *};
use std::sync::{Arc, Mutex, RwLock};

const MAX_LIFE: usize = 10_000;

pub mod types {
    pub use super::Simpleburg;
}
//type DrawGrid = Vec<Vec<Color>>;

pub struct Simpleburg {
    args: SimpleArgs,
    draw_grid_m: Arc<Mutex<WrappedGrid<Color>>>,
    draw_path_m: Arc<Mutex<(Color, Vec<Point>)>>,
    grid_rw: Arc<RwLock<WrappedGrid<usize>>>,
}
impl Simpleburg {
    pub fn new(args: SimpleArgs) -> Self {
        use self::color::*;
        let draw_path_m = Arc::new(Mutex::new((BLACK, Vec::<Point>::new())));
        let draw_grid_m = Arc::new(Mutex::new(WrappedGrid::<Color>::new(
            args.size,
            args.size,
            color::BLACK,
        )));
        let grid = WrappedGrid::<usize>::new(args.size, args.size, usize::MAX);
        let grid_rw = Arc::new(RwLock::new(grid));
        Self {
            args,
            draw_grid_m,
            draw_path_m,
            grid_rw,
        }
    }
    fn seek_threaded(
        &self,
        g_mut: Arc<RwLock<WrappedGrid<usize>>>,
        draw_grid_mut: Arc<Mutex<WrappedGrid<Color>>>,
        draw_path_mut: Arc<Mutex<(Color, Vec<Point>)>>,
        index: usize,
    ) {
        let size = self.args.size;
        let center = Point(size / 2, size / 2);
        let color = color::COLORS[index as usize];
        let (mut total, mut lived, mut steps, mut aged, crashed) = (0, 0, 0, 0, 0);

        'main: loop {
            total = total + 1;
            let mut dir = Compass::rand();
            let startx = size / 2 + rand::random::<usize>() % (size / 16) - (size / 32);
            let starty = size / 2 + rand::random::<usize>() % (size / 16) - (size / 32);
            let mut p = Point(starty, startx);
            let mut path = Vec::<Point>::new();
            let lucky = rand::random::<f64>() < 0.001;
            let turniness = 32; //rand::random::<usize>() % 64 + 2;
            let mut life = 0;
            'seek: loop {
                steps = steps + 1;
                life = life + 1;
                if life > MAX_LIFE {
                    aged = aged + 1;
                    if lucky {
                        let mut draw_path = draw_path_mut.lock().unwrap();
                        draw_path.1 = path;
                        draw_path.0 = color::COLORS[index as usize];
                    }
                    continue 'main;
                }
                if lucky {
                    path.push(p);
                };
                if Point::distance(center, p) > (size / 2 - 2) as f64 {
                    break;
                }

                let roll = rand::random::<usize>() % turniness;
                if roll == 0 {
                    dir = dir.left()
                }
                if roll == turniness - 1 {
                    dir = dir.right()
                }

                {
                    let g = g_mut.read().unwrap();
                    let next = g.step(p, dir);
                    let mut friendly = false;
                    let mut unfriendly = false;
                    for n_dir in Compass::all() {
                        let neighbor = g.get(g.step(p, n_dir));
                        if neighbor == index {
                            friendly = true
                        } else if neighbor != usize::MAX {
                            unfriendly = true
                        }
                    }
                    if friendly && !unfriendly {
                        break 'seek;
                    }
                    if g.get(next) != usize::MAX {
                        dir = dir.right();
                        continue 'seek;
                    }
                    drop(g);
                    p = next;
                }
            }
            if lucky {
                let mut draw_path = draw_path_mut.lock().unwrap();
                draw_path.1 = path;
                draw_path.0 = color::COLORS[index as usize];
            }
            lived = lived + 1;
            let mut g = g_mut.write().unwrap();
            g.set(p, index);
            drop(g);
            let mut draw_g = draw_grid_mut.lock().unwrap();
            draw_g.set(p, color);
            if Point::distance(center, p) < (size / 16) as f64 {
                break 'main;
            }
        }
        println!(
        "Thread {} finished after {} steps with a total of {} placed, {} crashed and {} aged out.",
        index, steps, lived, crashed, aged
    )
    }
}

impl Petersburg for Simpleburg {
    fn run(&self) {
        crossbeam::scope(|scope| {
            for i in 0..self.args.num_threads {
                let draw_grid_m = Arc::clone(&(self.draw_grid_m));
                let draw_path_m = Arc::clone(&(self.draw_path_m));
                let grid_rw = Arc::clone(&(self.grid_rw));
                scope.spawn(move |_| {
                    self.seek_threaded(grid_rw, draw_grid_m, draw_path_m, i);
                });
            }
        })
        .unwrap();
    }
    fn draw(&self, context: &Context) {
        self.draw_grid_m.lock().unwrap().draw(context);
        let pair = self.draw_path_m.lock().unwrap();
        let (color, path) = (pair.0, &pair.1);
        draw_utils::path_helper(context, self.args.size, color, path);
    }
}
