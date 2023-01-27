#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point(pub usize, pub usize);

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

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}
