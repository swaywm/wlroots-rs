extern crate wlroots;

fn main() {
    wlroots::utils::log::init_logging(wlroots::utils::log::WLR_DEBUG, None);
    wlroots::compositor::Builder::new().build_auto(()).run()
}
