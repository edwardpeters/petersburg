#![allow(unused_variables, unused_imports, dead_code)]
extern crate cairo;
use self::cairo::Context;

use super::super::*;
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use utils::*;

static TOTAL_COUNT: AtomicUsize = AtomicUsize::new(0);
const SIZE: usize = 512;
const THREADS: usize = 5;
const MAX_LIFE: usize = 10_000;
pub type SimpleConfig = (); //Beat that for simple
pub mod types {
    pub use super::Simple;
}
//type DrawGrid = Vec<Vec<Color>>;

pub struct Simple {
    draw_grid_m: Arc<Mutex<WrappedGrid<Color>>>,
    draw_path_m: Arc<Mutex<(Color, Vec<Point>)>>,
    grid_rw: Arc<RwLock<WrappedGrid<usize>>>,
}
impl Simple {}

impl Petersburg for Simple {
    type Config = SimpleConfig;
    fn new(c: SimpleConfig) -> Self {
        use self::color::*;
        let draw_path_m = Arc::new(Mutex::new((BLACK, Vec::<Point>::new())));
        let draw_grid_m = Arc::new(Mutex::new(WrappedGrid::<Color>::new(
            SIZE,
            SIZE,
            color::BLACK,
        )));
        let grid = WrappedGrid::<usize>::new(SIZE, SIZE, usize::MAX);
        let grid_rw = Arc::new(RwLock::new(grid));
        Self {
            draw_grid_m,
            draw_path_m,
            grid_rw,
        }
    }
    fn run(&self) {
        for i in 0..THREADS {
            seek_threaded_2(&self, i);
        }
    }
    fn draw(&self, context: &Context) {
        self.draw_grid_m.lock().unwrap().draw(context);
        let pair = self.draw_path_m.lock().unwrap();
        let (color, path) = (pair.0, &pair.1);
        draw_utils::path_helper(context, SIZE, color, path);
    }
}

fn seek_threaded_2(simple: &Simple, index: usize) {
    let draw_grid_m = Arc::clone(&(simple.draw_grid_m));
    let draw_path_m = Arc::clone(&(simple.draw_path_m));
    let grid_rw = Arc::clone(&(simple.grid_rw));
    thread::spawn(move || seek_threaded(grid_rw, draw_grid_m, draw_path_m, index));
}

pub fn run(
    draw_grid_mut: Arc<Mutex<WrappedGrid<Color>>>,
    draw_path_mut: Arc<Mutex<(Color, Vec<Point>)>>,
) {
    let grid = WrappedGrid::<usize>::new(SIZE, SIZE, usize::MAX);
    let grid_mut = Arc::new(RwLock::new(grid));
    //drop(grid);
    for i in 0..color::COLORS.len() {
        let grid_mut_clone = Arc::clone(&grid_mut);
        let draw_grid_mut_clone = Arc::clone(&draw_grid_mut);
        let draw_path_mut_clone = Arc::clone(&draw_path_mut);
        thread::spawn(move || {
            seek_threaded(grid_mut_clone, draw_grid_mut_clone, draw_path_mut_clone, i);
        });
    }
    //let mut last = TOTAL_COUNT.fetch_add(0,Ordering::SeqCst);
    // loop{
    //     let current = TOTAL_COUNT.fetch_add(0,Ordering::SeqCst);
    //     println!("In the last second there have been {} hits", current - last);
    //     last = current;
    //     thread::sleep(ONE_SECOND);
    // }
}

fn seek_threaded(
    g_mut: Arc<RwLock<WrappedGrid<usize>>>,
    draw_grid_mut: Arc<Mutex<WrappedGrid<Color>>>,
    draw_path_mut: Arc<Mutex<(Color, Vec<Point>)>>,
    index: usize,
) {
    let center = Point(SIZE / 2, SIZE / 2);
    let color = color::COLORS[index as usize];
    let (mut total, mut lived, mut steps, mut aged, crashed) = (0, 0, 0, 0, 0);

    'main: loop {
        total = total + 1;
        // if total % 500 == 0 {
        //     println!("Thread {} launching {}th particle. So far {} have aged out, {} have crashed and {} made it",
        //     index,
        //     total,
        //     aged,
        //     crashed,
        //     lived)
        // }
        let mut dir = Compass::rand();
        let startx = SIZE / 2 + rand::random::<usize>() % (SIZE / 16) - (SIZE / 32);
        let starty = SIZE / 2 + rand::random::<usize>() % (SIZE / 16) - (SIZE / 32);
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
            //if (steps % 1000000 ==0) {println!("Thread {} has taken its {}nth step", index, steps)};
            let Point(x, y) = p;
            //if x == 0 || y == 0 || x == SIZE as i32-1 || y == SIZE as i32 - 1{
            if Point::distance(center, p) > (SIZE / 2 - 2) as f64 {
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
        let Point(x, y) = p;
        draw_g.set(p, color);
        if Point::distance(center, p) < (SIZE / 16) as f64 {
            break 'main;
        }
        // if (count % 10 == 0) {
        //     let _unused = TOTAL_COUNT.fetch_add(10, Ordering::SeqCst);
        // }
        //println!("Thread {} performed its mission and wants more", index);
    }
    println!(
        "Thread {} finished after {} steps with a total of {} placed, {} crashed and {} aged out.",
        index, steps, lived, crashed, aged
    )
}
