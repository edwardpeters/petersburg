use cairo::Context;
pub trait Petersburg: Sync + Send + 'static {
    fn run(&self);
    fn draw(&self, context: &Context);
}
