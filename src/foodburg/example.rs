#![allow(dead_code)]

pub trait Simulation{
    fn new()-> Self;
    fn run(&self);
    fn print_state(&self);
}

use std::sync::Mutex;

struct Species{
    name :  &'static str,
    population : Mutex<i32>
}
impl Species{
    fn new(name : &'static str) -> Self{
        Species{name, population : Mutex::new(0)}
    }
    fn breed(&self, increase : i32){
        let mut pop = self.population.lock().unwrap();
        *pop += increase;
    }
}
impl std::fmt::Display for Species{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pop = self.population.lock().unwrap();
        write!(f, "{}: {}", self.name, pop)
    }
}

struct Simple{
    dogs : Species,
    cats : Species
}
impl Simulation for Simple{
    fn new() -> Self{
        let dogs = Species::new("dogs");
        let cats = Species::new("cats");
        Self{dogs, cats}
    }
    fn run(&self){
        self.dogs.breed(1);
        self.cats.breed(2);
    }
    fn print_state(&self){
        let dogs = &self.dogs;
        let cats = &self.cats;
        println!("{dogs}, {cats}")
    }
}
pub fn simple_top_level(){
    let sim = Simple::new();
    for _ in 0 .. 10{
        sim.run();
        sim.print_state();
    }
}

// struct SelfReferential<'a>{
//     species : [Species; 2],
//     grid : Mutex<Vec<&'a Species>>
// }
// impl<'a> Simulation for SelfReferential<'a>{
//     fn new() -> Self{
//         let dogs = Species::new("dogs");
//         let cats = Species::new("cats");
//         let species = [dogs, cats];
//         let grid = Mutex::new(vec!(&species[0]; 1000));
//         Self{species, grid}
//         //Self{species, grid}
//     }
//     fn run(&self){
//         let mut grid = self.grid.lock().unwrap();
//         let random_location = rand::random::<usize>() % grid.len();
//         let old_species = grid[random_location];
//         old_species.breed(-1);
//         let random_species: &'a Species = &self.species[rand::random::<usize>() % 2];
//         grid[random_location] = random_species;
//         random_species.breed(1);
//     }
//     fn print_state(&self){
//         for species in &self.species{
//             println!("{species}")
//         }
//     }

// }


// pub fn self_referential_top_level(){
//     let sim = Simple::new();
//     for _ in 0 .. 10{
//         sim.run();
//         sim.print_state();
//     }
// }

struct TwoLayer<'a>{
    species : &'a [Species; 2], //Now just holds a reference rather than data
    grid : Mutex<Vec<&'a Species>>
}
impl<'a> TwoLayer<'a>{
    pub fn make_species() -> [Species; 2]{
                let dogs = Species::new("dogs");
                let cats = Species::new("cats");
                [dogs, cats]
    }
    pub fn new(species : &'a[Species; 2]) -> Self{
        let grid = Mutex::new(vec!(&species[0]; 1000));
        species[0].breed(1000);
        Self{species, grid}
    }
    pub fn run(&self){
        let mut grid = self.grid.lock().unwrap();
        let random_location = rand::random::<usize>() % grid.len();
        let old_species = grid[random_location];
        old_species.breed(-1);
        let random_species = &self.species[rand::random::<usize>() % 2];
        grid[random_location] = random_species;
        random_species.breed(1);
    }
    pub fn print_state(&self){
        for species in self.species{
            println!("{species}")
        }
    }
}


pub fn two_layer_top_level(){
    let species = TwoLayer::make_species();
    let sim = TwoLayer::new(&species);
    for _ in 0 .. 10{
        sim.run();
        sim.print_state();
    }
}