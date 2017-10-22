//! Wrapper for wlr_cursor

use output::OutputLayout;
use std::{ptr, slice, mem};
use utils::safe_as_cstring;
use wlroots_sys::{wlr_cursor, wlr_cursor_attach_output_layout, wlr_cursor_create,
                  wlr_cursor_destroy, wlr_cursor_set_xcursor, wlr_xcursor, wlr_xcursor_theme,
                  wlr_xcursor_theme_get_cursor, wlr_xcursor_theme_load, wlr_xcursor_image};

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

    // TODO What's stopping me from not droping the xcursor now?
    pub unsafe fn set_xcursor(&mut self, xcursor: &mut XCursor) {
        wlr_cursor_set_xcursor(self.cursor, xcursor.as_raw())
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
    pub fn as_raw(&mut self) -> *mut wlr_xcursor {
        self.xcursor
    }

    pub fn images<'cursor>(&'cursor self) -> Vec<XCursorImage<'cursor>> {
        unsafe {
            let image_ptr = (*self.xcursor).images as *const *const wlr_xcursor_image;
            let length = (*self.xcursor).image_count;
            let cursors_slice: &'cursor [*const wlr_xcursor_image] =
                slice::from_raw_parts::<'cursor, *const wlr_xcursor_image>(image_ptr, length as usize);
            let mut result = Vec::with_capacity(cursors_slice.len());
            for cursor in cursors_slice {
                result.push(
                    XCursorImage {
                        width: (**cursor).width,
                        height: (**cursor).height,
                        hotspot_x: (**cursor).hotspot_x,
                        hotspot_y: (**cursor).hotspot_y,
                        delay: (**cursor).delay,
                        buffer: slice::from_raw_parts::<'cursor, u8>((**cursor).buffer as *const u8,
                                                                     (**cursor).width as usize *
                                                                     (**cursor).height as usize *
                                                                     mem::size_of::<u32>())})
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
