use super::{super::*, *};
use crate::maze::*;
use cairo::Context;
use std::collections::BinaryHeap;
use std::sync::Mutex;

pub mod types {
    pub use super::Foodburg;
}

const MAX_SPORE_LIFE: usize = 5_000;
const RIPE_AGE: usize = 100;
const ROT_AGE: usize = 5_000;
const FOOD_SPAWN_RATE: usize = 50;
const MAX_LIVING: usize = 400;
const CHILD_COUNT: usize = 4;

static MOLD_COLORS: [Color; 11] = [
    color::RED,
    color::BLUE,
    color::PURPLE,
    color::TEAL,
    color::LIME,
    color::BROWN,
    color::ORANGE,
    color::YELLOW,
    color::PINK,
    color::LICHEN,
    color::MAROON,
];

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
enum Square {
    Empty,
    Wall,
    Food,
    Mold {
        parent_dir: Option<Compass>,
        s: SpeciesID,
        spawn_time: usize,
    },
}
impl Colored for Square {
    fn color(&self) -> Color {
        use self::Square::*;
        match self {
            Empty => color::BLACK,
            Wall => color::WHITE,
            Food => color::GREEN,
            Mold { s, .. } => MOLD_COLORS[*s],
        }
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
enum GrowResult {
    Success {
        p: Point,
        s: SpeciesID,
        lifetime: usize,
        parent_dir: Compass,
    },
    Aged {
        lifetime: usize,
    },
    SpawnDied,
}

pub struct Foodburg {
    num_threads: usize,
    size: usize,
    grid: RwGrid<Square>,
    species: Vec<Species>,
    actors: Mutex<BinaryHeap<Actor>>,
    draw_path: Mutex<Option<SpeciesID>>,
    path: Mutex<Option<Vec<(Point, Color)>>>,
}
impl Petersburg for Foodburg {
    fn run(&self) {
        crossbeam::scope(|scope| {
            for i in 0..self.num_threads {
                scope.spawn(move |_| {
                    self.run_thread(i);
                });
            }
            scope.spawn(|_| {
                self.run_ui();
            });
        })
        .unwrap();
    }
    fn draw(&self, context: &Context) {
        self.grid.draw(context);

        self.path.lock().unwrap().as_ref().map(|path| {
            for (p, color) in path.into_iter() {
                context.set_color(*color);
                let (x, y) = (color::scale(p.0, self.size), color::scale(p.1, self.size));
                context.rectangle(
                    x,
                    y,
                    color::get_scale(self.size),
                    color::get_scale(self.size),
                );
                context.fill().unwrap();
            }
            ()
        });
    }
}
impl Foodburg {
    pub fn new(args: FoodburgArgs) -> Self {
        let grid = Self::grid_init(args.size, args.maze_args, args.wrapped);
        let species = Self::species_init(args.num_species, &grid);
        let actors = Self::actors_init(&species);
        Self {
            num_threads: args.num_threads,
            size: args.size,
            grid,
            species,
            actors,
            draw_path: Mutex::new(None),
            path: Mutex::new(None),
        }
    }
    fn grid_init(size: usize, maze_args: MazeArgs, wrapped: bool) -> RwGrid<Square> {
        let grid = RwGrid::<Square>::new(size, size, Square::Empty);
        let maze = Maze::new(size, wrapped, maze_args);
        for i in 0..size {
            for j in 0..size {
                let p = Point(i, j);
                grid.set_if(p, |_| maze.is_wall(p), Square::Wall);
            }
        }
        if !wrapped {
            for i in 0..size {
                let redge = Point(size - 1, i);
                let bedge = Point(i, size - 1);
                grid.set_if(redge, |_| true, Square::Wall);
                grid.set_if(bedge, |_| true, Square::Wall);
            }
        }
        grid
    }
    fn species_init(num_species: usize, grid: &RwGrid<Square>) -> Vec<Species> {
        if num_species > MOLD_COLORS.len() {
            panic!("More colors required for that many species")
        }
        let species = (0..num_species)
            .map(|s| {
                let mut p;
                'find_start: loop {
                    p = grid.rand();
                    if grid.set_if(
                        p,
                        |s| s == Square::Empty,
                        Square::Mold {
                            parent_dir: None,
                            s,
                            spawn_time: 0,
                        },
                    ) {
                        break 'find_start;
                    }
                }
                let start = p;
                let color = MOLD_COLORS[s];
                Species::new(s, color, start)
            })
            .collect();
        species
    }
    fn actors_init(species: &Vec<Species>) -> Mutex<BinaryHeap<Actor>> {
        let num_species = species.len();
        let mut queue = BinaryHeap::new();
        queue.push(Actor::FoodSpawn { time: 0 });
        for s in 0..num_species {
            queue.push(Actor::SporeSpawn {
                s,
                p: species[s].root,
                time: 0,
            });
        }
        Mutex::new(queue)
    }
    fn run_thread(&self, thread_id: usize) -> ! {
        use self::Actor::*;
        loop {
            let mut actors = self.actors.lock().unwrap();
            match actors.pop() {
                None => {
                    drop(actors);
                }
                Some(SporeSpawn {
                    s,
                    p: spawn_p,
                    time,
                }) => {
                    let mut queued_count = self.species[s].queued_count.lock().unwrap();
                    let mut active_count = self.species[s].active_count.lock().unwrap();
                    *queued_count -= 1;
                    *active_count += 1;
                    let true_queue = actors.count_species(s);
                    if true_queue != *queued_count {
                        println!("{thread_id} Popped {}, set count to {queued_count} queued, {active_count} active. True count {true_queue}", self.species[s]);
                    }
                    drop(queued_count);
                    drop(active_count);
                    drop(actors);

                    let result = self.attempt_grow(spawn_p, s);

                    let mut actors = self.actors.lock().unwrap();
                    let mut queued_count = self.species[s].queued_count.lock().unwrap();
                    let mut active_count = self.species[s].active_count.lock().unwrap();
                    *active_count -= 1;

                    match result {
                        GrowResult::SpawnDied => {
                            println!(
                                "Spawn died - this indicates (safely handled) thread collision."
                            );
                            actors.push(SporeSpawn {
                                s,
                                p: self.species[s].root,
                                time,
                            });
                            *queued_count += 1;
                            drop(actors)
                        }
                        GrowResult::Aged { lifetime } => {
                            let time = time + lifetime;
                            if *queued_count <= 0 && *active_count <= 0 {
                                actors.push(SporeSpawn {
                                    s,
                                    p: self.species[s].root,
                                    time,
                                });
                                *queued_count += 1;
                                drop(actors);
                            }
                        }
                        GrowResult::Success {
                            p,
                            s,
                            lifetime,
                            parent_dir,
                        } => {
                            let time = time + lifetime;
                            let potential_square = Square::Mold {
                                s,
                                parent_dir: Some(parent_dir),
                                spawn_time: time,
                            };
                            if self.grid.set_if(
                                p,
                                |square| square == Square::Empty,
                                potential_square,
                            ) {
                                if *queued_count < MAX_LIVING {
                                    for _ in 0..CHILD_COUNT {
                                        actors.push(SporeSpawn { s, p, time });
                                        *queued_count += 1;
                                    }
                                    drop(actors);
                                }
                            } else {
                                println!("{} from {spawn_p}, landed on {p}. Attempted growth on non-empty square after {lifetime} steps. Oh well.", self.species[s]);
                                println!("(BTW, the queue count is {queued_count} and this is thread {thread_id}");
                                actors.push(SporeSpawn {
                                    s,
                                    p: self.species[s].root,
                                    time,
                                });
                                *queued_count += 1;
                                drop(actors)
                            }
                        }
                    }
                }
                Some(FoodSpawn { time }) => {
                    actors.push(FoodSpawn {
                        time: time + FOOD_SPAWN_RATE,
                    });
                    drop(actors);
                    self.grow_food();
                }
            }
        }
    }
    #[allow(dead_code, unused_mut)]
    fn attempt_grow_pathed(&self, spawn_point: Point, s: SpeciesID) -> GrowResult {
        let (mut p, s) = match self.rand_descendent_leaf(spawn_point, s) {
            Some(start_p) => match self.grid.get(start_p) {
                Square::Mold { s, .. } => (start_p, s),
                _ => return GrowResult::SpawnDied,
            },
            _ => return GrowResult::SpawnDied,
        };
        let s_draw_path = self.draw_path.lock().unwrap();
        let draw_path = *s_draw_path == Some(s);
        drop(s_draw_path);
        let mut path: Vec<(Point, Color)> = Vec::new();
        let mut dir = Compass::rand();
        let mut lifetime = 0;
        let mut step = |n: Neighborhood<Square>| {
            if draw_path {
                path.push((p, self.species[s].color))
            }
            lifetime += 1;
            if lifetime >= MAX_SPORE_LIFE {
                return StepResult::Die;
            }
            if n.c == Square::Food {
                return StepResult::Stick(Square::Empty);
            }
            StepResult::Step(Some(dir))
        };
        match self.grid.path(p, &mut step) {
            PathResult::Stuck(p) => GrowResult::Success {
                p,
                s,
                lifetime,
                parent_dir: dir,
            },
            PathResult::Died(_) => GrowResult::Aged { lifetime },
        }
    }
    fn attempt_grow(&self, spawn_point: Point, s: SpeciesID) -> GrowResult {
        let (mut p, s) = match self.rand_descendent_leaf(spawn_point, s) {
            Some(start_p) => match self.grid.get(start_p) {
                Square::Mold { s, .. } => (start_p, s),
                _ => return GrowResult::SpawnDied,
            },
            _ => return GrowResult::SpawnDied,
        };
        let s_draw_path = self.draw_path.lock().unwrap();
        let draw_path = *s_draw_path == Some(s);
        drop(s_draw_path);
        let mut path = Vec::new();

        let mut dir = Compass::rand();
        let mut lifetime = 0;
        'seek_food: loop {
            if draw_path {
                path.push((p, self.species[s].color))
            }
            lifetime += 1;
            if lifetime >= MAX_SPORE_LIFE {
                if draw_path {
                    let s_draw_path = self.draw_path.lock().unwrap();
                    let draw_path = *s_draw_path == Some(s);
                    let mut s_path = self.path.lock().unwrap();
                    if draw_path {
                        *s_path = Some(path)
                    };
                    drop(s_draw_path);
                    drop(s_path);
                }
                return GrowResult::Aged { lifetime };
            };
            let neighbors = Compass::all()
                .into_iter()
                .map(|n_dir| self.grid.step(p, n_dir));
            for neighbor in neighbors {
                if self
                    .grid
                    .set_if(neighbor, |s| s == Square::Food, Square::Empty)
                {
                    break 'seek_food;
                }
            }
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }
        loop {
            if draw_path {
                path.push((p, self.species[s].color))
            }
            lifetime += 1;
            if lifetime >= MAX_SPORE_LIFE {
                if draw_path {
                    let s_draw_path = self.draw_path.lock().unwrap();
                    let draw_path = *s_draw_path == Some(s);
                    let mut s_path = self.path.lock().unwrap();
                    if draw_path {
                        *s_path = Some(path)
                    };
                    drop(s_draw_path);
                    drop(s_path);
                }
                return GrowResult::Aged { lifetime };
            };
            for facing_dir in Compass::all().into_iter() {
                match self.grid.get(self.grid.step(p, facing_dir)) {
                    Square::Mold { s: neighbor_s, .. } if neighbor_s == s => {
                        return GrowResult::Success {
                            p,
                            s,
                            lifetime,
                            parent_dir: facing_dir,
                        };
                    }
                    _ => (),
                }
            }
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }

    }
    fn grow_food(&self) {
        let mut p;
        'find_start: loop {
            p = self.grid.rand();
            if self.grid.get(p) == Square::Empty {
                break 'find_start;
            }
        }
        let mut dir = Compass::rand();
        let mut ripeness = 0;
        'ripen: loop {
            ripeness += 1;
            if ripeness > RIPE_AGE {
                break 'ripen;
            };
            let hood = self.grid.get_neighborhood(p);
            let moldy = hood.into_iter().any(|s| match s {
                Square::Mold { .. } => true,
                _ => false,
            });
            if moldy {
                return;
            };
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }
        'seek: loop {
            ripeness += 1;
            if ripeness > ROT_AGE {
                break 'seek;
            };
            let hood = self.grid.get_neighborhood(p);
            let n_count = hood
                .into_iter()
                .filter(|s| *s == Square::Wall || *s == Square::Food)
                .count();
            if n_count >= 4 {
                self.grid.set_if(p, |s| s == Square::Empty, Square::Food);
                break 'seek;
            };
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }
    }
    fn rand_descendent_leaf(&self, p: Point, s: SpeciesID) -> Option<Point> {
        use rand::seq::SliceRandom;
        self.get_descendant_leaves(p, s)
            .choose(&mut rand::thread_rng())
            .map(|pair| pair.0)
    }
    fn get_children(&self, p: Point, s: SpeciesID) -> Vec<(Point, usize)> {
        match self.grid.get(p) {
            Square::Mold { s: found_s, .. } if s == found_s => Compass::all()
                .iter()
                .map(|dir| {
                    let neighbor_p = self.grid.step(p, *dir);
                    let reverse_dir = Some(dir.reverse());
                    match self.grid.get(neighbor_p) {
                        Square::Mold {
                            s: neighbor_s,
                            parent_dir,
                            spawn_time,
                        } if neighbor_s == s && parent_dir == reverse_dir => {
                            Some((neighbor_p, spawn_time))
                        }
                        _ => None,
                    }
                })
                .flatten()
                .collect(),
            _ => Vec::new(),
        }
    }

    fn get_descendant_leaves(&self, p: Point, s: SpeciesID) -> Vec<(Point, usize)> {
        let mut to_process = vec![p];
        let mut found_leaves: Vec<(Point, usize)> = vec![];
        while let Some(candidate) = to_process.pop() {
            match self.grid.get(candidate) {
                Square::Mold {
                    s: found_s,
                    spawn_time,
                    ..
                } if s == found_s => {
                    let children = self.get_children(candidate, s);
                    if children.is_empty() {
                        found_leaves.push((candidate, spawn_time))
                    } else {
                        children.iter().for_each(|(child_p, _)| {
                            to_process.push(*child_p);
                        });
                    }
                }
                _ => {}
            }
        }
        found_leaves
    }

    fn run_ui(&self) {
        loop {
            let mut line = String::new();
            let _b1 = std::io::stdin().read_line(&mut line).unwrap();
            if line.starts_with("sound off") {
                for elem in self.species.iter() {
                    println!(
                        "{elem}:
                    \t root: {}
                    \t queued: {}
                    \t active: {}",
                        elem.root,
                        elem.queued_count.lock().unwrap(),
                        elem.active_count.lock().unwrap()
                    );
                }
            } else if line.starts_with("queue") {
                use self::Actor::*;
                let actors = self.actors.lock().unwrap();
                let cloned = actors.clone();
                drop(actors);
                for elem in cloned.iter() {
                    match elem {
                        FoodSpawn { time } => println!("Food should spawn at {time}"),
                        SporeSpawn { s, p, time } => {
                            println!("{} should spawn from {p} at {time}", self.species[*s]);
                        }
                    }
                }
            } else if line.starts_with("paths off") {
                let mut path_species = self.draw_path.lock().unwrap();
                *path_species = None;
                drop(path_species);
                let mut path = self.path.lock().unwrap();
                *path = None;
                drop(path);
                println!("Paths should be dropped");
            } else if line.starts_with("path for ") {
                let index_string = line.split(" ").last().unwrap().trim();
                let mut path_species = self.draw_path.lock().unwrap();
                *path_species = index_string.parse().ok();
                drop(path_species);
            } else {
                println!("I didn't understand: {line}");
            }
        }
    }
    fn update_dir(dir: &mut Compass) {
        let roll = roll::usize(32);
        if roll == 0 {
            *dir = dir.left();
        } else if roll == 1 {
            *dir = dir.right();
        }
    }
    fn bounce_move(&self, p: &mut Point, dir: &mut Compass) {
        let straight = self.is_empty(self.grid.step(*p, *dir));
        let left = self.is_empty(self.grid.step(*p, dir.left()));
        let right = self.is_empty(self.grid.step(*p, dir.right()));
        if straight && (left || right) {
            *p = self.grid.step(*p, *dir);
        } else if !straight && right {
            *dir = dir.right().right();
        } else if !straight && left {
            *dir = dir.left().left();
        } else {
            *dir = dir.reverse();
        }
    }
    fn is_empty(&self, p: Point) -> bool {
        self.grid.get(p) == Square::Empty
    }
}
