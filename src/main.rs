mod app;
use crate::app::application::Application;

fn main() {
    let mut app = Application::new(1280, 720, "Bezier Curve Editor").unwrap();
    app.run()
}