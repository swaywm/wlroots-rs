extern crate wlroots;

struct InputManager;
struct OutputManager;

impl wlroots::OutputManagerHandler for OutputManager {}
impl wlroots::InputManagerHandler for InputManager {}

fn main() {
    use wlroots::utils::*;
    init_logging(L_DEBUG, None);
    wlroots::CompositorBuilder::new().build_auto((),
                                                 Box::new(InputManager),
                                                 Box::new(OutputManager))
                                     .run()
}
