#![allow(unused_imports, unused_labels, dead_code)]
use cairo::Context;
use colored::Colorize;
use general::direction::*;
use draw_utils;
use general::maze::*;
use petersburg::*;
use std::fmt;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use types_constants::*;
use general::utils::*;
use general::draw_utils::*;
use wrap_grid::*;
use once_cell::sync::OnceCell;
use genes::*;
use general::grid::Point;

type MoveDir = Compass;
type BuildDir = Compass;
type Square = usize;
const EMPTY_SQUARE: usize = usize::MAX;
const MAX_CHILDREN : usize= 200;
const MAX_AGE: i64 = SIZE as i64 * 200;
const TURNINESS: usize = 32;
const MIN_DISTANCE_WRAPPED: f64 = 0.0;
const MIN_DISTANCE_UNWRAPPED: f64 = 0.0;
const MAX_DEAD_STREAK: usize = 10_000;
const THREADS : usize = 16;
const FITNESS_HISTORY: usize = 200;
const NORMAL_STEPS : usize = 10;
//static LIFE_RECORD: AtomicI64 = AtomicI64::new(0);

#[derive(Debug)]
#[allow(dead_code)]
struct Config{
    size : usize,
    squares : usize,
    wrapped : bool
}
static CONFIG:OnceCell<Config> = OnceCell::new();

struct Species {
    index: usize,
    origin: Point,
    color: Color,
    alive: bool,
    total_time: i64,
    dead_streak: usize,
    adams : usize,
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
    maze: Arc<Maze>,
    grid_rw: Arc<RwLock<Grid<Square>>>,
    draw_grid_m: Arc<Mutex<DrawGrid>>,
    draw_path_m: Arc<Mutex<(Color, Vec<Point>)>>,
    species_m: Arc<Mutex<Vec<Species>>>,
    draw_line_params : Arc<Mutex<(bool, Option<usize>)>>
}

impl Petersburg for Mazeburg {
    fn run(self: Arc<Self>) {
        self.run_threaded(THREADS);
    }
    fn draw(&self, context: &Context) {
        self.maze.draw(context);
        let draw_grid = self.draw_grid_m.lock().unwrap();
        let size = CONFIG.get().unwrap().size;
        draw_utils::draw_grid(context, size, &draw_grid);
        drop(draw_grid);
        let path_params = self.draw_line_params.lock().unwrap();
        let draw_path = path_params.0;
        drop(path_params);
        if draw_path{
            let draw_path = self.draw_path_m.lock().unwrap();
            draw_utils::draw_path(context, size, draw_path.0, &draw_path.1);
        }
    }
}

