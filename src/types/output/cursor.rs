//! TODO documentation

use std::ptr;

use wlroots_sys::{
    wlr_output_cursor, wlr_output_cursor_create, wlr_output_cursor_destroy, wlr_output_cursor_move,
    wlr_output_cursor_set_image, wlr_output_cursor_set_surface
};

use crate::{
    output::{self, Output},
    render,
    surface::{self, Surface},
    utils::{HandleErr, HandleResult, Handleable}
};

#[derive(Debug, Eq, PartialEq)]
pub struct Cursor {
    cursor: *mut wlr_output_cursor,
    output_handle: output::Handle
}

impl Cursor {
    /// Creates a new `output::Cursor` that's bound to the given `Output`.
    ///
    /// When the `Output` is destroyed each call will return an Error.
    ///
    /// # Ergonomics
    ///
    /// To make this easier for you, I would suggest putting the
    /// `output::Cursor` in your `OutputHandler` implementor's state so that
    /// when the `Output` is removed you just don't have to think about it
    /// and it will clean itself up by itself.
    pub fn new(output: &mut Output) -> Option<Cursor> {
        unsafe {
            let output_handle = output.weak_reference();
            let cursor = wlr_output_cursor_create(output.as_ptr());
            if cursor.is_null() {
                None
            } else {
                Some(Cursor {
                    cursor,
                    output_handle
                })
            }
        }
    }

    /// Return a copy of the output handle used by this cursor.
    ///
    /// There are no guarantees that it is valid to use.
    pub fn output(&self) -> output::Handle {
        self.output_handle.clone()
    }

    /// Sets the hardware cursor's image.
    pub fn set_image(&mut self, image: &render::Image) -> HandleResult<bool> {
        unsafe {
            let cursor = self.cursor;
            if !self.output_handle.is_alive() {
                return Err(HandleErr::AlreadyDropped);
            }
            Ok(wlr_output_cursor_set_image(
                cursor,
                image.pixels.as_ptr(),
                image.stride,
                image.width,
                image.height,
                image.hotspot_x,
                image.hotspot_y
            ))
        }
    }

    /// Sets the hardware cursor's surface.
    pub fn set_surface<'a, T>(&mut self, surface: T, hotspot_x: i32, hotspot_y: i32) -> HandleResult<()>
    where
        T: Into<Option<&'a Surface>>
    {
        unsafe {
            let surface_ptr = surface
                .into()
                .map(|surface| surface.as_ptr())
                .unwrap_or_else(|| ptr::null_mut());
            if !self.output_handle.is_alive() {
                return Err(HandleErr::AlreadyDropped);
            }
            Ok(wlr_output_cursor_set_surface(
                self.cursor,
                surface_ptr,
                hotspot_x,
                hotspot_y
            ))
        }
    }

    /// Moves the hardware cursor to the desired location
    pub fn move_relative(&mut self, x: f64, y: f64) -> HandleResult<bool> {
        unsafe {
            if !self.output_handle.is_alive() {
                return Err(HandleErr::AlreadyDropped);
            }
            Ok(wlr_output_cursor_move(self.cursor, x, y))
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
