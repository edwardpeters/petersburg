use super::super::*;
use super::*;
#[allow(unused_imports)]
use super::{super::*, *};
use crate::genes::*;
use crate::maze::*;
use cairo::Context;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

type MoveDir = Compass;
type BuildDir = Compass;
const MAX_CHILDREN: usize = 200;
//const MAX_AGE: i64 = SIZE as i64 * 200;
const TURNINESS: usize = 32;
const MIN_DISTANCE_WRAPPED: f64 = 0.0;
const MIN_DISTANCE_UNWRAPPED: f64 = 0.0;
const MAX_DEAD_STREAK: usize = 10_000;
const FITNESS_HISTORY: usize = 200;
const NORMAL_STEPS: usize = 10;
//static LIFE_RECORD: AtomicI64 = AtomicI64::new(0);

pub mod types {
    pub use super::Mazeburg;
}

struct Species {
    index: usize,
    origin: Point,
    color: Color,
    alive: bool,
    total_time: usize,
    dead_streak: usize,
    root_ancestors: usize,
    fitness: f64,
    candidates: Vec<GenoType>,
}

impl fmt::Display for Species {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = (
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
        );
        let colored = format!("Species {}", self.index).truecolor(r, g, b);
        write!(f, "{}", colored)
    }
}
#[allow(dead_code)]
enum Result {
    Stuck(Point),
    Aged,
    Crashed,
}

pub struct Mazeburg {
    args: MazeburgArgs,
    maze: Arc<Maze>,
    grid_rw: Arc<RwLock<RwGrid<Square>>>,
    draw_path_m: Arc<Mutex<(Color, Vec<Point>)>>,
    species_m: Arc<Mutex<Vec<Species>>>,
    draw_line_params: Arc<Mutex<(bool, Option<usize>)>>,
    max_age: usize,
}

impl Petersburg for Mazeburg {
    fn run(&self) {
        crossbeam::scope(|scope| {
            for i in 0..self.args.num_threads {
                scope.spawn(move |_| {
                    self.run_thread(i);
                });
            }
            scope.spawn(|_| {
                self.stdin_io();
            });
        })
        .unwrap();
    }
    fn draw(&self, context: &Context) {
        self.maze.draw(context);
        let size = self.args.size;
        self.grid_rw.read().unwrap().draw(context);
        let path_params = self.draw_line_params.lock().unwrap();
        let draw_path = path_params.0;
        drop(path_params);
        if draw_path {
            let draw_path = self.draw_path_m.lock().unwrap();
            draw_utils::path_helper(context, size, draw_path.0, &draw_path.1);
        }
    }
}

