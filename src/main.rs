// extern crate cairo;
// extern crate colored;
// extern crate gdk;
// extern crate glib;
// extern crate gtk;
// extern crate itertools;
// extern crate once_cell;
// extern crate priority_queue;
// extern crate rand;

pub mod geography;
//use geography::local::*;

// use gtk::prelude::*;
// use gtk::{Application, ApplicationWindow, DrawingArea};
// #[allow(unused_imports)]
// use mazeburg::Mazeburg;
// #[allow(unused_imports)]
// use petersburg::Petersburg;
// #[allow(unused_imports)]
// use simple::*;
// use std::cell::RefCell;
// use std::sync::Arc;
// use std::thread;
// use types_constants::*;
// use foodburg::foodburg::*;
// use geography::draw_utils;

// thread_local!(
//     static GLOBAL: RefCell<Option<ApplicationWindow>> = RefCell::new(None);
// );

// fn check_update_display() {
//     GLOBAL.with(|global| {
//         if let Some(win) = &*global.borrow() {
//             win.queue_draw();
//         }
//     })
// }

fn main() {
    println!("Project undergoing surgery, check back soon")
}

// fn main() {
//     let config = foodburg::foodburg::Config{
//         size : 1024,
//         maze_squares : 32,
//         num_species : 10,
//         openness: 0.1,
//         wrapped : true
//     };
//     use geography::burg::Burg;
//     let simulation = Foodburg::new(config);
//     //let simulation = Mazeburg::init(2048, 16, 12, false, 0.1, false);
//     let simulation_a = Arc::new(simulation);
//     let simulation_draw = Arc::clone(&simulation_a);
//     thread::spawn(move || {
//         simulation_a.run();
//     });

//     let app = Application::builder()
//         .application_id("org.example.HelloWorld")
//         .build();
//     gtk::init().expect("GTK init failed");
//     let draw_area = DrawingArea::new();

//     let _id = draw_area.connect_draw(move |_unused, context| {
//         context.set_source_rgb(0.0, 0.0, 0.0);
//         context.paint().expect("Painting failed");
//         simulation_draw.draw(context);
//         Inhibit(false)
//     });

//     app.connect_activate(move |app| {
//         let win = ApplicationWindow::builder()
//             .application(app)
//             .default_width(320)
//             .default_height(200)
//             .title("steveburg")
//             .build();
//         win.add(&draw_area);
//         win.show_all();

//         GLOBAL.with(|global| {
//             *global.borrow_mut() = Some(win);
//         });

//         glib::timeout_add(REFRESH, move || {
//             check_update_display();
//             Continue(true)
//         });
//     });

//     // let berg_grid_mut = Arc::clone(&grid_mut);
//     // thread::spawn(move || {
//     //     edge_food::steveburg(berg_grid_mut);
//     // });
//     app.run();
// }
