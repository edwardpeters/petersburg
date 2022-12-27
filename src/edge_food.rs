#![allow(unused_variables, dead_code, unreachable_code)]

use general::direction::{Compass, Direction};
use std::sync::{Arc, Mutex};
use std::thread;
use types_constants::*;
#[allow(unused_imports)]
use general::utils::*;
use general::draw_utils::*;

fn rand_direction() -> Compass {
    Compass::from(rand::random::<usize>() % 8)
}

//fn seek_home(x : &mut usize, y : &mut usize, )

fn seek(
    start: (usize, usize),
    grid: &mut ScentGrid,
    clock: &mut usize,
    draw_mut: Arc<Mutex<DrawGrid>>,
) -> (usize, usize) {
    let mut direction = rand_direction();
    let (mut x, mut y) = start;
    let mut homesickness = SIZE * SIZE;
    let color = (
        rand::random::<f64>(),
        rand::random::<f64>(),
        rand::random::<f64>(),
    );
    'seek_food: loop {
        // if !grid[x][y].stuck {
        //     let mut draw_grid = draw_mut.lock().unwrap();
        //     draw_grid[x][y] = color;
        //     drop(draw_grid)
        // };
        homesickness = homesickness - 1;
        *clock = *clock + 1;
        if x == 0 || y == 0 || x == SIZE - 1 || y == SIZE - 1 {
            break 'seek_food;
        }
        let ScentSquare { food, home, stuck } = grid[x][y];
        //let home_scent = if (homesickness as usize) > *clock  {0} else {*clock - (homesickness as usize)};
        let home_scent = std::cmp::max(homesickness as usize, home);
        grid[x][y] = ScentSquare {
            food: food,
            home: home_scent,
            stuck: stuck,
        };
        let drift = rand::random::<usize>() % 8;
        if drift == 0 {
            direction = direction.left()
        } else if drift == 7 {
            direction = direction.right()
        };
        let (xs, ys) = direction.step();
        (x, y) = ((x as i32 + xs) as usize, (y as i32 + ys) as usize)
    }
    let legal = |(xp, yp): (i32, i32)| xp >= 0 && yp >= 0 && xp < SIZE as i32 && yp < SIZE as i32;
    let legal_u = |(xp, yp): (usize, usize)| legal((xp as i32, yp as i32));
    let step = |p: (usize, usize), dir: Compass| {
        let (xp, yp) = p;
        let (xs, ys) = dir.step();
        ((xp as i32 + xs) as usize, (yp as i32 + ys) as usize)
    };
    'seek_home: loop {
        // if !grid[x][y].stuck {
        //     let mut draw_grid = draw_mut.lock().unwrap();
        //     draw_grid[x][y] = color;
        //     drop(draw_grid)
        // };
        *clock = *clock + 1;
        for i in 0..3 {
            for j in 0..3 {
                let (xp, yp) = (x as i32 + i - 1, y as i32 + j - 1);
                if legal((xp, yp)) && grid[xp as usize][yp as usize].stuck {
                    break 'seek_home;
                }
            }
        }

        let spos = step((x, y), direction);
        let rpos = step((x, y), direction.right());
        let lpos = step((x, y), direction.left());
        if legal_u(spos) {
            let (mut sweight, mut lweight, mut rweight) = (3, 1, 1);
            let ((sx, sy), (lx, ly), (rx, ry)) = (spos, lpos, rpos);

            let sh = grid[sx][sx].home;
            let rh = if !legal_u((rx, ry)) {
                rweight = 0;
                0
            } else {
                grid[rx][ry].home
            };
            let lh = if !legal_u((lx, ly)) {
                lweight = 0;
                0
            } else {
                grid[lx][ly].home
            };

            if sh > lh {
                sweight = sweight + 2
            } else if lh > sh {
                lweight = lweight + 2
            }
            if rh > lh {
                rweight = rweight + 2
            } else if lh > rh {
                lweight = lweight + 2
            }
            if sh > rh {
                sweight = sweight + 2
            } else if rh > sh {
                rweight = rweight + 2
            }
            let roll = rand::random::<usize>() % (lweight + rweight + sweight);
            if roll < sweight {
                (x, y) = (sx, sy);
            } else if roll < sweight + rweight {
                direction = direction.right();
                (x, y) = (rx, ry);
            } else {
                direction = direction.left();
                (x, y) = (lx, ly)
            }
        } else {
            direction = direction.reverse();
        }
    }
    (x, y)
}

//fn steveburg(tx : Sender<Message>){
pub fn steveburg(draw_grid_mut: Arc<Mutex<DrawGrid>>) {
    let empty = ScentSquare {
        food: 0,
        home: 0,
        stuck: false,
    };
    let mut clock: usize = 3200;
    let mut grid: ScentGrid = vec![vec![empty; SIZE]; SIZE];
    let mut successes = 0;
    let (mut x, mut y) = (SIZE / 2, SIZE / 2);
    grid[SIZE / 2][SIZE / 2].stuck = true;

    loop {
        let start_time = clock;
        let draw_mut_seek = Arc::clone(&draw_grid_mut);
        (x, y) = seek((x, y), &mut grid, &mut clock, draw_mut_seek);
        successes = successes + 1;
        grid[x][y].stuck = true;
        let mut draw_grid = draw_grid_mut.lock().unwrap();
        draw_grid[x][y] = WHITE;
        drop(draw_grid);
        //tx.send(new_point).unwrap();
        println!(
            "A particle made its way home in {} steps, sticking at ({},{})",
            clock - start_time,
            x,
            y
        );
        if clock - start_time < 3 {
            break;
        };
    }
    println!("All done with a total clock of {}", clock);

    loop {
        thread::sleep(REFRESH);
    }
    let mut max = 0;
    let mut min = usize::MAX;
    for i in 0..SIZE {
        for j in 0..SIZE {
            if grid[i][j].home > max {
                max = grid[i][j].home
            }
            if grid[i][j].home != 0 && grid[i][j].home < min {
                min = grid[i][j].home
            }
        }
    }
    let range = max - min;
    println!("Heat range is {}", range);
    {
        let mut draw_grid = draw_grid_mut.lock().unwrap();
        for i in 0..SIZE {
            for j in 0..SIZE {
                if grid[i][j].home != 0 {
                    draw_grid[i][j] = heat_to_color(grid[i][j].home - min, range);
                }
                // tx.send((i, j, heat_to_color(grid[i][j].home, max))).unwrap();
            }
        }
        drop(draw_grid);
    }
    println!("ALl dropped?");
}
