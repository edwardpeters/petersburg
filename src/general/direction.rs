#![allow(dead_code)]

use std::fmt::Display;


pub trait Direction: Clone + Copy + Display + From<usize> + PartialEq + Eq{
    fn step(&self) -> (i32, i32);
    fn left(&self) -> Self;
    fn right(&self) -> Self;
    fn reverse(&self) -> Self;
    fn rand() -> Self;
    fn all() -> Vec<Self>;
}

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Compass {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl From<usize> for Compass {
    fn from(i: usize) -> Compass {
        use general::direction::Compass::*;
        match i % 8 {
            0 => N,
            1 => NE,
            2 => E,
            3 => SE,
            4 => S,
            5 => SW,
            6 => W,
            7 => NW,
            _ => panic!("{} % 8 should have been 1-7, what happened?", i),
        }
    }
}

impl Display for Compass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Direction for Compass {
    fn step(&self) -> (i32, i32) {
        use general::direction::Compass::*;
        match self {
            N => (0, -1),
            NE => (1, -1),
            E => (1, 0),
            SE => (1, 1),
            S => (0, 1),
            SW => (-1, 1),
            W => (-1, 0),
            NW => (-1, -1),
        }
    }
    fn left(&self) -> Compass {
        Compass::from((*self as usize + 7) % 8)
    }
    fn right(&self) -> Compass {
        Compass::from((*self as usize + 1) % 8)
    }
    fn reverse(&self) -> Compass {
        Compass::from((*self as usize + 4) % 8)
    }
    fn rand() -> Compass {
        Compass::from(rand::random::<usize>() % 8)
    }
    fn all() -> Vec<Compass> {
        use general::direction::Compass::*;
        vec![N, NW, W, SW, S, SE, E, NE]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Cardinal {
    N,
    E,
    S,
    W,
}

impl Display for Cardinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<usize> for Cardinal {
    fn from(i: usize) -> Cardinal {
        use general::direction::Cardinal::*;
        match i % 8 {
            0 => N,
            1 => E,
            2 => S,
            3 => W,
            _ => panic!("{} % 4 should have been 1-3, what happened?", i),
        }
    }
}

impl Direction for Cardinal {
    fn step(&self) -> (i32, i32) {
        use general::direction::Cardinal::*;
        match self {
            N => (0, -1),
            E => (1, 0),
            S => (0, 1),
            W => (-1, 0),
        }
    }
    fn left(&self) -> Cardinal {
        Cardinal::from((*self as usize + 3) % 4)
    }
    fn right(&self) -> Cardinal {
        Cardinal::from((*self as usize + 1) % 4)
    }
    fn reverse(&self) -> Cardinal {
        Cardinal::from((*self as usize + 2) % 4)
    }
    fn all() -> Vec<Cardinal> {
        use general::direction::Cardinal::*;
        vec![N, W, S, E]
    }
    fn rand() -> Cardinal {
        Cardinal::from(rand::random::<usize>() % 4)
    }
}
