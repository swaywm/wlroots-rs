extern crate log;
extern crate wlroots;

use log::LevelFilter;

fn main() {
    wlroots::utils::log::Logger::init(LevelFilter::Debug);
    wlroots::compositor::Builder::new().build_auto(()).run()
}
