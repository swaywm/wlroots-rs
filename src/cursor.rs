//! Wrapper for wlr_cursor

use device::Device;
use output::OutputLayout;
use std::{mem, ptr, slice};
use std::cell::RefCell;
use std::rc::Rc;
use utils::safe_as_cstring;
use wlroots_sys::{wlr_cursor, wlr_cursor_attach_output_layout, wlr_cursor_create,
                  wlr_cursor_destroy, wlr_cursor_set_xcursor, wlr_cursor_warp, wlr_xcursor,
                  wlr_xcursor_image, wlr_xcursor_theme, wlr_xcursor_theme_get_cursor,
                  wlr_xcursor_theme_load};

#[derive(Debug)]
pub struct Cursor {
    cursor: *mut wlr_cursor,
    xcursor: Option<XCursor>,
    layout: Option<Rc<RefCell<OutputLayout>>>
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
                Some(Cursor {
                         cursor,
                         xcursor: None,
                         layout: None
                     })
            }
        }
    }

    pub fn coords(&self) -> (f64, f64) {
        unsafe { ((*self.cursor).x, (*self.cursor).y) }
    }

    pub fn warp(&mut self, dev: Option<Device>, x: f64, y: f64) -> bool {
        unsafe {
            let dev_ptr = dev.map(|dev| dev.to_ptr()).unwrap_or(ptr::null_mut());
            wlr_cursor_warp(self.cursor, dev_ptr, x, y)
        }
    }

    pub fn set_xcursor(&mut self, xcursor: Option<XCursor>) {
        self.xcursor = xcursor;
        unsafe {
            let xcursor_ptr = self.xcursor
                .as_mut()
                .map(|xcursor| xcursor.as_raw())
                .unwrap_or(ptr::null_mut());
            wlr_cursor_set_xcursor(self.cursor, xcursor_ptr)
        }
    }

    pub fn xcursor(&self) -> Option<&XCursor> {
        self.xcursor.as_ref()
    }

    /// Attaches an output layout to the cursor.
    /// The layout specifies the boundaries of the cursor, i.e where it can go.
    pub fn attach_output_layout(&mut self, layout: Rc<RefCell<OutputLayout>>) {
        unsafe {
            // NOTE Rationale for why the pointer isn't leaked from the refcell:
            // * A pointer is not stored to the layout, the internal state is just updated.
            wlr_cursor_attach_output_layout(self.cursor, layout.borrow_mut().as_ptr());
            self.layout = Some(layout);
        }
    }

    pub fn output_layout(&self) -> &Option<Rc<RefCell<OutputLayout>>> {
        &self.layout
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
    pub unsafe fn as_raw(&mut self) -> *mut wlr_xcursor {
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
                                buffer: slice::from_raw_parts::<'cursor, u8>((**cursor).buffer as
                                                                             *const u8,
                                                                             (**cursor).width as
                                                                             usize *
                                                                             (**cursor).height as
                                                                             usize *
                                                                             mem::size_of::<u32>())
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
