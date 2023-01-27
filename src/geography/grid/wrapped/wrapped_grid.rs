use super::super::super::*;
use super::super::*;
use crate::utils::*;

pub struct WrappedGrid<T: Copy> {
    pub(super) height: usize,
    pub(super) width: usize,
    pub(super) grid: Vec<Vec<T>>,
}

impl<T: Copy> Grid<T> for WrappedGrid<T> {
    #[inline(always)]
    fn get(&self, p: Point) -> T {
        let Point(x, y) = self.fix(p);
        self.grid[x][y]
    }
    fn get_neighborhood(&self, p: Point) -> Neighborhood<T> {
        Neighborhood::local().map(|a| {
            self.get(Point(
                modulo(p.0 as i32 + a.0, self.width),
                modulo(p.1 as i32 + a.1, self.height),
            ))
        })
    }
    #[inline(always)]
    fn set(&mut self, p: Point, value: T) {
        let Point(x, y) = self.fix(p);
        self.grid[x][y] = value;
    }
    #[inline(always)]
    fn step<D: Direction>(&self, pt: Point, dir: D) -> Point {
        let (x, y) = (pt.0 as i32, pt.1 as i32);
        let (xs, ys) = dir.step();
        let result = Point(modulo(x + xs, self.width), modulo(y + ys, self.height));
        // if x + xs < 0 || y + ys < 0 {
        //     println!("{pt} stepped in {dir} for result {result}")
        // };
        result
    }
    #[inline(always)]
    fn distance(&self, pt1: Point, pt2: Point) -> f64 {
        let x_dist = (pt1.0 as i32 - pt2.0 as i32) % (self.width / 2) as i32;
        let y_dist = (pt1.1 as i32 - pt2.1 as i32) % (self.height / 2) as i32;
        ((x_dist * x_dist + y_dist * y_dist) as f64).sqrt()
    }
    fn rand(&self) -> Point {
        Point(roll::usize(self.width), roll::usize(self.height))
    }

    fn update<F>(&mut self, p: Point, update: F)
    where
        F: Fn(T) -> T,
    {
        let Point(x, y) = self.fix(p);
        let updated = update(self.grid[x][y]);
        self.grid[x][y] = updated;
    }
}

impl<T: Copy> WrappedGrid<T> {
    pub fn new(h: usize, w: usize, default: T) -> Self {
        WrappedGrid::<T> {
            width: w,
            height: h,
            grid: vec![vec!(default; h); w],
        }
    }
    #[inline(always)]
    fn fix(&self, p: Point) -> Point {
        let Point(x, y) = p;
        Point(modulo(x as i32, self.width), modulo(y as i32, self.width))
    }
}
