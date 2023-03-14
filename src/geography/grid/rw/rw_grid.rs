#[allow(unused_imports)]
use super::{
    super::{super::*, *},
    *,
};
use std::sync::RwLock;

//These are consts because they're really for perf tuning, and they're here because their efficacy is impl-specific.
pub(super) const REGIONS_PER_DIMENSION: usize = 8;
pub(super) const TOTAL_REGIONS: usize = REGIONS_PER_DIMENSION * REGIONS_PER_DIMENSION;
pub struct RwGrid<T> {
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) region_size: usize,
    pub(super) regions: [RwLock<Vec<T>>; TOTAL_REGIONS],
}

impl<T: Copy> Grid<T> for RwGrid<T> {
    fn step<D: Direction>(&self, pt: Point, dir: D) -> Point {
        let (x, y) = (pt.0 as i32, pt.1 as i32);
        let (xs, ys) = dir.step();
        let result = Point(modulo(x + xs, self.width), modulo(y + ys, self.height));
        result
    }

    fn get(&self, p: Point) -> T {
        let (region_index, index_in_region) = self.map_coordinates(p);
        let region = self.regions[region_index].read().unwrap();
        region[index_in_region]
    }
    fn get_neighborhood(&self, p: Point) -> Neighborhood<T> {
        let p = self.fix(p);
        Neighborhood {
            c: self.get(p),
            n: self.get(self.step(p, Compass::N)),
            ne: self.get(self.step(p, Compass::NE)),
            e: self.get(self.step(p, Compass::E)),
            se: self.get(self.step(p, Compass::SE)),
            s: self.get(self.step(p, Compass::S)),
            sw: self.get(self.step(p, Compass::SW)),
            w: self.get(self.step(p, Compass::W)),
            nw: self.get(self.step(p, Compass::NW)),
        }
    }
    fn set(&mut self, p: Point, value: T) {
        let (region_index, index_in_region) = self.map_coordinates(p);
        let mut region = self.regions[region_index].write().unwrap();
        region[index_in_region] = value;
    }
    fn rand(&self) -> Point {
        Point(roll::usize(self.width), roll::usize(self.height))
    }
    fn distance(&self, pt1: Point, pt2: Point) -> f64 {
        let x_dist = (pt1.0 as i32 - pt2.0 as i32) % (self.width / 2) as i32;
        let y_dist = (pt1.1 as i32 - pt2.1 as i32) % (self.height / 2) as i32;
        ((x_dist * x_dist + y_dist * y_dist) as f64).sqrt()
    }

    fn update<F>(&mut self, p: Point, update: F)
    where
        F: Fn(T) -> T,
    {
        let (region_index, index_in_region) = self.map_coordinates(p);
        let region = self.regions[region_index].get_mut().unwrap();
        let updated = update(region[index_in_region]);
        region[index_in_region] = updated;
    }
}

impl<T: Copy> ThreadedGrid<T> for RwGrid<T> {
    fn set_if<F>(&self, p: Point, f: F, value: T) -> bool
    where
        F: Fn(T) -> bool,
    {
        let (region_index, index_in_region) = self.map_coordinates(p);
        let mut region = self.regions[region_index].write().unwrap();
        let pre_existing = region[index_in_region];
        if f(pre_existing) {
            region[index_in_region] = value;
            true
        } else {
            false
        }
    }

    fn path<F>(&self, _p: Point, _step: &mut F) -> PathResult
    where
        F: FnMut(Neighborhood<T>) -> StepResult<T>,
    {
        todo!()
        // use self::Compass::*;
        // let mut p = p;
        // let mut dir: Option<Compass> = None;
        // loop {
        //     if let Some(dir) = dir {
        //         p = self.step(p, dir)
        //     }
        //     let mut locked = self.lock_hood(p);
        //     let neighborhood = Neighborhood {
        //         c: *locked.get(None),
        //         n: *locked.get(Some(N)),
        //         ne: *locked.get(Some(NE)),
        //         e: *locked.get(Some(E)),
        //         se: *locked.get(Some(SE)),
        //         s: *locked.get(Some(S)),
        //         sw: *locked.get(Some(SW)),
        //         w: *locked.get(Some(W)),
        //         nw: *locked.get(Some(NW)),
        //     };
        //     match step(neighborhood) {
        //         StepResult::Step(d) => dir = d,
        //         StepResult::Stick(t) => {
        //             println!("Sticking!");
        //             *locked.get(None) = t;
        //             return PathResult::Stuck(p);
        //         }
        //         StepResult::Die => return PathResult::Died(p),
        //         StepResult::Change(n) => todo!(),
        //     }
        // }
    }
}
impl<T: Copy> RwGrid<T> {
    // fn locked_hood(&self, p : Point) -> (Neighborhood<T>, HashSet<RwLockWriteGuard<T>>){

