//! TODO Documentation

use std::{mem, ptr, slice};

use wlroots_sys::{wlr_xcursor, wlr_xcursor_image, wlr_xcursor_theme, wlr_xcursor_theme_get_cursor,
                  wlr_xcursor_theme_load};

use utils::safe_as_cstring;

#[derive(Debug)]
pub struct XCursorTheme {
    theme: *mut wlr_xcursor_theme
}

#[derive(Debug)]
pub struct XCursor {
    xcursor: *mut wlr_xcursor
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
