extern crate wlroots;

struct InputManager;
struct OutputManager;

impl wlroots::OutputManagerHandler for OutputManager {}
impl wlroots::InputManagerHandler for InputManager {}

fn main() {
    wlroots::Compositor::new(Box::new(InputManager), Box::new(OutputManager)).run()
}
