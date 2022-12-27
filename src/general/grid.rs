use std::{fmt, sync::{RwLock, RwLockWriteGuard}, collections::HashSet};
use cairo::Context;
use super::{utils::*, direction::*};
use super::draw_utils::*;

const REGIONS_PER_DIMENSION : usize = 8;
const TOTAL_REGIONS : usize = REGIONS_PER_DIMENSION * REGIONS_PER_DIMENSION;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point(pub usize, pub usize);


pub struct Neighborhood<T>{
    pub c : T,
    pub n :T,
    pub ne :T,
    pub e :T,
    pub se :T,
    pub s :T,
    pub sw :T,
    pub w :T,
    pub nw : T

}
impl<T : Copy> Neighborhood<T>{
    pub fn from_dir(&self, dir : Compass) -> T{
    use self::Compass::*;
        match dir{
            N => self.n,
            NE => self.ne,
            E => self.e,
            SE => self.se,
            S => self.s,
            SW => self.sw,
            W => self.w,
            NW => self.nw
        }
    }
}

pub struct Ring<T>{
    pub n :T,
    pub ne :T,
    pub e :T,
    pub se :T,
    pub s :T,
    pub sw :T,
    pub w :T,
    pub nw : T
}

impl<T : Copy> Ring<T>{
    pub fn count_matching<F>(&self, f : F) -> u8 where
        F : Fn(T) -> bool {
            let mut found = 0;
            let all = vec![self.n, self.ne, self.e, self.se, self.s, self.sw, self.w, self.nw];
            for ele in all.into_iter() {
                if f(ele) {found = found + 1}
            };
            found
        }
}

impl Point {
    #[inline(always)]
    pub fn distance(p1: Self, p2: Self) -> f64 {
        let (x1, y1, x2, y2) = (p1.0 as i32, p1.1 as i32, p2.0 as i32, p2.1 as i32);
        (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt()
    }
    #[inline(always)]
    pub fn x(&self) -> usize {
        self.0
    }
    #[inline(always)]
    pub fn y(&self) -> usize {
        self.1
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}
impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

pub enum StepResult<T>{
    Step(Option<Compass>),
    Stick(T, fn(T) -> bool),
    Die
}
pub enum PathResult{
    Stuck(Point),
    Died(Point),
}


pub trait Grid<T : Copy + Colored>{
    fn get(&self, p : Point) -> T;
    fn get_ring(&self, p : Point) -> Ring<T>;
    fn set(&self, p : Point, value: T);
    fn set_if<F>(&self, p : Point, f : F, value : T) -> bool where
        F : Fn(T) -> bool;
    fn rand(&self) -> Point;
    fn step<D: Direction>(&self, pt: Point, dir: D) -> Point;
    fn draw(&self, context : &Context);
    fn path<F, State>(&self, p : Point, state : State, step : F) -> PathResult where
        F : Fn(Neighborhood<T>, State) -> StepResult<T>;
    //fn clone(&self) -> Self;
}

pub struct RwGrid<T>{
    width : usize,
    height : usize,
    region_size : usize,
    regions : [RwLock<Vec<T>>; TOTAL_REGIONS]
}

impl<T: Copy + Colored> Grid<T> for RwGrid<T>{

    fn step<D: Direction>(&self, pt: Point, dir: D) -> Point {
        let (x, y) = (pt.0 as i32, pt.1 as i32);
        let (xs, ys) = dir.step();
        let result = Point(modulo(x + xs, self.width), modulo(y + ys, self.height));
        result
    }

    fn get(&self, p : Point) -> T{
        let (region_index, index_in_region) = self.map_coordinates(p);
        let region = self.regions[region_index].read().unwrap();
        region[index_in_region]
    }
    fn get_ring(&self, p : Point) -> Ring<T>{
        let p  = self.fix(p);
        Ring{
            n : self.get(self.step(p, Compass::N)),
            ne : self.get(self.step(p, Compass::NE)),
            e : self.get(self.step(p, Compass::E)),
            se : self.get(self.step(p, Compass::SE)),
            s : self.get(self.step(p, Compass::S)),
            sw : self.get(self.step(p, Compass::SW)),
            w : self.get(self.step(p, Compass::W)),
            nw : self.get(self.step(p, Compass::NW)),
        }
    }
    fn set(&self, p : Point, value: T){
        let (region_index, index_in_region) = self.map_coordinates(p);
        let mut region = self.regions[region_index].write().unwrap();
        region[index_in_region] = value;
    }
    fn set_if<F>(&self, p : Point, f : F, value : T) -> bool where
        F : Fn(T) -> bool{
            let (region_index, index_in_region) = self.map_coordinates(p);
            let mut region = self.regions[region_index].write().unwrap();
            let pre_existing = region[index_in_region];
            if f(pre_existing){
                region[index_in_region] = value;
                true
            } else {false}
        }
    fn rand(&self) -> Point{
        Point(roll::usize(self.width), roll::usize(self.height))
    }


    fn path<F, State>(&self, p : Point, state : State, step : F) -> PathResult where
        F : Fn(Neighborhood<T>, State) -> StepResult<T>{
        let centered : bool = true;
        'outer : loop {
            if centered {
                //grab appropriate lock
                'centered : loop {
                    if !centered {
                        break 'centered;
                    }
                }
            } else if !centered {
                //grab all locks?
                'uncentered : loop{
                    if centered{
                        break 'uncentered
                    }
                }
            }
        }
        
    }

    fn draw(&self, context : &Context){
        let (region_width, region_height) = (self.width/REGIONS_PER_DIMENSION, self.height/REGIONS_PER_DIMENSION);
        
        for i in 0 .. TOTAL_REGIONS{
            let(corner_x, corner_y) = ((i % REGIONS_PER_DIMENSION) * region_width, (i / REGIONS_PER_DIMENSION) * region_height);
            let region = self.regions[i].read().unwrap();
            for j in 0 .. self.region_size {
                let (x, y) = ( corner_x + j % region_width, corner_y + j / region_width);
                let (scaled_x, scaled_y) = (scale(x, self.width), scale(y, self.height));
                let color = region[j].color();
                if color != BLACK {
                    context.set_color(color);
                    context.rectangle(scaled_x, scaled_y, get_scale(self.width), get_scale(self.height));
                    context.fill().unwrap();
                }
            }
        }




        // for i in 0 .. self.width{
        //     for j in 0 .. self.height{
        //         let (region_i, i_region) = self.map_coordinates(Point(i, j));
        //         let (x, y) = (scale(i, self.width), scale(j, self.height));
        //         context.move_to(x, y);
        //         let red = 0.0;//(region_i % 3) as f64 / (3.0);
        //         let green = (i_region % (self.width/REGIONS_PER_DIMENSION)) as f64 / self.region_size as f64;
        //         let blue = 0.0;
        //         context.set_source_rgb(red, green, blue);
        //         context.rectangle(x, y, get_scale(self.width), get_scale(self.height));
        //         context.fill().unwrap();
        //     }
        // }
    }



}

