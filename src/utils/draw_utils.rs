extern crate cairo;
use self::cairo::Context;
use super::*;
use geography::*;

pub mod types {
    pub use super::Draw;
}

pub trait Draw {
    fn draw(&self, context: &Context);
}

impl ColorSettable for Context {
    #[inline(always)]
    fn set_color(&self, c: Color) {
        let Color { r, g, b } = c;
        self.set_source_rgb(r, g, b)
    }
}

pub fn path_helper(context: &Context, size: usize, path_color: Color, path: &Vec<Point>) {
    let scale = color::get_scale(size);
    let square = color::get_scale(size);
    context.set_color(path_color);
    for p in path {
        let Point(i, j) = *p;
        context.rectangle(i as f64 * scale, j as f64 * scale, square, square);
    }
    context.stroke().unwrap();
}
