
const REGIONS_PER_DIMENSION: usize = 8;
const TOTAL_REGIONS: usize = REGIONS_PER_DIMENSION * REGIONS_PER_DIMENSION;
pub const NUM_THREADS: usize = 1;

pub struct Ring<T> {
    pub n: T,
    pub ne: T,
    pub e: T,
    pub se: T,
    pub s: T,
    pub sw: T,
    pub w: T,
    pub nw: T,
}

impl<T: Copy> Ring<T> {
    pub fn count_matching<F>(&self, f: F) -> u8
    where
        F: Fn(T) -> bool,
    {
        let mut found = 0;
        let all = vec![
            self.n, self.ne, self.e, self.se, self.s, self.sw, self.w, self.nw,
        ];
        for ele in all.into_iter() {
            if f(ele) {
                found = found + 1
            }
        }
        found
    }
}


pub struct LockedHood<'a, T: Copy> {
    //Directions are: index of lock in locks, index of point in lock
    c: (usize, usize),
    n: (usize, usize),
    ne: (usize, usize),
    e: (usize, usize),
    se: (usize, usize),
    s: (usize, usize),
    sw: (usize, usize),
    w: (usize, usize),
    nw: (usize, usize),
    locks: Vec<RwLockWriteGuard<'a, Vec<T>>>,
}


impl<'a, T: Copy> LockedHood<'a, T> {
    fn center(&self) -> T {
        let (lock_index, index_in_lock) = self.c;
        self.locks[lock_index][index_in_lock]
    }
    fn get(&mut self, dir: Option<Compass>) -> &mut T {
        use self::Compass::*;
        let (lock_index, index_in_lock) = match dir {
            None => self.c,
            Some(dir) => match dir {
                N => self.n,
                NE => self.ne,
                E => self.e,
                SE => self.se,
                S => self.s,
                SW => self.sw,
                W => self.w,
                NW => self.nw,
            },
        };
        &mut self.locks[lock_index][index_in_lock]
    }
    fn set(&mut self, dir: Option<Compass>, value: T) {
        use self::Compass::*;
        let (lock_index, index_in_lock) = match dir {
            None => self.c,
            Some(dir) => match dir {
                N => self.n,
                NE => self.ne,
                E => self.e,
                SE => self.se,
                S => self.s,
                SW => self.sw,
                W => self.w,
                NW => self.nw,
            },
        };
        self.locks[lock_index][index_in_lock] = value;
    }
}