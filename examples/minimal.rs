extern crate wlroots;

struct InputManager;
struct OutputManager;

impl wlroots::manager::OutputManagerHandler for OutputManager {}
impl wlroots::manager::InputManagerHandler for InputManager {}

fn main() {
    wlroots::compositor::Compositor::new(Box::new(InputManager), Box::new(OutputManager)).run()
}
