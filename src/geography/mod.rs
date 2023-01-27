extern crate crossbeam;

//Organization reasoning: I like relatively flat structures, following the idiom that functions should be addressed by parent crate, but structs/enums can be addressed directly

pub mod local;
pub use self::local::*;
pub mod grid;
pub use self::grid::*;
pub mod displays;
