//! [From the man page](ftp://www.x.org/pub/X11R7.7/doc/man/man3/Xcursor.3.xhtml):
//! > Xcursor is a simple library designed to help locate and load cursors.
//! > Cursors can be loaded from files or memory. A library of common cursors
//! > exists which map to the standard X cursor names. Cursors can exist in
//! > several sizes and the library automatically picks the best size.

use std::{mem, ptr, slice};
use std::marker::PhantomData;
use std::time::Duration;

use libc::{c_int, c_uint};
use wlroots_sys::{wlr_xcursor, wlr_xcursor_frame, wlr_xcursor_image, wlr_xcursor_theme,
                  wlr_xcursor_theme_destroy, wlr_xcursor_theme_get_cursor, wlr_xcursor_theme_load};

use utils::{c_to_rust_string, safe_as_cstring};
#[cfg(feature = "unstable")]
pub use super::xcursor_manager::*;

#[derive(Debug)]
pub struct Theme {
    theme: *mut wlr_xcursor_theme
}

/// An XCursor from the X11 xcursor library. `  wl`
#[derive(Debug)]
pub struct XCursor<'theme> {
    xcursor: *mut wlr_xcursor,
    phantom: PhantomData<&'theme Theme>
}

/// Contains the information necessary to render the cursor.
///
/// The hotspot of the image shows where the cursor will interact with the
/// environment. i.e. it refers to the location of the cursor "point".
/// Note that the coordinates could be inside the image.
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Image<'cursor> {
    /// The width of the image in pixels.
    pub width: u32,
    /// The height of the image in pixels.
    pub height: u32,
    /// The x coordinate of the hotspot, which must be inside the image.
    pub hotspot_x: u32,
    /// The y coordinate of the hotspot, which must be inside the image.
    pub hotspot_y: u32,
    /// Animation delay to the next frame in milliseconds.
    pub delay: u32,
    /// The bytes in ARGB format.
    pub buffer: &'cursor [u8],
    /// A marker to indicate you can't send this type across threads.
    /// Also ensures you can't construct it outside of this library.
    #[doc(hidden)]
    _no_send: PhantomData<*mut wlr_xcursor_image>
}

impl Theme {
    pub(crate) unsafe fn new(theme: *mut wlr_xcursor_theme) -> Theme {
        Theme { theme }
    }

    /// If no name is given, defaults to "default".
    pub fn load_theme<T: Into<Option<String>>>(name: T, size: i32) -> Option<Self> {
        unsafe {
            let name_str = name.into().map(safe_as_cstring);
            let name_ptr = name_str.map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
            let theme = wlr_xcursor_theme_load(name_ptr, size);
            if theme.is_null() {
                None
            } else {
                Some(Theme { theme })
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
        let xcursor = unsafe { wlr_xcursor_theme_get_cursor(self.theme, name_str.as_ptr()) };
        if xcursor.is_null() {
            None
        } else {
            Some(XCursor { xcursor,
                           phantom: PhantomData })
        }
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe { wlr_xcursor_theme_destroy(self.theme) }
    }
}

impl<'theme> XCursor<'theme> {
    /// NOTE this lifetime is defined by the user of the function, but it must not outlive the
    /// `XCursorManager` that hosts the xcursor.
    pub(crate) unsafe fn from_ptr<'unbound>(xcursor: *mut wlr_xcursor) -> XCursor<'unbound> {
        XCursor { xcursor,
                  phantom: PhantomData }
    }

    pub fn frame(&mut self, duration: Duration) -> c_int {
        unsafe {
            // TODO Is the correct unit of time?
            wlr_xcursor_frame(self.xcursor, duration.subsec_nanos())
        }
    }

    pub fn images<'cursor>(&'cursor self) -> Vec<Image<'cursor>> {
        unsafe {
            let image_ptr = (*self.xcursor).images as *const *const wlr_xcursor_image;
            let length = (*self.xcursor).image_count;
            let cursors_slice: &'cursor [*const wlr_xcursor_image] =
                slice::from_raw_parts::<'cursor, *const wlr_xcursor_image>(image_ptr,
                                                                           length as usize);
            let mut result = Vec::with_capacity(cursors_slice.len());
            for cursor in cursors_slice {
                result.push(Image {
                    width: (**cursor).width,
                    height: (**cursor).height,
                    hotspot_x: (**cursor).hotspot_x,
                    hotspot_y: (**cursor).hotspot_y,
                    delay: (**cursor).delay,
                    buffer: slice::from_raw_parts::<'cursor, u8>(
                        (**cursor).buffer as *const u8,
                        (**cursor).width as usize * (**cursor).height as usize
                            * mem::size_of::<u32>()
                    ),
                    _no_send: PhantomData
                })
            }
            result
        }
    }
}
