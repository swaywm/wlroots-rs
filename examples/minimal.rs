extern crate wlroots;

fn main() {
    wlroots::utils::init_logging(wlroots::utils::WLR_DEBUG, None);
    wlroots::CompositorBuilder::new().build_auto(()).run()
}