impl<T : Copy + Colored> RwGrid<T>{

    // fn locked_hood(&self, p : Point) -> (Neighborhood<T>, HashSet<RwLockWriteGuard<T>>){
        
    // }

    pub fn draw_debug(&self, context : &Context){
        for i in 0 .. self.width{
            for j in 0 .. self.height{
                let p = Point(i, j);
                let color = self.get(p).color();
                if color != BLACK {
                    context.set_color(color);
                    context.rectangle(scale(i, self.width), scale(j, self.height), get_scale(self.width), get_scale(self.height));
                    context.fill().unwrap();
                }

            }
        }
    }
    pub fn new(width : usize, height : usize, default : T) -> Self{
        if width % REGIONS_PER_DIMENSION != 0 {panic!("Width ({}) must be a multiple of {}", width, REGIONS_PER_DIMENSION)};
        if height % REGIONS_PER_DIMENSION != 0 {panic!("Height ({}) must be a multiple of {}", height, REGIONS_PER_DIMENSION)};
        let region_size = height * width / TOTAL_REGIONS;
        let regions = [0; TOTAL_REGIONS].map(|_|{
            RwLock::new(vec![default; region_size])
            });
        RwGrid{width, height, region_size, regions}

    }
    fn map_coordinates(&self, p : Point) -> (usize, usize){
        let Point(x, y) = self.fix(p);
        let (region_width, region_height) = (self.width / REGIONS_PER_DIMENSION, self.height/REGIONS_PER_DIMENSION);
        let (target_square_x, target_square_y) = (x/region_width, y/region_height);
        let target_square_i = target_square_y * REGIONS_PER_DIMENSION + target_square_x;
        let (x_in_square, y_in_square) = (x % region_width, y % region_width);
        let index_in_square = y_in_square * region_width + x_in_square; (target_square_i, index_in_square)
    }
    fn fix(&self, p: Point) -> Point{
        let Point(x, y) = p;
        Point(modulo(x as i32, self.width), modulo(y as i32, self.height))
    }

}