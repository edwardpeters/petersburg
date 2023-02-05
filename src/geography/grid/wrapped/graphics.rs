#[allow(unused_imports)]
use super::{
    super::{super::*, *},
    *,
};
use cairo::*;

impl<T: Copy + Colored> Draw for WrappedGrid<T> {
    fn draw(&self, context: &Context) {
        //TODO! review this
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.paint().expect("Painting failed");
        let square = color::get_scale(self.height);
        for i in 0..self.height {
            for j in 0..self.width {
                let color = self.grid[i][j].color();
                if color != color::BLACK {
                    let Color { r, g, b } = color;
                    context.set_source_rgb(r, g, b);
                    let (scaled_x, scaled_y) =
                        (color::scale(i, self.width), color::scale(j, self.height));

                    //context.rectangle(i as f64 * scale, j as f64 * scale, square, square);
                    context.rectangle(scaled_x, scaled_y, square, square);
                    context.fill().unwrap();
                    // context.move_to(i as f64 * scale, j as f64 * scale);
                    // context.in_fill(scale, scale).unwrap();
                    context.stroke().unwrap();
                }
            }
        }
    }
}
