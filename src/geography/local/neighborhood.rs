use super::*;
use std::iter::FromIterator;
pub struct Neighborhood<T> {
    pub c: T,
    pub n: T,
    pub ne: T,
    pub e: T,
    pub se: T,
    pub s: T,
    pub sw: T,
    pub w: T,
    pub nw: T,
}

impl Neighborhood<(i32, i32)> {
    pub fn local() -> Neighborhood<(i32, i32)> {
        Neighborhood {
            c: (0, 0),
            n: (0, -1),
            ne: (1, -1),
            e: (1, 0),
            se: (1, 1),
            s: (0, 1),
            sw: (-1, 1),
            w: (-1, 0),
            nw: (-1, -1),
        }
    }
}

impl<T: Copy> Neighborhood<T> {
    pub fn from_dir(&self, dir: Compass) -> T {
        use super::Compass::*;
        match dir {
            N => self.n,
            NE => self.ne,
            E => self.e,
            SE => self.se,
            S => self.s,
            SW => self.sw,
            W => self.w,
            NW => self.nw,
        }
    }
    pub fn map<F, A>(&self, f: F) -> Neighborhood<A>
    where
        F: Fn(T) -> A,
    {
        Neighborhood {
            c: f(self.c),
            n: f(self.n),
            ne: f(self.ne),
            e: f(self.e),
            se: f(self.se),
            s: f(self.s),
            sw: f(self.sw),
            w: f(self.w),
            nw: f(self.nw),
        }
    }
}

impl<T> IntoIterator for Neighborhood<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            self.c, self.n, self.ne, self.e, self.se, self.s, self.sw, self.w, self.nw,
        ]
        .into_iter()
    }
}
impl<T> FromIterator<T> for Neighborhood<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut i = iter.into_iter();
        Neighborhood {
            c: i.next().unwrap(),
            n: i.next().unwrap(),
            ne: i.next().unwrap(),
            e: i.next().unwrap(),
            se: i.next().unwrap(),
            s: i.next().unwrap(),
            sw: i.next().unwrap(),
            w: i.next().unwrap(),
            nw: i.next().unwrap(),
        }
    }
}
