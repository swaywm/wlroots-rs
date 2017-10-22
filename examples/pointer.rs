extern crate wlroots;

use std::cell::Cell;
use std::rc::Rc;
use wlroots::compositor::Compositor;
use wlroots::cursor::{Cursor, XCursor, XCursorTheme};
use wlroots::device::Device;
use wlroots::key_event::KeyEvent;
use wlroots::manager::{InputManagerHandler, OutputManagerHandler};
use wlroots::pointer;
use wlroots::output::Output;
use wlroots::wlroots_sys::{gl};
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;
use wlroots::wlroots_sys::wlr_button_state::{WLR_BUTTON_RELEASED, WLR_BUTTON_PRESSED};

struct OutputHandler {
    color: Rc<Cell<[f32; 4]>>
}

struct InputHandler {
    color: Rc<Cell<[f32; 4]>>,
    default_color: [f32; 4]
}

impl OutputManagerHandler for OutputHandler {}

impl InputManagerHandler for InputHandler {
    fn button(&mut self, event: pointer::ButtonEvent) {
        if event.state() == WLR_BUTTON_RELEASED {
            self.color.set(self.default_color.clone())
        } else {
            let mut red: [f32; 4] = [0.25, 0.25, 0.25, 1.0];
            red[event.button() as usize % 3] = 1.0;
            self.color.set(red);
        }
    }
}

fn managers() -> (OutputHandler, InputHandler) {
    let color = Rc::new(Cell::new([0.25, 0.25, 0.25, 1.0]));
    (OutputHandler { color: color.clone() },
     InputHandler { color: color.clone(), default_color: color.get() })
}

fn main() {
    let mut cursor = Cursor::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let xcursor = xcursor_theme.get_cursor("left_ptr".into()).expect("Could not load cursor from theme");
    cursor.set_xcursor(xcursor);

    let (output_manager, input_manager) = managers();
    let compositor = Compositor::new(Box::new(input_manager), Box::new(output_manager));
    compositor.run();
}
