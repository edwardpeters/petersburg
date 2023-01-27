extern crate gtk;

use self::gtk::prelude::*;
use self::gtk::{Application, ApplicationWindow, DrawingArea};
use burgs::*;
use geography::*;
use simulation::Petersburg;
use std::sync::Arc;
use std::thread;
use std::time;

fn check_update_display(win: &ApplicationWindow) {
    win.queue_draw();
}

pub fn run() {
    let p: Point = Point(0, 2);

    // let config = MazeburgConfig {
    //     threads: 16,
    //     size: 2048,
    //     squares: 16,
    //     species_count: 6,
    //     wrapped: false,
    //     openness: 0.02,
    //     show_lines: false,
    // };
    // let simulation = Mazeburg::new(config);

    let config = FoodConfig {
        threads: 16,
        size: 1024,
        maze_squares: 16,
        num_species: 8,
        openness: 0.05,
        wrapped: false,
    };
    let simulation = Foodburg::new(config);

    // let config = ();
    // let simulation = Simple::new(config);

    // let config = EdgeFoodConfig { size: 512 };
    // let simulation = EdgeFoodBurg::new(config);

    let simulation_run = Arc::new(simulation);
    let simulation_draw = Arc::clone(&simulation_run);

    thread::spawn(move || {
        simulation_run.run();
    });
    let app = Application::builder()
        .application_id("org.petersburg.Petersburg")
        .build();
    gtk::init().expect("GTK init failed");
    let draw_area = DrawingArea::new();

    let _id = draw_area.connect_draw(move |_unused, context| {
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.paint().expect("Painting failed");
        simulation_draw.draw(context);
        Inhibit(false)
    });

    app.connect_activate(move |app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("steveburg")
            .build();
        win.add(&draw_area);
        win.show_all();

        glib::timeout_add_local(time::Duration::from_millis(500), move || {
            check_update_display(&win);
            Continue(true)
        });
    });
    app.run();/*  */
}