impl Mazeburg {
    pub fn new(args: MazeburgArgs) -> Self {
        let maze_raw = Maze::new(args.size, args.wrapped, args.maze_args);
        let mut grid: RwGrid<Square> = RwGrid::new(args.size, args.size, square::EMPTY);
        let min_distance = (args.size as f64
            * if args.wrapped {
                MIN_DISTANCE_WRAPPED
            } else {
                MIN_DISTANCE_UNWRAPPED
            }) as usize;
        let line_params_m = Arc::new(Mutex::new((args.show_lines, None)));
        let species = (0..args.num_species)
            .map(|i| Self::species_init(args, i, &mut grid, &maze_raw, min_distance))
            .collect::<Vec<Species>>();
        let species_m = Arc::new(Mutex::new(species));
        let grid_rw = Arc::new(RwLock::new(grid));
        let maze = Arc::new(maze_raw);
        let draw_path_m = Arc::new(Mutex::new((color::BLACK, Vec::new())));
        let max_age = args.size * 200;
        Self {
            args,
            maze,
            grid_rw,
            draw_path_m,
            species_m,
            draw_line_params: line_params_m,
            max_age,
        }
    }
    fn run_thread(&self, _thread_index: usize) {
        'main: loop {
            let mut species = self.species_m.lock().unwrap();

            let mut youngest = species
                .iter_mut()
                .filter(|s| s.alive)
                .min_by(|a, b| a.total_time.cmp(&b.total_time));
            match youngest {
                None => {
                    drop(species);
                    println!("There is no youth, there is no future");
                    break 'main;
                }
                Some(ref mut s) => {
                    let (index, origin, root_ancestors) = (s.index, s.origin, s.root_ancestors);
                    let genes = if s.candidates.is_empty() {
                        s.root_ancestors = s.root_ancestors + 1;
                        GenoType::new(NORMAL_STEPS, format!("{}:{}", index, root_ancestors))
                    } else {
                        let i = roll::usize(s.candidates.len());
                        s.candidates.swap_remove(i)
                    };
                    drop(species);
                    let should_path = {
                        let path_params = self.draw_line_params.lock().unwrap();
                        match (path_params.0, path_params.1) {
                            (false, _) => false,
                            (true, None) => roll::under(0.001),
                            (true, Some(chosen_index)) if index == chosen_index => roll::under(0.1),
                            _ => false,
                        }
                    };
                    let (time, result, path) = self.feno_seek(index, origin, should_path, &genes);
                    let mut species = self.species_m.lock().unwrap();
                    let mut this_species = &mut species[index];
                    if should_path {
                        println!("{genes}");
                        let mut draw_path = self.draw_path_m.lock().unwrap();
                        draw_path.0 = this_species.color;
                        draw_path.1 = path;
                    }
                    this_species.total_time = this_species.total_time + time;
                    match result {
                        Result::Aged | Result::Crashed => {
                            this_species.fitness = this_species.fitness
                                * (FITNESS_HISTORY as f64 - 1.0)
                                / (FITNESS_HISTORY as f64);
                            if this_species.fitness < 1.0 / (FITNESS_HISTORY as f64) {
                                this_species.dead_streak = this_species.dead_streak + 1;
                            };
                            if (this_species.dead_streak > MAX_DEAD_STREAK) && this_species.alive {
                                println!("{this_species} just couldn't cut it. So sad. :(");
                                this_species.alive = false;
                            }
                            drop(species)
                        }
                        Result::Stuck(p) => {
                            this_species.fitness = (1.0
                                + this_species.fitness * (FITNESS_HISTORY as f64 - 1.0))
                                / (FITNESS_HISTORY as f64);
                            let num_children = Self::children_for_fitness(this_species.fitness);
                            for i in 0..num_children {
                                this_species.candidates.push(genes.mutate(i));
                            }
                            if p == origin && this_species.alive {
                                println!("{this_species} has found its hole");
                                this_species.alive = false;
                            }
                            this_species.dead_streak = 0;
                            drop(species);
                            let mut grid = self.grid_rw.write().unwrap();
                            grid.set(p, Square { species: index });
                            drop(grid);
                        }
                    }
                }
            };
        }
    }
    fn species_init(
        args: MazeburgArgs,
        index: usize,
        grid: &mut RwGrid<Square>,
        maze: &Maze,
        min_distance: usize,
    ) -> Species {
        let mut destination;
        let mut origin;
        let (size, wrapped) = (args.size, args.wrapped);
        loop {
            destination = match roll::usize(4) {
                0 => Point(roll::usize(size), roll::usize(size / 8) + 7 * size / 8),
                1 => Point(roll::usize(size / 8) + 7 * size / 8, roll::usize(size)),
                2 => Point(roll::usize(size), roll::usize(size / 8)),
                _ => Point(roll::usize(size / 8), roll::usize(size)),
            };
            origin = Point(
                roll::usize(size / 4) + 3 * size / 8,
                roll::usize(size / 4) + 3 * size / 8,
            );
            if !maze.is_wall(origin)
                && !maze.is_wall(destination)
                && if wrapped {
                    grid.distance(origin, destination)
                } else {
                    Point::distance(origin, destination)
                } > min_distance as f64
            {
                break;
            };
        }
        grid.set(destination, Square { species: index });
        let color = color::COLORS[index];
        Species {
            index,
            color,
            origin,
            alive: true,
            total_time: 0,
            dead_streak: 0,
            root_ancestors: 0,
            fitness: 0.0,
            candidates: Vec::new(),
        }
    }

    fn feno_seek(
        &self,
        index: usize,
        origin: Point,
        track_path: bool,
        steps: &GenoType,
    ) -> (usize, Result, Vec<Point>) {
        let mut p = origin;
        let mut steps = steps.clone();
        let mut path = Vec::new();
        let mut lifetime = 0;
        let mut time_to_next: i32 = 0;
        let mut dir = Compass::N;
        'seek: loop {
            if track_path {
                path.push(p);
            }
            lifetime = lifetime + 1;
            time_to_next = time_to_next - roll::i32(0, 3);
            if lifetime >= self.max_age {
                return (lifetime, Result::Aged, path);
            }
            if time_to_next <= 0 {
                match steps.0.split_first() {
                    None => {
                        return (lifetime, Result::Aged, path);
                    }
                    Some((step, rest)) => {
                        let rest = rest.to_owned();
                        dir = step.dir;
                        time_to_next = step.time_to_next as i32;
                        steps = GenoType(rest.to_vec(), "".to_string());
                    }
                }
            }
            let mut open = true;
            {
                //Critical block
                let grid = self.grid_rw.read().unwrap();
                //Stick?
                let (mut friendly, mut unfriendly) = (false, false);
                for neighbor_dir in BuildDir::all() {
                    if grid.get(grid.step(p, neighbor_dir)).species == index {
                        friendly = true;
                    } else if grid.get(grid.step(p, neighbor_dir)) != square::EMPTY {
                        unfriendly = true;
                    }
                    if self.maze.is_wall(grid.step(p, neighbor_dir)) {
                        open = false
                    }
                }
                if friendly && !unfriendly {
                    return (lifetime, Result::Stuck(p), path);
                };
                //Turn?
                let blocked = |dir| {
                    let next = grid.step(p, dir);
                    self.maze.is_wall(next) || grid.get(next) != square::EMPTY
                };
                if blocked(dir) || (blocked(dir.left()) && blocked(dir.right())) {
                    if blocked(dir.left().left()) {
                        dir = dir.right().right()
                    } else if blocked(dir.right().right()) {
                        dir = dir.left().left()
                    } else {
                        dir = dir.reverse()
                    }
                } else {
                    if open && !friendly && !unfriendly {
                        let roll = roll::usize(TURNINESS);
                        if roll == 0 {
                            p = grid.step(p, dir.right())
                        } else if roll == 1 {
                            p = grid.step(p, dir.left())
                        } else {
                            p = grid.step(p, dir);
                        }
                    } else {
                        p = grid.step(p, dir);
                    }
                }
            }
        }
    }

    fn stdin_io(&self) {
        loop {
            let mut line = String::new();
            let _b1 = std::io::stdin().read_line(&mut line).unwrap();
            if line.starts_with("sound off") {
                let species = self.species_m.lock().unwrap();
                for elem in species.iter() {
                    println!(
                        "{elem}:\n
                        \t {} alive
                        \t {} root_ancestors
                        \t {} candidates
                        \t {:.5} fitness
                        \t {} deadiness",
                        elem.alive,
                        elem.root_ancestors,
                        elem.candidates.len(),
                        elem.fitness,
                        elem.dead_streak
                    );
                }
            } else if line.starts_with("paths on") {
                self.draw_line_params.lock().unwrap().0 = true;
            } else if line.starts_with("paths off") {
                self.draw_line_params.lock().unwrap().0 = false;
            } else if line.starts_with("path for ") {
                let index_string = line.split(" ").last().unwrap().trim();
                let mut params = self.draw_line_params.lock().unwrap();
                params.0 = true;
                params.1 = index_string.parse().ok()
            } else if line.starts_with("path any") {
                self.draw_line_params.lock().unwrap().1 = None;
            } else {
                println!("I didn't understand: {line}");
            }
        }
    }
    #[allow(dead_code)]
    fn species_seek(
        &self,
        index: usize,
        origin: Point,
        track_path: bool,
    ) -> (usize, Result, Vec<Point>) {
        let mut p = origin;
        let mut dir = MoveDir::rand();
        let mut lifetime = 0;
        let mut path = Vec::new();
        'seek: loop {
            if track_path {
                path.push(p);
            }
            lifetime = lifetime + 1;
            if lifetime >= self.max_age {
                return (lifetime, Result::Aged, path);
            }
            {
                //Critical block
                let grid = self.grid_rw.read().unwrap();

                let (mut friendly, mut unfriendly) = (false, false);
                for neighbor_dir in BuildDir::all() {
                    if grid.get(grid.step(p, neighbor_dir)).species == index {
                        friendly = true;
                    } else if grid.get(grid.step(p, neighbor_dir)) != square::EMPTY {
                        unfriendly = true;
                    }
                }
                if friendly && !unfriendly {
                    return (lifetime, Result::Stuck(p), path);
                };
                let next = grid.step(p, dir);
                if self.maze.is_wall(next)
                    || grid.get(next) != square::EMPTY
                    || (grid.get(grid.step(p, dir.right())) != square::EMPTY
                        && grid.get(grid.step(p, dir.left())) != square::EMPTY)
                {
                    if roll::bool() {
                        dir = dir.right().right()
                    } else {
                        dir = dir.left().left()
                    }
                } else {
                    p = next;
                }
            }
            let roll = roll::usize(TURNINESS);
            if roll == 0 {
                dir = dir.right();
            } else if roll == 1 {
                dir = dir.left()
            }
        }
    }
    fn children_for_fitness(fitness: f64) -> usize {
        let whole = (1.0 / fitness).floor() as f64;
        let fraction = (1.0 / fitness) as f64 - whole;
        let total = whole as usize + if roll::under(fraction) { 1 } else { 0 };
        usize::min(total, MAX_CHILDREN)
    }
}
