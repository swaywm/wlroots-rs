//! TODO documentation

use std::ptr;

use wlroots_sys::{wlr_output_cursor, wlr_output_cursor_create, wlr_output_cursor_destroy,
                  wlr_output_cursor_move, wlr_output_cursor_set_image,
                  wlr_output_cursor_set_surface};

use {render,
     output::{self, Output},
     surface::{self, Surface},
     utils::{HandleErr, Handleable}};

#[derive(Debug, Eq, PartialEq)]
pub struct Cursor {
    cursor: *mut wlr_output_cursor,
    output_handle: output::Handle
}

impl Cursor {
    /// Creates a new `output::Cursor` that's bound to the given `Output`.
    ///
    /// When the `Output` is destroyed, this can no longer be used.
    ///
    /// # Ergonomics
    ///
    /// To make this easier for you, I would suggest putting the `output::Cursor` in your
    /// `OutputHandler` implementor's state so that when the `Output` is removed you
    /// just don't have to think about it and it will clean itself up by itself.
    pub fn new(output: &mut Output) -> Option<Cursor> {
        unsafe {
            let output_handle = output.weak_reference();
            let cursor = wlr_output_cursor_create(output.as_ptr());
            if cursor.is_null() {
                None
            } else {
                Some(Cursor { cursor,
                              output_handle })
            }
        }
    }

    /// Sets the hardware cursor's image.
    pub fn set_image(&mut self, image: &render::Image) -> bool {
        unsafe {
            let cursor = self.cursor;
            let res = self.output_handle.run(|_| {
                wlr_output_cursor_set_image(cursor,
                                            image.pixels.as_ptr(),
                                            image.stride,
                                            image.width,
                                            image.height,
                                            image.hotspot_x,
                                            image.hotspot_y)
            });
            match res {
                Ok(res) => res,
                Err(HandleErr::AlreadyDropped) => false,
                err @ Err(HandleErr::AlreadyBorrowed) => panic!(err)
            }
        }
    }

    /// Sets the hardware cursor's surface.
    pub fn set_surface<T>(&mut self, surface: T, hotspot_x: i32, hotspot_y: i32)
    where T: Into<Option<Surface>>
    {
        unsafe {
            let surface_ptr = surface.into()
                .map(|surface| surface.as_ptr())
                .unwrap_or_else(|| ptr::null_mut());
            let cursor = self.cursor;
            let res = self.output_handle.run(|_| {
                wlr_output_cursor_set_surface(cursor,
                                              surface_ptr,
                                              hotspot_x,
                                              hotspot_y)
            });
            match res {
                Ok(_) | Err(HandleErr::AlreadyDropped) => {}
                err @ Err(HandleErr::AlreadyBorrowed) => panic!(err)
            }
        }
    }

    /// Moves the hardware cursor to the desired location
    pub fn move_to(&mut self, x: f64, y: f64) -> bool {
        unsafe {
            let cursor = self.cursor;
            let res = self.output_handle.run(|_| wlr_output_cursor_move(cursor, x, y));
            match res {
                Ok(res) => res,
                Err(HandleErr::AlreadyDropped) => false,
                err @ Err(HandleErr::AlreadyBorrowed) => panic!(err)
            }
        }
    }

    /// Get the coordinates of the cursor.
    ///
    /// Returned value is in (x, y) format.
    pub fn coords(&self) -> (f64, f64) {
        unsafe { ((*self.cursor).x, (*self.cursor).y) }
    }

    /// Determines if the hardware cursor is enabled or not.
    pub fn enabled(&self) -> bool {
        unsafe { (*self.cursor).enabled }
    }

    /// Determines if the hardware cursor is visible or not.
    pub fn visible(&self) -> bool {
        unsafe { (*self.cursor).visible }
    }

    /// Gets the width and height of the hardware cursor.
    ///
    /// Returned value is in (width, height) format.
    pub fn size(&self) -> (u32, u32) {
        unsafe { ((*self.cursor).width, (*self.cursor).height) }
    }

    /// Gets the hotspot coordinates of the hardware cursor.
    ///
    /// Returned value is in (x, y) coordinates.
    pub fn hotspots(&self) -> (i32, i32) {
        unsafe { ((*self.cursor).hotspot_x, (*self.cursor).hotspot_y) }
    }

    /// Gets the texture for the cursor, if a software cursor is used without a
    /// surface.
    pub fn texture<'surface>(&'surface self) -> Option<render::Texture<'surface>> {
        unsafe {
            let texture = (*self.cursor).texture;
            if texture.is_null() {
                None
            } else {
                Some(render::Texture::from_ptr(texture))
            }
        }
    }

    /// Gets the surface for the cursor, if using a cursor surface.
    pub fn surface(&self) -> Option<surface::Handle> {
        unsafe {
            let surface = (*self.cursor).surface;
            if surface.is_null() {
                None
            } else {
                Some(surface::Handle::from_ptr(surface))
            }
        }
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe { wlr_output_cursor_destroy(self.cursor) }
    }
}
