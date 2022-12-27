use std::collections::BinaryHeap;
use std::sync::{Mutex};

use cairo::Context;
use general::burg::Burg;
use general::grid::{Point, RwGrid, Grid};
use general::constants::*;
use general::direction::{*};
#[allow(unused_imports)]
use general::utils::*;
use general::draw_utils::*;
use crate::general::maze::Maze;

use super::species::*;
use super::actor_queue::*;

const MAX_SPORE_LIFE : usize = 5_000;
const RIPE_AGE : usize = 100;
const ROT_AGE : usize = 5_000;
const FOOD_SPAWN_RATE : usize = 50;
const MAX_LIVING : usize = 400;
const CHILD_COUNT : usize= 4;

static MOLD_COLORS : [Color; 11] = [
    RED, BLUE, PURPLE, TEAL, LIME, BROWN, ORANGE, YELLOW, PINK, LICHEN, MAROON
];


#[derive(Copy, Clone, PartialEq, Hash, Eq)]
enum Square{
    Empty,
    Wall,
    Food,
    Mold{parent_dir: Option<Compass>, s : SpeciesID, spawn_time : usize}
}
impl Colored for Square{
    fn color(&self) -> Color {
        use self::Square::*;
        match self {
            Empty => BLACK,
            Wall => WHITE,
            Food => GREEN,
            Mold{s, ..} => MOLD_COLORS[*s]
        }
    }
}


#[derive(Hash, Eq, PartialEq, Copy, Clone)]
enum GrowResult{
    Success{p : Point, s : SpeciesID, lifetime : usize, parent_dir : Compass},
    Aged{lifetime : usize},
    SpawnDied
}
 
