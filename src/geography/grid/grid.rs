use super::super::*;
use super::*;

pub mod types {
    pub use super::{Grid, PathResult, StepResult, ThreadedGrid};
}

pub enum StepResult<T> {
    Step(Option<Compass>),
    Change(Neighborhood<T>),
    Stick(T),
    Die,
}
pub enum PathResult {
    Stuck(Point),
    Died(Point),
}
pub trait Grid<T: Copy> {
    fn get(&self, p: Point) -> T;
    fn get_neighborhood(&self, p: Point) -> Neighborhood<T>;
    fn set(&mut self, p: Point, value: T);
    fn update<F>(&mut self, p: Point, update: F)
    where
        F: Fn(T) -> T;
    fn step<D: Direction>(&self, pt: Point, dir: D) -> Point;
    fn rand(&self) -> Point;
    fn distance(&self, p1: Point, p2: Point) -> f64;
}

pub trait ThreadedGrid<T: Copy>: Grid<T> {
    fn set_if<F>(&self, p: Point, f: F, value: T) -> bool
    where
        F: Fn(T) -> bool;
    fn path<F>(&self, p: Point, step: &mut F) -> PathResult
    where
        F: FnMut(Neighborhood<T>) -> StepResult<T>;
}
