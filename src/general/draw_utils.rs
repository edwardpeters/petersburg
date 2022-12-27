use cairo::Context;
use std::collections::LinkedList as List;
use colored::Colorize;
use std::sync::{Arc, Mutex};
use types_constants::*;
#[allow(unused_imports)]
use general::utils::*;
use std::fmt;

use general::grid::*;

#[derive(Copy, Clone)]
pub struct Color{
    pub r : f64,
    pub g : f64,
    pub b : f64
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = (
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        );
        let colored = format!("Color(r:{}, g:{}, b:{})", self.r, self.g, self.b).truecolor(r, g, b);
        write!(f, "{}", colored)
    }
}


pub static BLACK :Color = Color{r : 0.0, g : 0.0, b : 0.0};
pub static WHITE :Color = Color{r : 1.0, g : 1.0, b : 1.0};
pub static GREEN : Color = Color{r : 0.0, g : 1.0, b : 0.0};
pub static RED : Color = Color{r: 1.0, g : 0.0, b : 0.0};
pub static BLUE : Color = Color{r : 0.0, g : 0.0, b : 1.0};
pub static PURPLE : Color = Color{r : 0.5, g : 0.0, b : 0.8};
pub static TEAL : Color = Color{r : 0.4, g : 1.0, b : 1.0};
pub static LIME : Color = Color{r : 0.6, g : 1.0, b : 0.2};
pub static BROWN : Color = Color{r : 0.5, g : 0.2, b : 0.0};
pub static ORANGE : Color = Color{r : 0.9, g : 0.3, b : 0.0};
pub static YELLOW : Color = Color{r : 1.0, g : 1.0, b : 0.0};
pub static PINK : Color = Color{r: 1.0, g : 0.4, b : 0.8};
pub static LICHEN : Color = Color{r: 0.6, g : 0.8, b : 0.5};
pub static MAROON : Color = Color{r: 0.4, g : 0.0, b : 0.0};

pub static COLORS : [Color; 12] = [
    RED, BLUE, GREEN, PURPLE, TEAL, LIME, BROWN, ORANGE, YELLOW, PINK, LICHEN, MAROON
];



pub trait Colored{
    fn color(&self) -> Color;
}



pub trait ColorSettable{
    fn set_color(&self, c : Color);
}

impl std::hash::Hash for Color{
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        ((self.r * 255.0) as u8,(self.b * 255.0) as u8,(self.g * 255.0) as u8).hash(state)
    }
}

impl PartialEq for Color{
    fn eq(&self, other : &Self) -> bool{
        ((self.r * 255.0) as u8,(self.b * 255.0) as u8,(self.g * 255.0) as u8) ==((other.r * 255.0) as u8,(other.b * 255.0) as u8,(other.g * 255.0) as u8)
    }
}

impl Eq for Color{}

impl ColorSettable for Context{
    #[inline(always)]
    fn set_color(&self, c : Color){
        let Color{r, g, b} = c;
        self.set_source_rgb(r, g, b)
    }
}

pub fn heat_to_color(heat: usize, scale: usize) -> Color {
    let (r, g, b) = if heat < scale / 4 {
        (0.0, heat as f64 / (scale as f64 / 4.0), 0.0)
    } else if heat < scale / 2 {
        let intensity = (heat - scale / 4) as f64 / (scale as f64 / 4.0);
        (0.0, 1.0 - intensity, intensity)
    } else if heat < (scale * 3) / 4 {
        let intensity = (heat - scale / 2) as f64 / (scale as f64 / 4.0);
        (intensity, 0.0, 1.0 - intensity)
    } else {
        let intensity = (heat - (3 * scale) / 4) as f64 / (scale as f64 / 4.0);
        (1.0, intensity, intensity)
    };
    Color{r, g, b}
}


pub fn get_scale(size: usize) -> f64 {
    900 as f64 / size as f64
}
pub fn scale(length : usize, size : usize) -> f64{
    let s = get_scale(size);
    length as f64 * s
}


pub fn random_color() -> Color {
    let (r, g, b) = (
        rand::random::<f64>(),
        rand::random::<f64>(),
        rand::random::<f64>(),
    );
    Color{r, g, b}
}

pub fn grid_helper(context: &Context, draw_grid_mut: &Arc<Mutex<DrawGrid>>) {
    let grid = draw_grid_mut.lock().unwrap();
    context.set_source_rgb(0.0, 0.0, 0.0);
    context.paint().expect("Painting failed");
    let scale = get_scale(SIZE);
    let square = get_scale(SIZE);
    for i in 0..SIZE {
        for j in 0..SIZE {
            let color = grid[i][j];
            if color != BLACK {
                let Color{r, g, b} = color;
                context.set_source_rgb(r, g, b);

                context.rectangle(i as f64 * scale, j as f64 * scale, square, square);
                context.fill().unwrap();
                // context.move_to(i as f64 * scale, j as f64 * scale);
                // context.in_fill(scale, scale).unwrap();
                context.stroke().unwrap();
            }
        }
    }
    drop(grid);
}
pub fn draw_grid(context: &Context, size: usize, grid: &DrawGrid) {
    let scale = get_scale(size);
    for x in 0..size {
        for y in 0..size {
            let color = grid[x][y];
            if color != BLACK {
                context.set_color(color);
                context.rectangle(x as f64 * scale, y as f64 * scale, scale, scale);
                context.fill().unwrap();
            }
        }
    }
}
pub fn draw_path(context: &Context, size: usize, color: Color, path: &Vec<Point>) {
    context.set_color(color);
    let scale = get_scale(size);
    for p in path {
        let Point(x, y) = *p;
        context.rectangle(x as f64 * scale, y as f64 * scale, scale, scale);
        context.fill().unwrap();
    }
}
pub fn path_helper(context: &Context, draw_path_mut: &Arc<Mutex<(Color, List<Point>)>>) {
    let pair = draw_path_mut.lock().unwrap();
    let path = &pair.1;
    let path_color = pair.0;
    let scale = get_scale(SIZE);
    let square = get_scale(SIZE);
    context.set_color(path_color);
    path.iter().for_each(|p| {
        let Point(i, j) = *p;
        context.rectangle(i as f64 * scale, j as f64 * scale, square, square);
    });
    context.stroke().unwrap();
}
