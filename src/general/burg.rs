use cairo::Context;
pub trait Burg{
    type Config;
    fn new(c : Self::Config) -> Self;
    fn run(&self);
    fn draw(&self, context : &Context);
}