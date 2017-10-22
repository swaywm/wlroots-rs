#[macro_use]
extern crate wlroots;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wlroots::compositor::Compositor;
use wlroots::cursor::{Cursor, XCursorTheme};
use wlroots::manager::{InputManagerHandler, OutputManagerHandler};
use wlroots::output::{Output, OutputLayout};
use wlroots::pointer;
use wlroots::wlroots_sys::gl;
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;

struct OutputHandler {
    color: Rc<Cell<[f32; 4]>>,
    cursor: Cursor
}

struct InputHandler {
    color: Rc<Cell<[f32; 4]>>,
    default_color: [f32; 4]
}

impl OutputManagerHandler for OutputHandler {
    fn output_added(&mut self, mut output: Output) {
        let cursor = &mut self.cursor;
        {
            let xcursor = cursor.xcursor().expect("XCursor was not set!");
            let image = &xcursor.images()[0];
            // TODO use output config if present instead of auto
            let layout = cursor
                .output_layout()
                .as_ref()
                .expect("Could not get output layout");
            layout.borrow_mut().add_auto(&mut output);
            if output.set_cursor(image).is_err() {
                wlr_log!(L_DEBUG, "Failed to set hardware cursor");
                return;
            }
        }
        let (x, y) = cursor.coords();
        // https://en.wikipedia.org/wiki/Mouse_warping
        cursor.warp(None, x, y);
    }

    fn output_frame(&mut self, output: Output) {
        output.make_current();
        unsafe {
            gl::ClearColor(self.color.get()[0],
                           self.color.get()[1],
                           self.color.get()[2],
                           1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        output.swap_buffers();
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

fn managers(mut cursor: Cursor) -> (OutputHandler, InputHandler) {
    let layout = Rc::new(RefCell::new(OutputLayout::new()));
    // TODO Ensure this can be safe...
    // e.g what's stopping me from simply dropping layout now that I gave it to
    // cursor?
    cursor.attach_output_layout(layout);
    let cursor = cursor;
    let color = Rc::new(Cell::new([0.25, 0.25, 0.25, 1.0]));
    (OutputHandler {
         color: color.clone(),
         cursor: cursor
     },
     InputHandler {
         color: color.clone(),
         default_color: color.get()
     })
}

fn main() {
    let mut cursor = Cursor::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let xcursor = xcursor_theme
        .get_cursor("left_ptr".into())
        .expect("Could not load cursor from theme");
    cursor.set_xcursor(Some(xcursor));

    let (output_manager, input_manager) = managers(cursor);
    let compositor = Compositor::new(Box::new(input_manager), Box::new(output_manager));
    compositor.run();
}
