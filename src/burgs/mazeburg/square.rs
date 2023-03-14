#[allow(unused_imports)]
use super::{super::*, *};
pub const EMPTY: Square = Square {
    species: usize::MAX,
};

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub struct Square {
    pub species: usize,
}
impl Colored for Square {
    fn color(&self) -> Color {
        if *self == EMPTY {
            color::BLACK
        } else {
            color::COLORS[self.species]
        }
    }
}