    // }

    pub fn new(width: usize, height: usize, default: T) -> Self {
        if width % REGIONS_PER_DIMENSION != 0 {
            panic!(
                "Width ({}) must be a multiple of {}",
                width, REGIONS_PER_DIMENSION
            )
        };
        if height % REGIONS_PER_DIMENSION != 0 {
            panic!(
                "Height ({}) must be a multiple of {}",
                height, REGIONS_PER_DIMENSION
            )
        };
        let region_size = height * width / TOTAL_REGIONS;
        let regions = [0; TOTAL_REGIONS].map(|_| RwLock::new(vec![default; region_size]));
        RwGrid {
            width,
            height,
            region_size,
            regions,
        }
    }
    pub(super) fn map_coordinates(&self, p: Point) -> (usize, usize) {
        let Point(x, y) = self.fix(p);
        let (region_width, region_height) = (
            self.width / REGIONS_PER_DIMENSION,
            self.height / REGIONS_PER_DIMENSION,
        );
        let (target_square_x, target_square_y) = (x / region_width, y / region_height);
        let target_square_i = target_square_y * REGIONS_PER_DIMENSION + target_square_x;
        let (x_in_square, y_in_square) = (x % region_width, y % region_width);
        let index_in_square = y_in_square * region_width + x_in_square;
        (target_square_i, index_in_square)
    }
    pub(super) fn fix(&self, p: Point) -> Point {
        let Point(x, y) = p;
        Point(modulo(x as i32, self.width), modulo(y as i32, self.height))
    }
    //fn lock_hood(&self, p: Point) -> LockedHood<T> {
    //     use self::Compass::*;
    //     let mut squares: Vec<Point> = Compass::all()
    //         .into_iter()
    //         .map(|d| self.step(p, d))
    //         .collect();
    //     squares.push(p);
    //     let mapped: HashMap<Point, (usize, usize)> = squares
    //         .into_iter()
    //         .map(|p| (p, self.map_coordinates(p)))
    //         .collect();
    //     let locks: Vec<(usize, RwLockWriteGuard<Vec<T>>)> = mapped
    //         .values()
    //         .map(|t| t.0)
    //         .sorted()
    //         .dedup()
    //         .map(|i| (i, self.regions[i].write().unwrap()))
    //         .collect();
    //     let mapped_again: HashMap<Point, (usize, usize)> = mapped
    //         .into_iter()
    //         .map(|(p, (big_i, little_i))| {
    //             let lock_index = locks.iter().position(|(i, _)| *i == big_i).unwrap();
    //             (p, (lock_index, little_i))
    //         })
    //         .collect();
    //     let just_the_locks: Vec<RwLockWriteGuard<Vec<T>>> =
    //         locks.into_iter().map(|(_, lock)| lock).collect();
    //     LockedHood {
    //         c: mapped_again[&p],
    //         n: mapped_again[&self.step(p, N)],
    //         ne: mapped_again[&self.step(p, NE)],
    //         e: mapped_again[&self.step(p, E)],
    //         se: mapped_again[&self.step(p, SE)],
    //         s: mapped_again[&self.step(p, S)],
    //         sw: mapped_again[&self.step(p, SW)],
    //         w: mapped_again[&self.step(p, W)],
    //         nw: mapped_again[&self.step(p, NW)],
    //         locks: just_the_locks,
    //     }
    // }
}