pub struct Foodburg{
    size : usize,
    grid : RwGrid<Square>,
    species : Vec<Species>,
    actors : Mutex<BinaryHeap<Actor>>,
    draw_path : Mutex<Option<SpeciesID>>,
    path : Mutex<Option<Vec<(Point, Color)>>>
}
#[derive(Copy, Clone)]
pub struct Config{
    pub size : usize,
    pub maze_squares : usize,
    pub num_species : usize,
    pub openness : f64,
    pub wrapped : bool
}
impl Burg for Foodburg{
    type Config = Config;
    fn new(c : Config) -> Self{
        let grid = Self::grid_init(&c);
        let species = Self::species_init(&c, &grid);
        let actors = Self::actors_init(&c, &species);
        Self{size : c.size, grid, species, actors, draw_path : Mutex::new(None), path : Mutex::new(None)}
    }
    fn run(&self){
        crossbeam::scope(|scope|{
            for i in 0 .. NUM_THREADS{
                scope.spawn(move |_|{
                    self.run_thread(i);
                });
            };
            scope.spawn(|_|{
                self.run_ui();
            });
        }).unwrap();
    }
    fn draw(&self, context : &Context){
            self.grid.draw(context);

            self.path.lock().unwrap().as_ref().map(|path|{
                for (p, color) in path.into_iter(){
                    context.set_color(*color);
                    let (x, y) = (scale(p.0, self.size), scale(p.1, self.size));
                    context.rectangle(x, y, get_scale(self.size), get_scale(self.size));
                    context.fill().unwrap();
                };
                ()
            });

    }


}
impl Foodburg{
    fn grid_init(c : &Config) -> RwGrid<Square>{
        let grid = RwGrid::<Square> :: new(c.size, c.size, Square::Empty);
        let mut maze = Maze::random_pathed(c.maze_squares, c.maze_squares, c.size/c.maze_squares, c.wrapped);
        maze.remove_walls(c.openness);
        for i in 0 .. c.size{
            for j in 0 .. c.size{
                let p = Point(i, j);
                if maze.is_wall(p){
                    grid.set(p, Square::Wall)    
                }
            }

        }
        if !c.wrapped{
            for i in 0 .. c.size{
                let redge = Point(c.size-1, i);
                let bedge = Point(i, c.size-1);
                grid.set(redge, Square::Wall);    
                grid.set(bedge, Square::Wall);    
            }
        }
        grid
    }
    fn species_init(c : &Config, grid : &RwGrid<Square>) -> Vec<Species>{
        if c.num_species > MOLD_COLORS.len() {panic!("More colors required for that many species")}
        let species = (0 .. c.num_species).map(|s|{
            let mut p;
            'find_start : loop{
                p = grid.rand();
                if grid.get(p) == Square::Empty {
                    break 'find_start
                }
            }
            let start = p;
            let color = MOLD_COLORS[s];
            grid.set(start, Square::Mold{parent_dir : None, s, spawn_time : 0});
            Species::new(s, color, start)
        }).collect();
        species
    }
    fn actors_init(c: &Config, species : &Vec<Species>) -> Mutex<BinaryHeap<Actor>> {
        //panic!("Unimplemented actors_int")
        let mut queue = BinaryHeap::new();
        queue.push(Actor::FoodSpawn{time : 0});
        for s in 0 .. c.num_species{
            queue.push(Actor::SporeSpawn{s, p : species[s].root, time : 0});
        }
        Mutex::new(queue)
    }
    fn run_thread(&self, thread_id : usize) -> !{
        use self::Actor::*;
        loop{
            let mut actors = self.actors.lock().unwrap();
            match actors.pop(){
                None => {
                    drop(actors);
                },
                Some(SporeSpawn{s, p : spawn_p, time}) => {

                    let mut queued_count = self.species[s].queued_count.lock().unwrap();
                    let mut active_count = self.species[s].active_count.lock().unwrap();
                    *queued_count -= 1;
                    *active_count += 1;
                    let true_queue = actors.count_species(s);
                    if true_queue != *queued_count{
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

                    match result{
                        GrowResult::SpawnDied => {
                            //println!("Spawn died - this indicates (safely handled) thread collision.");
                            actors.push(SporeSpawn{s, p : self.species[s].root, time});
                            *queued_count += 1;
                            drop(actors)
                        },
                        GrowResult::Aged{lifetime} => {
                            
                            let time = time + lifetime;
                            if *queued_count <= 0  && *active_count <= 0{
                                actors.push(SporeSpawn{s, p: self.species[s].root, time});
                                *queued_count += 1;
                                drop(actors);
                            }
                        },
                        GrowResult::Success{p, s, lifetime, parent_dir} => {
                            let time = time + lifetime;
                            let potential_square = Square::Mold{s, parent_dir : Some(parent_dir), spawn_time : time};
                            if self.grid.set_if(p, |square|{square == Square::Empty}, potential_square){
                                if *queued_count < MAX_LIVING{

                                    for _ in 0 .. CHILD_COUNT{
                                        actors.push(SporeSpawn{s, p, time});
                                        *queued_count += 1;
                                    }
                                    drop(actors);
                                }
                            } else{
                                println!("{} from {spawn_p}, landed on {p}. Attempted growth on non-empty square. Oh well.", self.species[s]);
                                actors.push(SporeSpawn{s, p : self.species[s].root, time});
                                *queued_count+=1;
                                drop(actors)
                            }
                        }
                    }
                },
                Some(FoodSpawn{time}) => {
                    actors.push(FoodSpawn{time : time + FOOD_SPAWN_RATE});
                    drop(actors);
                    self.grow_food();
                },
            }

        }
    }
    fn attempt_grow(&self, spawn_point : Point, s : SpeciesID) -> GrowResult{
        let (mut p, s) = match self.rand_descendent_leaf(spawn_point, s){
            Some(start_p) => {
                match self.grid.get(start_p){
                    Square::Mold{s, ..} => (start_p, s),
                    _ => {return GrowResult::SpawnDied}
                }
            },
            _ => {return GrowResult::SpawnDied}
        };
        let s_draw_path = self.draw_path.lock().unwrap();
        let draw_path = *s_draw_path == Some(s);
        drop(s_draw_path);
        let mut path = Vec::new();

        let mut dir = Compass::rand();
        let mut lifetime = 0;
        'seek_food : loop {
            if draw_path{
                path.push((p, self.species[s].color))
            }
            lifetime += 1;
            if lifetime >= MAX_SPORE_LIFE {
                if draw_path{
                    let s_draw_path = self.draw_path.lock().unwrap();
                    let draw_path = *s_draw_path == Some(s);
                    let mut s_path = self.path.lock().unwrap();
                    if draw_path {*s_path = Some(path)};
                    drop(s_draw_path);
                    drop(s_path);
                }
                return GrowResult::Aged{lifetime};
            };
            let neighbors = Compass::all().into_iter().map(|n_dir| {self.grid.step(p, n_dir)});
            for neighbor in neighbors{
                if self.grid.set_if(neighbor, |s|{s == Square::Food}, Square::Empty){
                    break 'seek_food;
                }
            }
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }
        loop {
            if draw_path{
                path.push((p, self.species[s].color))
            }
            lifetime += 1;
            if lifetime >= MAX_SPORE_LIFE {
                if draw_path{
                    let s_draw_path = self.draw_path.lock().unwrap();
                    let draw_path = *s_draw_path == Some(s);
                    let mut s_path = self.path.lock().unwrap();
                    if draw_path {*s_path = Some(path)};
                    drop(s_draw_path);
                    drop(s_path);
                }
                return GrowResult::Aged{lifetime};
            };
            for facing_dir in Compass::all().into_iter(){
                match self.grid.get(self.grid.step(p, facing_dir)){
                    Square::Mold{s : neighbor_s, ..} if neighbor_s == s => {
                        return GrowResult::Success{p, s, lifetime, parent_dir : facing_dir};
                    },
                    _ => ()
                }
            }
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }


        // let genes = species.get_genotype();
        // let steps = genes.get_fenotype();
       
    }
    fn grow_food(&self){
        let mut p;
        'find_start : loop{
            p = self.grid.rand();
            if self.grid.get(p) == Square::Empty {
                break 'find_start
            }
        }
        let mut dir = Compass::rand();
        let mut ripeness = 0;
        'ripen : loop {
            ripeness += 1;
            if ripeness > RIPE_AGE {break 'ripen};
            let ring = self.grid.get_ring(p);
            let mold_count = ring.count_matching(|s|{
                match s {
                    Square::Mold{..} => true,
                    _ => false,
                }
            });
            if mold_count > 0 {
                return
            };
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }
        'seek : loop{
            ripeness += 1;
            if ripeness > ROT_AGE {break 'seek};
            let ring = self.grid.get_ring(p);
            let n_count = ring.count_matching(|s|{
                s == Square::Wall || s == Square::Food
            });
            if n_count >= 4 {
                self.grid.set_if(p, |s|{s==Square::Empty}, Square::Food);
                self.grid.set(p, Square::Food);
                break 'seek
            };
            Self::update_dir(&mut dir);
            self.bounce_move(&mut p, &mut dir);
        }
        //Check stuck?
        //Pick dir
        //Bounce? | Move
        //Move
    }
    #[allow(unused)]
    fn rand_pathed_descendent(&self, p : Point, s : SpeciesID) -> Option<Point>{
        use rand::seq::SliceRandom; 
        match self.grid.get(p){
            Square::Mold{s : found_s, ..} if s == found_s => {
                let points_and_times : Vec<(Point, usize)> = self.get_children(p, s);
                let random_child = points_and_times.choose(&mut rand::thread_rng());
                random_child.and_then(|pair|{self.rand_pathed_descendent(pair.0, s)}).or(Some(p))
            },
            _ => {None}
        }
    }
    #[allow(unused)]
    fn rand_descendent_leaf(&self, p : Point, s : SpeciesID) -> Option<Point>{
        use rand::seq::SliceRandom;
        self.get_descendant_leaves(p, s)
            .choose(&mut rand::thread_rng())
            .map(|pair|{pair.0})
    }
    #[allow(unused)]
    fn get_freshest(&self, p : Point, s : SpeciesID) -> Option<Point>{
        match self.grid.get(p){
            Square::Mold{s : found_s, ..} if s == found_s => {
                let points_and_times : Vec<(Point, usize)> = self.get_children(p, s);
                let least = points_and_times.iter().min_by(|a, b|{a.1.cmp(&b.1)});
                least.and_then(|pair|{self.get_freshest(pair.0, s)}).or(Some(p))
            },
            _ => {None}
        }
    }
    fn get_children(&self, p : Point, s : SpeciesID) -> Vec<(Point, usize)>{
        match self.grid.get(p){
            Square::Mold{s :found_s, ..} if s == found_s => {
                Compass::all().iter().map(|dir|{
                    let neighbor_p = self.grid.step(p, *dir);
                    let reverse_dir = Some(dir.reverse());
                    match self.grid.get(neighbor_p){
                        Square::Mold{s : neighbor_s, parent_dir, spawn_time}
                            if neighbor_s == s && parent_dir == reverse_dir => {Some((neighbor_p, spawn_time))},
                        _ => {None}
                    }
                }).flatten().collect()
            },
            _ => Vec::new()
        }
    }
    fn get_descendant_leaves(&self, p : Point, s : SpeciesID) -> Vec<(Point, usize)>{
        match self.grid.get(p){
            Square::Mold{s :found_s, spawn_time, ..} if s == found_s => {
                let children = self.get_children(p, s);
                if children.is_empty() {vec![(p, spawn_time)]}
                else {
                    children.iter().map(|child_p|{self.get_descendant_leaves(child_p.0, s)}).flatten().collect()
                }
            },
            _ => Vec::new()
        }
    }

    fn run_ui(&self){
        loop{
            let mut line = String::new();
            let _b1 = std::io::stdin().read_line(&mut line).unwrap();
            if line.starts_with("sound off"){
                for elem in self.species.iter(){
                    println!("{elem}:
                    \t root: {}
                    \t queued: {}
                    \t active: {}", elem.root, elem.queued_count.lock().unwrap(), elem.active_count.lock().unwrap());
                }
            } 
            else if line.starts_with("queue"){
                use self::Actor::*;
                let actors = self.actors.lock().unwrap();
                let cloned = actors.clone();
                drop(actors);
                for elem in cloned.iter(){
                    match elem{
                        FoodSpawn { time } => println!("Food should spawn at {time}"),
                        SporeSpawn { s, p, time } => {println!("{} should spawn from {p} at {time}", self.species[*s]);}
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
            } else if line.starts_with("path for "){
                let index_string = line.split(" ").last().unwrap().trim();
                let mut path_species = self.draw_path.lock().unwrap();
                *path_species = index_string.parse().ok();
                drop(path_species);
            } else {
                println!("I didn't understand: {line}");
            }
        }
    }
    fn update_dir(dir : &mut Compass){
        let roll = roll::usize(32);
        if roll == 0 {*dir = dir.left();}
        else if roll == 1  {*dir = dir.right();}
    }
    fn bounce_move(&self, p: &mut Point, dir : &mut Compass){
        let straight = self.is_empty(self.grid.step(*p, *dir));
        let left = self.is_empty(self.grid.step(*p, dir.left()));
        let right = self.is_empty(self.grid.step(*p, dir.right()));
        if straight && (left || right){
            *p = self.grid.step(*p, *dir);
        }
        else if !straight && right {
            *dir = dir.right().right();
        }
        else if !straight && left {
            *dir = dir.left().left();
        }
        else {
            *dir = dir.reverse();
        }
    }
    fn is_empty(&self, p : Point) -> bool{
        self.grid.get(p) == Square::Empty
    }
}