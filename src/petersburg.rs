use cairo::Context;
use std::sync::Arc;

pub trait Petersburg {
    fn run(self: Arc<Self>);
    fn draw(&self, context: &Context);
}
