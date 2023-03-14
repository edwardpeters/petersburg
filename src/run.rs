use crate::burgs::*;
use crate::simulation::*;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};
use std::sync::Arc;
use std::thread;
use std::time;

use clap::*;

#[derive(Parser, Debug, Copy, Clone)]
struct Cli {
    #[command(subcommand)]
    simulation: Simulation,
}

#[derive(Subcommand, Debug, Copy, Clone)]
enum Simulation {
    Unburg(UnburgArgs),
    Foodburg(FoodburgArgs),
    Mazeburg(MazeburgArgs),
    Scentburg(ScentburgArgs),
    Simpleburg(SimpleArgs),
}

pub fn run() {
    let cli = Cli::parse();

    match cli.simulation {
        Simulation::Unburg(args) => {
            let sim = Unburg::new(args);
            run_helper(sim);
        }
        Simulation::Foodburg(args) => {
            let sim = Foodburg::new(args);
            run_helper(sim)
        }
        Simulation::Simpleburg(args) => {
            let sim = Simpleburg::new(args);
            run_helper(sim)
        }
        Simulation::Mazeburg(args) => {
            let sim = Mazeburg::new(args);
            run_helper(sim)
        }
        Simulation::Scentburg(args) => {
            let sim = Scentburg::new(args);
            run_helper(sim)
        }
    }
}
fn run_helper<T: Petersburg>(simulation: T) {
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
    let empty: Vec<String> = vec![];
    app.run_with_args(&empty);
}

fn check_update_display(win: &ApplicationWindow) {
    win.queue_draw();
}
