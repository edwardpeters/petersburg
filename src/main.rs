#![allow(unused_labels)]

pub mod burgs;
pub mod constants;
pub mod genes;
pub mod geography;
pub mod maze;
mod run;
pub mod simulation;
pub mod utils;

fn main() {
    run::run();
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
