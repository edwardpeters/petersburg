use super::regional_grid::*;
use crate::utils::{color, color::types::*, draw_utils::Draw};
use cairo::Context;

impl<T: Colored + Copy> Draw for RegionalGrid<T> {
    fn draw(&self, context: &Context) {
        let (region_width, region_height) = (
            self.width / REGIONS_PER_DIMENSION,
            self.height / REGIONS_PER_DIMENSION,
        );

        for i in 0..TOTAL_REGIONS {
            let (corner_x, corner_y) = (
                (i % REGIONS_PER_DIMENSION) * region_width,
                (i / REGIONS_PER_DIMENSION) * region_height,
            );
            let region = self.regions[i].read().unwrap();
            for j in 0..self.region_size {
                let (x, y) = (corner_x + j % region_width, corner_y + j / region_width);
                let (scaled_x, scaled_y) =
                    (color::scale(x, self.width), color::scale(y, self.height));
                let color = region[j].color();
                if color != color::BLACK {
                    context.set_color(color);
                    context.rectangle(
                        scaled_x,
                        scaled_y,
                        color::get_scale(self.width),
                        color::get_scale(self.height),
                    );
                    context.fill().unwrap();
                }
            }
        }

        //     for i in 0..self.width {
        //         for j in 0..self.height {
        //             let (_, i_region) = self.map_coordinates(Point(i, j));
        //             let (x, y) = (color::scale(i, self.width), color::scale(j, self.height));
        //             context.move_to(x, y);
        //             let red = 0.0; //(region_i % 3) as f64 / (3.0);
        //             let green = (i_region % (self.width / REGIONS_PER_DIMENSION)) as f64
        //                 / self.region_size as f64;
        //             let blue = 0.0;
        //             context.set_source_rgb(red, green, blue);
        //             context.rectangle(
        //                 x,
        //                 y,
        //                 color::get_scale(self.width),
        //                 color::get_scale(self.height),
        //             );
        //             context.fill().unwrap();
        //         }
        //     }
    }
}
//Referenced nowhere so GFY
// pub fn draw_debug(self: RwGrid, context: &Context) {
//     for i in 0..self.width {
//         for j in 0..self.height {
//             let p = Point(i, j);
//             let color = self.get(p).color();
//             if color != BLACK {
//                 context.set_color(color);
//                 context.rectangle(
//                    color::scale(i, self.width),
//                    color::scale(j, self.height),
//                    color::get_scale(self.width),
//                    color::get_scale(self.height),
//                 );
//                 context.fill().unwrap();
//             }
//         }
//     }
// }
