//! Wrapper for wlr_cursor

use std::ptr;

use wlroots_sys::{wlr_cursor, wlr_cursor_create, wlr_cursor_destroy, wlr_cursor_move,
                  wlr_cursor_set_image, wlr_cursor_warp};

use {InputDevice, XCursorImage};

#[derive(Debug)]
pub struct CursorBuilder {
    cursor: *mut wlr_cursor
}

#[derive(Debug)]
pub struct Cursor {
    cursor: *mut wlr_cursor
}

impl CursorBuilder {
    pub fn new() -> Option<Self> {
        unsafe {
            let cursor = wlr_cursor_create();
            if cursor.is_null() {
                None
            } else {
                Some(CursorBuilder { cursor: cursor })
            }
        }
    }

    /// Sets the image of the cursor to the image from the XCursor.
    pub fn set_cursor_image(self, image: &XCursorImage) -> Self {
        unsafe {
            let scale = 0.0;
            // NOTE Rationale for why lifetime isn't attached:
            //
            // wlr_cursor_set_image uses gl calls internally, which copies
            // the buffer and so it doesn't matter what happens to the
            // xcursor image after this call.
            wlr_cursor_set_image(self.cursor,
                                 image.buffer.as_ptr(),
                                 image.width as i32,
                                 image.width,
                                 image.height,
                                 image.hotspot_x as i32,
                                 image.hotspot_y as i32,
                                 scale)
        }
        self
    }

    pub(crate) fn build(self) -> Cursor {
        Cursor { cursor: self.cursor }
    }
}

impl Cursor {
    pub fn coords(&self) -> (f64, f64) {
        unsafe { ((*self.cursor).x, (*self.cursor).y) }
    }

    pub fn warp(&mut self, dev: Option<&InputDevice>, x: f64, y: f64) -> bool {
        unsafe {
            let dev_ptr = dev.map(|dev| dev.as_ptr()).unwrap_or(ptr::null_mut());
            wlr_cursor_warp(self.cursor, dev_ptr, x, y)
        }
    }

    pub fn move_to(&mut self, dev: &InputDevice, delta_x: f64, delta_y: f64) {
        unsafe { wlr_cursor_move(self.cursor, dev.as_ptr(), delta_x, delta_y) }
    }

    /// Sets the image of the cursor to the image from the XCursor.
    pub fn set_cursor_image(&mut self, image: &XCursorImage) {
        unsafe {
            let scale = 0.0;
            // NOTE Rationale for why lifetime isn't attached:
            //
            // wlr_cursor_set_image uses gl calls internally, which copies
            // the buffer and so it doesn't matter what happens to the
            // xcursor image after this call.
            wlr_cursor_set_image(self.cursor,
                                 image.buffer.as_ptr(),
                                 image.width as i32,
                                 image.width,
                                 image.height,
                                 image.hotspot_x as i32,
                                 image.hotspot_y as i32,
                                 scale)
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_cursor {
        self.cursor
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe { wlr_cursor_destroy(self.cursor) }
    }
}
