extern crate wlroots;

use wlroots::{
    compositor,
    utils::log::{init_logging, WLR_DEBUG}
};

fn main() {
    init_logging(WLR_DEBUG, None);
    compositor::Builder::new().build_auto(()).run()
}
