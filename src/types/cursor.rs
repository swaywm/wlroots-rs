//! Wrapper for wlr_cursor

use std::{mem, ptr, slice};

use wlroots_sys::{wlr_cursor, wlr_cursor_create, wlr_cursor_destroy, wlr_cursor_move,
                  wlr_cursor_set_image, wlr_cursor_warp, wlr_xcursor, wlr_xcursor_image,
                  wlr_xcursor_theme, wlr_xcursor_theme_get_cursor, wlr_xcursor_theme_load};

use InputDevice;
use utils::safe_as_cstring;

#[derive(Debug)]
pub struct CursorBuilder {
    cursor: *mut wlr_cursor
}

#[derive(Debug)]
pub struct Cursor {
    cursor: *mut wlr_cursor
}

#[derive(Debug)]
pub struct XCursorTheme {
    theme: *mut wlr_xcursor_theme
}

#[derive(Debug)]
pub struct XCursor {
    xcursor: *mut wlr_xcursor
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

impl XCursorTheme {
    /// If no name is given, defaults to "default".
    pub fn load_theme(name: Option<String>, size: i32) -> Option<Self> {
        unsafe {
            let name_str = name.map(safe_as_cstring);
            let name_ptr = name_str.map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
            let theme = wlr_xcursor_theme_load(name_ptr, size);
            if theme.is_null() {
                None
            } else {
                Some(XCursorTheme { theme })
            }
        }
    }

    pub fn get_cursor(&self, name: String) -> Option<XCursor> {
        let name_str = safe_as_cstring(name);
        let xcursor =
            unsafe { wlr_xcursor_theme_get_cursor(self.theme, name_str.as_ptr()) };
        if xcursor.is_null() {
            None
        } else {
            Some(XCursor { xcursor })
        }
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn as_ptr(self) -> *mut wlr_xcursor_theme {
        self.theme
    }
}

impl XCursor {
    #[allow(dead_code)]
    pub(crate) unsafe fn as_ptr(&mut self) -> *mut wlr_xcursor {
        self.xcursor
    }

    pub fn images<'cursor>(&'cursor self) -> Vec<XCursorImage<'cursor>> {
        unsafe {
            let image_ptr = (*self.xcursor).images as *const *const wlr_xcursor_image;
            let length = (*self.xcursor).image_count;
            let cursors_slice: &'cursor [*const wlr_xcursor_image] =
                slice::from_raw_parts::<'cursor, *const wlr_xcursor_image>(image_ptr,
                                                                           length as usize);
            let mut result = Vec::with_capacity(cursors_slice.len());
            for cursor in cursors_slice {
                result.push(XCursorImage {
                    width: (**cursor).width,
                    height: (**cursor).height,
                    hotspot_x: (**cursor).hotspot_x,
                    hotspot_y: (**cursor).hotspot_y,
                    delay: (**cursor).delay,
                    buffer: slice::from_raw_parts::<'cursor, u8>(
                        (**cursor).buffer as *const u8,
                        (**cursor).width as usize * (**cursor).height as usize
                            * mem::size_of::<u32>()
                    )
                })
            }
            result
        }
    }
}

#[derive(Debug)]
pub struct XCursorImage<'cursor> {
    pub width: u32,
    pub height: u32,
    pub hotspot_x: u32,
    pub hotspot_y: u32,
    pub delay: u32,
    pub buffer: &'cursor [u8]
}
