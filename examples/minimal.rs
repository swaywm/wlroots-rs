extern crate wlroots;

fn main() {
    wlroots::utils::init_logging(wlroots::utils::L_DEBUG, None);
    wlroots::CompositorBuilder::new().build_auto(()).run()
}
