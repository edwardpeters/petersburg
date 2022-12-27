use general::direction::Direction;
use general::utils::*;
use general::grid::Point;



pub struct Grid<T: Copy> {
    pub height: usize,
    pub width: usize,
    pub grid: Vec<Vec<T>>,
}

impl<T: Copy> Grid<T> {
    pub fn new(h: usize, w: usize, default: T) -> Grid<T> {
        Grid::<T> {
            width: w,
            height: h,
            grid: vec![vec!(default; h); w],
        }
    }
    #[inline(always)]
    pub fn at(&self, p: Point) -> T {
        let Point(x, y) = self.fix(p);
        self.grid[x][y]
    }
    #[inline(always)]
    pub fn set(&mut self, p: Point, value: T) {
        let Point(x, y) = self.fix(p);
        self.grid[x][y] = value;
    }
    #[inline(always)]
    pub fn fix(&self, p: Point) -> Point {
        let Point(x, y) = p;
        Point(modulo(x as i32, self.width), modulo(y as i32, self.width))
    }
    #[inline(always)]
    pub fn step<D: Direction>(&self, pt: Point, dir: D) -> Point {
        let (x, y) = (pt.0 as i32, pt.1 as i32);
        let (xs, ys) = dir.step();
        let result = Point(modulo(x + xs, self.width), modulo(y + ys, self.height));
        // if x + xs < 0 || y + ys < 0 {
        //     println!("{pt} stepped in {dir} for result {result}")
        // };
        result
    }
    #[inline(always)]
    pub fn distance(&self, pt1: Point, pt2: Point) -> f64 {
        let x_dist = (pt1.0 as i32 - pt2.0 as i32) % (self.width / 2) as i32;
        let y_dist = (pt1.1 as i32 - pt2.1 as i32) % (self.height / 2) as i32;
        ((x_dist * x_dist + y_dist * y_dist) as f64).sqrt()
    }
    pub fn rand_point(&self) -> Point {
        Point(roll::usize(self.width), roll::usize(self.height))
    }
}
