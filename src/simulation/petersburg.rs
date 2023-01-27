extern crate cairo;
use self::cairo::Context;
pub trait Petersburg {
    type Config;
    fn new(c: Self::Config) -> Self;
    fn run(&self);
    fn draw(&self, context: &Context);
}
