//! TODO Documentation

use std::{mem, ptr, slice};
use std::marker::PhantomData;
use std::time::Duration;

use libc::{c_int, c_uint};
use wlroots_sys::{wlr_xcursor, wlr_xcursor_frame, wlr_xcursor_image, wlr_xcursor_theme,
                  wlr_xcursor_theme_destroy, wlr_xcursor_theme_get_cursor, wlr_xcursor_theme_load};

use utils::{c_to_rust_string, safe_as_cstring};

#[derive(Debug)]
pub struct XCursorTheme {
    theme: *mut wlr_xcursor_theme
}

#[derive(Debug)]
pub struct XCursor<'theme> {
    xcursor: *mut wlr_xcursor,
    phantom: PhantomData<&'theme XCursorTheme>
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

    /// Get the name of this theme.
    ///
    /// If the name returned by wlroots was malformed, or non existant,
    /// then the value will be None.
    pub fn name(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.theme).name) }
    }

    /// Get the size of the images.
    pub fn size(&self) -> c_int {
        unsafe { (*self.theme).size }
    }

    /// Get the number of cursors in this theme.
    pub fn cursor_count(&self) -> c_uint {
        unsafe { (*self.theme).cursor_count }
    }

    /// Gets all the cursors from this theme.
    pub fn cursors<'theme>(&'theme mut self) -> Vec<XCursor<'theme>> {
        unsafe {
            let cursor_ptr = (*self.theme).cursors as *const *mut wlr_xcursor;
            let length = self.cursor_count() as usize;
            let xcursors_slice: &'theme [*mut wlr_xcursor] =
                slice::from_raw_parts::<'theme, *mut wlr_xcursor>(cursor_ptr, length);
            xcursors_slice.into_iter()
                          .map(|&xcursor| {
                                   XCursor { xcursor,
                                             phantom: PhantomData }
                               })
                          .collect()
        }
    }

    /// Get the cursor with the provided name, if it exists.
    pub fn get_cursor<'theme>(&'theme self, name: String) -> Option<XCursor<'theme>> {
        let name_str = safe_as_cstring(name);
        let xcursor =
            unsafe { wlr_xcursor_theme_get_cursor(self.theme, name_str.as_ptr()) };
        if xcursor.is_null() {
            None
        } else {
            Some(XCursor { xcursor,
                           phantom: PhantomData })
        }
    }
}

impl Drop for XCursorTheme {
    fn drop(&mut self) {
        unsafe { wlr_xcursor_theme_destroy(self.theme) }
    }
}

impl<'theme> XCursor<'theme> {
    pub fn frame(&mut self, duration: Duration) -> c_int {
        unsafe {
            // TODO Is the correct unit of time?
            wlr_xcursor_frame(self.xcursor, duration.subsec_nanos())
        }
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
