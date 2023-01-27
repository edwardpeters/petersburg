use super::*;
use std::fmt::{Display, Formatter, Result};
impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({},{})", self.0, self.1)
    }
}