impl Mazeburg {
    pub fn init(
        size: usize,
        squares: usize,
        species_count: usize,
        wrapped: bool,
        openness: f64,
        show_lines: bool
    ) -> Self {
        if size % squares != 0 {
            panic!(
                "Maze must be an exact fit: {size}%{squares} == {}",
                size % squares
            )
        }
        let config = Config{size, wrapped, squares};
        CONFIG.set(config).unwrap();
        let mut maze_raw = Maze::random_pathed(squares, squares, size / squares, wrapped);
        maze_raw.remove_walls(openness);
        let mut grid: Grid<Square> = Grid::new(size, size, EMPTY_SQUARE);
        let mut draw_grid = vec![vec![BLACK; size]; size];
        let min_distance = (size as f64 * if wrapped {MIN_DISTANCE_WRAPPED} else {MIN_DISTANCE_UNWRAPPED}) as usize;
        let line_params_m = Arc::new(Mutex::new((show_lines, None)));
        let species = (0..species_count)
            .map(|i| {
                Self::species_init(
                    i,
                    &mut grid,
                    &mut draw_grid,
                    &maze_raw,
                    min_distance,
                )
            })
            .collect::<Vec<Species>>();
        let species_m = Arc::new(Mutex::new(species));
        let grid_rw = Arc::new(RwLock::new(grid));
        let maze = Arc::new(maze_raw);
        let draw_grid_m: Arc<Mutex<DrawGrid>> = Arc::new(Mutex::new(draw_grid));
        let draw_path_m = Arc::new(Mutex::new((BLACK, Vec::new())));
        Self {
            maze,
            grid_rw,
            draw_grid_m,
            draw_path_m,
            species_m,
            draw_line_params: line_params_m
        }
    }
    fn run_threaded(self: Arc<Self>, threads: usize) {
        for _thread_index in 0..threads {
            let self_clone = Arc::clone(&self);
            thread::spawn(move || 'main: loop {
                let mut species = self_clone.species_m.lock().unwrap();

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
                        let (index, origin, adams) = (s.index, s.origin, s.adams);
                        let genes = if s.candidates.is_empty(){
                            s.adams = s.adams + 1;
                            GenoType::new(NORMAL_STEPS, format!("{}:{}", index, adams))
                        } else {
                            let i = roll::usize(s.candidates.len());
                            s.candidates.swap_remove(i)
                        };
                        drop(species);
                        let should_path = {
                            let path_params = self_clone.draw_line_params.lock().unwrap();
                            match (path_params.0, path_params.1) {
                                (false, _) => false,
                                (true, None) => roll::under(0.001),
                                (true, Some(chosen_index)) if index == chosen_index => roll::under(0.1),
                                _ => false
                            }
                        };
                        let (time, result, path) =
                            self_clone.feno_seek(index, origin, should_path, &genes);
                        let mut species = self_clone.species_m.lock().unwrap();
                        let mut this_species = &mut species[index];
                        if should_path {
                            println!("{genes}");
                            let mut draw_path = self_clone.draw_path_m.lock().unwrap();
                            draw_path.0 = this_species.color;
                            draw_path.1 = path;
                        }
                        this_species.total_time = this_species.total_time + time;
                        match result {
                            Result::Aged | Result::Crashed => {
                                this_species.fitness = this_species.fitness * (FITNESS_HISTORY as f64 - 1.0)/(FITNESS_HISTORY as f64);
                                if this_species.fitness < 1.0/(FITNESS_HISTORY as f64){
                                    this_species.dead_streak = this_species.dead_streak + 1;
                                };
                                if (this_species.dead_streak > MAX_DEAD_STREAK)
                                    && this_species.alive
                                {
                                    println!("{this_species} just couldn't cut it. So sad. :(");
                                    this_species.alive = false;
                                }
                                drop(species)
                            }
                            Result::Stuck(p) => {
                                this_species.fitness = (1.0 + this_species.fitness * (FITNESS_HISTORY as f64 - 1.0))/(FITNESS_HISTORY as f64);
                                let num_children = Self::children_for_fitness(this_species.fitness);
                                //println!("Success, adding {num_children}");
                                for i in 0 .. num_children{
                                    this_species.candidates.push(genes.mutate(i));
                                }
                                let color = this_species.color;
                                if p == origin && this_species.alive {
                                    println!("{this_species} has found its hole");
                                    this_species.alive = false;
                                }
                                this_species.dead_streak = 0;
                                drop(species);
                                let mut grid = self_clone.grid_rw.write().unwrap();
                                grid.set(p, index);
                                drop(grid);
                                let mut draw_grid = self_clone.draw_grid_m.lock().unwrap();
                                draw_grid[p.0][p.1] = color;
                                drop(draw_grid);
                            }
                        }
                    }
                };
            });
        }
        Self::stdin_io(self);
    }
    fn species_init(
        index: usize,
        grid: &mut Grid<Square>,
        draw_grid: &mut DrawGrid,
        maze: &Maze,
        min_distance: usize
    ) -> Species {
        let mut destination;
        let mut origin;
        let config = CONFIG.get().unwrap();
        let (size, wrapped) = (config.size, config.wrapped);
        loop {

             destination = 
                match roll::usize(4) {
                        0 => {grid.fix(Point(roll::usize(size), roll::usize(size/8) + 7 * size/8))}
                        1 => {grid.fix(Point(roll::usize(size/8) + 7 * size/8, roll::usize(size)))}
                        2 => {grid.fix(Point(roll::usize(size), roll::usize(size/8)))}
                        _ => {grid.fix(Point(roll::usize(size/8), roll::usize(size)))}
                    };
            origin = grid.fix(Point(roll::usize(size/4) + 3 * size/8, roll::usize(size/4) + 3 * size/8));
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
        grid.set(destination, index);
        let color = COLORS[index];
        draw_grid[destination.0][destination.1] = color;
        Species {
            index,
            color,
            origin,
            alive: true,
            total_time: 0,
            dead_streak: 0,
            adams : 0,
            fitness: 0.0,
            candidates: Vec::new(),
        }
    }

    fn feno_seek(
        &self,
        index : usize,
        origin : Point,
        track_path : bool,
        steps : &GenoType
    ) -> (i64, Result, Vec<Point>){
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
            if lifetime >= MAX_AGE{
                return (lifetime, Result::Aged, path);
            }
            if time_to_next <= 0 {
                match steps.0.split_first(){
                    None => {return (lifetime, Result::Aged, path);}
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
                    if grid.at(grid.step(p, neighbor_dir)) == index {
                        friendly = true;
                    } else if grid.at(grid.step(p, neighbor_dir)) != EMPTY_SQUARE {
                        unfriendly = true;
                    }
                    if self.maze.is_wall(grid.step(p, neighbor_dir)) {open = false}
                }
                if friendly && !unfriendly {
                    return (lifetime, Result::Stuck(p), path);
                };
                //Turn?
                let blocked = |dir|{
                    let next = grid.step(p, dir);
                    self.maze.is_wall(next) || grid.at(next) != EMPTY_SQUARE};
                if blocked(dir) || (blocked(dir.left()) && blocked(dir.right())){
                     if blocked(dir.left().left()) {dir = dir.right().right()}
                     else if blocked(dir.right().right()) { dir = dir.left().left()}
                     else {dir = dir.reverse()}
                } else {
                    if open && !friendly && !unfriendly{
                        let roll = roll::usize(TURNINESS);
                        if roll == 0 {p = grid.step(p, dir.right())}
                        else if roll == 1 {p = grid.step(p, dir.left())}
                        else 
                        {p = grid.step(p, dir);}
                    } else {
                    p = grid.step(p, dir);
                    }
                }
            }
        }
    }

    fn stdin_io(self_a : Arc<Self>){
        thread::spawn(move ||{
            loop{
                let mut line = String::new();
                let _b1 = std::io::stdin().read_line(&mut line).unwrap();
                if line.starts_with("sound off"){
                let species = self_a.species_m.lock().unwrap();
                    for elem in species.iter(){
                        println!("{elem}:\n
                        \t {} alive
                        \t {} adams
                        \t {} candidates
                        \t {:.5} fitness
                        \t {} deadiness", elem.alive, elem.adams, elem.candidates.len(), elem.fitness, elem.dead_streak);
                    }
                } else if line.starts_with("paths on") {
                    self_a.draw_line_params.lock().unwrap().0 = true;
                } else if line.starts_with("paths off") {
                    self_a.draw_line_params.lock().unwrap().0 = false;
                } else if line.starts_with("path for "){
                    let index_string = line.split(" ").last().unwrap().trim();
                    let mut params = self_a.draw_line_params.lock().unwrap();
                    params.0 = true;
                    params.1 = index_string.parse().ok()
                } else if line.starts_with("path any") {
                    self_a.draw_line_params.lock().unwrap().1 = None;
                } else {
                    println!("I didn't understand: {line}");
                }
            }
        });
    }
    #[allow(dead_code)]
    fn species_seek(
        &self,
        index: usize,
        origin: Point,
        track_path: bool,
    ) -> (i64, Result, Vec<Point>) {
        let mut p = origin;
        let mut dir = MoveDir::rand();
        let mut lifetime = 0;
        let mut path = Vec::new();
        'seek: loop {
            if track_path {
                path.push(p);
            }
            lifetime = lifetime + 1;
            if lifetime >= MAX_AGE {
                return (lifetime, Result::Aged, path);
            }
            {
                //Critical block
                let grid = self.grid_rw.read().unwrap();

                let (mut friendly, mut unfriendly) = (false, false);
                for neighbor_dir in BuildDir::all() {
                    if grid.at(grid.step(p, neighbor_dir)) == index {
                        friendly = true;
                    } else if grid.at(grid.step(p, neighbor_dir)) != EMPTY_SQUARE {
                        unfriendly = true;
                    }
                }
                if friendly && !unfriendly {
                    return (lifetime, Result::Stuck(p), path);
                };
                let next = grid.step(p, dir);
                if self.maze.is_wall(next)
                    || grid.at(next) != EMPTY_SQUARE
                    || (grid.at(grid.step(p, dir.right())) != EMPTY_SQUARE
                        && grid.at(grid.step(p, dir.left())) != EMPTY_SQUARE)
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
    fn children_for_fitness(fitness : f64) -> usize{
        let whole = (1.0/fitness).floor() as f64;
        let fraction = (1.0/fitness) as f64 - whole;
        let total = whole as usize + if roll::under(fraction) {1} else {0};
        usize::min(total, MAX_CHILDREN)
    }
}
