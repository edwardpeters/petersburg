mod mazeburg;
pub(self) mod square;
pub use self::mazeburg::types::*;
pub(self) use self::square::Square;
mod args;
pub use self::args::MazeburgArgs;