//! Wrapper for wlr_cursor

use output::OutputLayout;
use std::ptr;
use utils::safe_as_cstring;
use wlroots_sys::{wlr_cursor, wlr_cursor_attach_output_layout, wlr_cursor_create,
                  wlr_cursor_destroy, wlr_cursor_set_xcursor, wlr_xcursor, wlr_xcursor_theme,
                  wlr_xcursor_theme_get_cursor, wlr_xcursor_theme_load};

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

impl Cursor {
    pub fn new() -> Option<Cursor> {
        unsafe {
            let cursor = wlr_cursor_create();
            if cursor.is_null() {
                None
            } else {
                Some(Cursor { cursor })
            }
        }
    }

    pub fn set_xcursor(&mut self, xcursor: XCursor) {
        unsafe { wlr_cursor_set_xcursor(self.cursor, xcursor.into_raw()) }
    }

    pub unsafe fn attach_output_layout(&mut self, layout: &mut OutputLayout) {
        // FIXME TODO How do we ensure that layout lives long enough for the
        // cursor to use the pointer? We control the destrucion of layout,
        // which dies when its scope ends. Perhaps we need lifetime annotations here.
        wlr_cursor_attach_output_layout(self.cursor, layout.as_ptr())
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
        let xcursor = unsafe { wlr_xcursor_theme_get_cursor(self.theme, name_str.as_ptr()) };
        if xcursor.is_null() {
            None
        } else {
            Some(XCursor { xcursor })
        }
    }

    pub fn into_raw(self) -> *mut wlr_xcursor_theme {
        self.theme
    }
}

impl XCursor {
    pub fn into_raw(self) -> *mut wlr_xcursor {
        self.xcursor
    }
}
