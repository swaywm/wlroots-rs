extern crate wlroots;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wlroots::compositor::Compositor;
use wlroots::cursor::{Cursor, XCursor, XCursorTheme};
use wlroots::device::Device;
use wlroots::key_event::KeyEvent;
use wlroots::manager::{InputManagerHandler, OutputManagerHandler};
use wlroots::output::{Output, OutputLayout};
use wlroots::pointer;
use wlroots::wlroots_sys::gl;
use wlroots::wlroots_sys::wlr_button_state::{WLR_BUTTON_PRESSED, WLR_BUTTON_RELEASED};
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct OutputHandler {
    pub layout: OutputLayout,
    color: Rc<Cell<[f32; 4]>>,
    cursor: Rc<RefCell<Cursor>>,
    xcursor: Rc<RefCell<XCursor>>
}

struct InputHandler {
    color: Rc<Cell<[f32; 4]>>,
    default_color: [f32; 4],
    cursor: Rc<RefCell<Cursor>>,
    xcursor: Rc<RefCell<XCursor>>
}

impl OutputManagerHandler for OutputHandler {
    fn output_added(&mut self, output: Output) {
        // TODO set cursor on screen
        // To do that, we need to have cursor and xcursor shared between
        // the InputHandler and the OutputHandler...
        // hmm, This screams Rc + Refcell, but I don't want to do that unless
        // I have to.
    }
}

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

fn managers(mut cursor: Cursor, xcursor: XCursor) -> (OutputHandler, InputHandler) {
    let mut layout = OutputLayout::new();
    // TODO Ensure this can be safe...
    // e.g what's stopping me from simply dropping layout now that I gave it to
    // cursor?
    unsafe {
        cursor.attach_output_layout(&mut layout);
    }
    let cursor = Rc::new(RefCell::new(cursor));
    let xcursor = Rc::new(RefCell::new(xcursor));
    let color = Rc::new(Cell::new([0.25, 0.25, 0.25, 1.0]));
    (OutputHandler {
        color: color.clone(),
        layout,
        cursor: cursor.clone(),
        xcursor: xcursor.clone()
     },
     InputHandler {
         color: color.clone(),
         default_color: color.get(),
         cursor: cursor.clone(),
         xcursor: xcursor.clone()
     })
}

fn main() {
    let mut cursor = Cursor::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let mut xcursor = xcursor_theme
        .get_cursor("left_ptr".into())
        .expect("Could not load cursor from theme");
    unsafe {
        cursor.set_xcursor(&mut xcursor);
    }

    let (mut output_manager, input_manager) = managers(cursor, xcursor);
    let compositor = Compositor::new(Box::new(input_manager), Box::new(output_manager));
    compositor.run();
}
