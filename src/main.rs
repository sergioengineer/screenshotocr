mod application;
mod features;
mod types;
use gtk::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    let application = gtk::Application::new(
        Some("screenshotocr.sergioengineer.githubio"),
        Default::default(),
    );

    application.connect_activate(application::build_application);

    application.run();
}
