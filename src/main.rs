#[allow(unused_imports)]
use crate::app::App;

mod app;
pub mod commands;

fn main() {
    App::new().run().unwrap();
}
