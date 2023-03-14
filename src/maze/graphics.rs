use super::*;
use crate::utils::*;
use cairo::Context;

impl Draw for Maze {
    fn draw(&self, context: &Context) {
        let Self {
            num_squares: size,
            vbars,
            hbars,
            scale,
            ..
        } = self;
        let draw_scale = color::get_scale(scale * size);
        context.set_color(color::WHITE);
        for i in 0..*size {
            for j in 0..*size {
                if hbars[i][j] {
                    let startx = (i * scale) as f64 * draw_scale;
                    let y = (j * scale) as f64 * draw_scale;
                    context.rectangle(startx, y, (1 + *scale) as f64 * draw_scale, draw_scale);
                    context.fill().unwrap();
                    if j == 0 {
                        let y_wrap = (*size * scale) as f64 * draw_scale;
                        context.rectangle(
                            startx,
                            y_wrap,
                            (1 + *scale) as f64 * draw_scale,
                            draw_scale,
                        );
                        context.fill().unwrap();
                    }
                    context.stroke().unwrap();
                }
                if vbars[i][j] {
                    let starty = (j * scale) as f64 * draw_scale;
                    let x = (i * scale) as f64 * draw_scale;
                    context.rectangle(x, starty, draw_scale, (1 + *scale) as f64 * draw_scale);
                    context.fill().unwrap();
                    if i == 0 {
                        let x_wrap = (*size * scale) as f64 * draw_scale;
                        context.rectangle(
                            x_wrap,
                            starty,
                            draw_scale,
                            (1 + *scale) as f64 * draw_scale,
                        );
                        context.fill().unwrap();
                    }
                }
            }
        }
    }
}

// pub fn alt_draw(&self, context: &Context) {
//     if roll::bool() {
//         self.draw(context);
//     } else {
//         let draw_scale = get_scale(self.scale * self.size);
//         context.set_color(WHITE);
//         for i in 0..SIZE {
//             for j in 0..SIZE {
//                 if self.is_wall(Point(i, j)) {
//                     context.rectangle(
//                         i as f64 * draw_scale,
//                         j as f64 * draw_scale,
//                         draw_scale,
//                         draw_scale,
//                     )
//                 }
//             }
//         }
//         context.stroke().unwrap();
//     }
// }
