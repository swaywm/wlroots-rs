//! Wrappers around the XCursor library used by wlroots.
//!
//! [From the man page](ftp://www.x.org/pub/X11R7.7/doc/man/man3/Xcursor.3.xhtml):
//! > Xcursor is a simple library designed to help locate and load cursors.
//! > Cursors can be loaded from files or memory. A library of common cursors
//! > exists which map to the standard X cursor names. Cursors can exist in
//! > several sizes and the library automatically picks the best size.

use std::marker::PhantomData;
use std::time::Duration;
use std::{mem, ptr, slice};

use crate::libc::c_int;
use wlroots_sys::{
    wlr_xcursor, wlr_xcursor_frame, wlr_xcursor_image, wlr_xcursor_theme, wlr_xcursor_theme_destroy,
    wlr_xcursor_theme_get_cursor, wlr_xcursor_theme_load
};

#[cfg(feature = "unstable")]
pub use super::xcursor_manager::*;
use crate::utils::{c_to_rust_string, safe_as_cstring};

/// Wrapper for an xcursor theme from the X11 xcursor library.
///
/// Xcursor (mostly) follows the freedesktop.org spec for theming icons.
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Theme {
    theme: *mut wlr_xcursor_theme
}

/// Wrapper for an xcursor from the X11 xcursor library.
///
/// The cursor must live as long as the [`Theme`](struct.Theme.html)
/// that it comes from.
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
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
    pub(crate) _no_send: PhantomData<*mut wlr_xcursor_image>
}

impl Theme {
    /// Loads the named xcursor theme at the given cursor size (in pixels).
    ///
    /// This is useful if you need cursor images for your compositor to use when
    /// a client-side cursors is not available or you wish to override
    /// client-side cursors for a particular UI interaction (such as using a
    /// grab cursor when moving a window around)
    ///
    /// The default search path it uses is ~/.icons, /usr/share/icons,
    /// /usr/share/pixmaps. Within each of these directories, it searches
    /// for a directory using the theme name. Within the theme directory, it
    /// looks for cursor files in the 'cursors' subdirectory. It uses the
    /// first cursor file found along the path.
    ///
    /// If necessary, Xcursor also looks for a "index.theme" file in each theme
    /// directory to find inherited themes and searches along the path for
    /// those themes as well.
    ///
    /// If no name is given, defaults to "default".
    /// If no theme can be found `None` is returned.
    pub fn load_theme<T: Into<Option<String>>>(name: T, size: c_int) -> Option<Self> {
        unsafe {
            let name_str = name.into().map(safe_as_cstring);
            let name_ptr = name_str.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
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
    /// If the name returned by wlroots was malformed, or nonexistent,
    /// then the value will be None.
    pub fn name(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.theme).name) }
    }

    /// Get the size of the images.
    pub fn size(&self) -> c_int {
        unsafe { (*self.theme).size }
    }

    /// Get the number of cursors in this theme.
    pub fn cursor_count(&self) -> usize {
        unsafe { (*self.theme).cursor_count as usize }
    }

    /// Gets all the cursors from this theme.
    pub fn cursors<'theme>(&'theme mut self) -> Vec<XCursor<'theme>> {
        unsafe {
            let cursor_ptr = (*self.theme).cursors as *mut *mut wlr_xcursor;
            let length = self.cursor_count();
            let xcursors_slice: &'theme [*mut wlr_xcursor] =
                slice::from_raw_parts::<'theme, *mut wlr_xcursor>(cursor_ptr, length);
            xcursors_slice
                .iter()
                .map(|&xcursor| XCursor::from_ptr(xcursor))
                .collect()
        }
    }

    /// Get the cursor with the provided name (e.g. "left_ptr"), if it exists.
    pub fn get_cursor(&self, name: String) -> Option<XCursor> {
        unsafe {
            let name_str = safe_as_cstring(name);
            let xcursor = wlr_xcursor_theme_get_cursor(self.theme, name_str.as_ptr());
            if xcursor.is_null() {
                None
            } else {
                Some(XCursor::from_ptr(xcursor))
            }
        }
    }

    /// Constructs a `Theme` from a raw pointer.
    ///
    /// # Unsafety
    /// Takes ownership of the cursor theme. When the `Theme` is dropped it
    /// calls `wlr_cursor_theme_destroy`.
    pub unsafe fn from_ptr(theme: *mut wlr_xcursor_theme) -> Theme {
        Theme { theme }
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe { wlr_xcursor_theme_destroy(self.theme) }
    }
}

impl<'theme> XCursor<'theme> {
    /// Get all the image frames associated with an xcursor.
    pub fn images<'cursor>(&'cursor self) -> Vec<Image<'cursor>> {
        unsafe {
            let cursors_slice = self.cursor_slice();
            let mut result = Vec::with_capacity(cursors_slice.len());
            for cursor in cursors_slice {
                result.push(Image::from_xcursor_image(*cursor))
            }
            result
        }
    }

    /// Get a specific frame of the image from the animation. Returns `None` if
    /// the index is out of bounds.
    ///
    /// We suggest paring this with `XCursor::frame` to avoid going out of
    /// bounds.
    pub fn image(&self, index: usize) -> Option<Image> {
        unsafe {
            let cursors_slice = self.cursor_slice();
            cursors_slice
                .get(index)
                .map(|&cursor| Image::from_xcursor_image(cursor))
        }
    }

    unsafe fn cursor_slice(&self) -> &[*mut wlr_xcursor_image] {
        let image_ptr = (*self.xcursor).images as *mut *mut wlr_xcursor_image;
        slice::from_raw_parts::<'_, *mut wlr_xcursor_image>(image_ptr, self.image_len())
    }

    /// Returns the current frame number for an animated cursor give a
    /// monotonic time reference in milliseconds.
    ///
    /// e.g. if it's been animating for 500 milliseconds `duration`
    /// should be constructed from `Duration::from_millis(500)`.
    pub fn frame(&mut self, duration: Duration) -> usize {
        unsafe { wlr_xcursor_frame(self.xcursor, duration.subsec_millis()) as usize }
    }

    /// Get the number of animation frames for the cursor.
    pub fn image_len(&self) -> usize {
        unsafe { (*self.xcursor).image_count as _ }
    }

    /// Length of the animation in milliseconds.
    pub fn total_delay(&self) -> u32 {
        unsafe { (*self.xcursor).total_delay }
    }

    /// Constructs an `XCursor` from a raw pointer
    ///
    /// # Unsafety
    /// This lifetime is unbounded, but it must not outlive the
    /// xcursor manager that the pointer came from.
    pub unsafe fn from_ptr<'unbound>(xcursor: *mut wlr_xcursor) -> XCursor<'unbound> {
        XCursor {
            xcursor,
            phantom: PhantomData
        }
    }
}

impl<'unbound> Image<'unbound> {
    unsafe fn from_xcursor_image(image: *mut wlr_xcursor_image) -> Self {
        Image {
            width: (*image).width,
            height: (*image).height,
            hotspot_x: (*image).hotspot_x,
            hotspot_y: (*image).hotspot_y,
            delay: (*image).delay,
            buffer: slice::from_raw_parts::<'_, u8>(
                (*image).buffer as *const u8,
                (*image).width as usize * (*image).height as usize * mem::size_of::<u32>()
            ),
            _no_send: PhantomData
        }
    }
}
