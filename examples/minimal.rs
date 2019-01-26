extern crate log;
extern crate wlroots;

use log::LevelFilter;

fn main() {
    wlroots::utils::log::Logger::init(LevelFilter::Debug, None);
    wlroots::compositor::Builder::new().build_auto(()).run()
}
