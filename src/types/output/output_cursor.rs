//! TODO documentation

use wlroots_sys::{wlr_output_cursor, wlr_output_cursor_create, wlr_output_cursor_destroy,
                  wlr_output_cursor_move, wlr_output_cursor_set_image,
                  wlr_output_cursor_set_surface};

use {Image, Output, OutputHandle, Surface, UpgradeHandleErr};

#[derive(Debug, Eq, PartialEq)]
pub struct OutputCursor {
    cursor: *mut wlr_output_cursor,
    output_handle: OutputHandle
}

impl OutputCursor {
    /// Creates a new `OutputCursor` that's bound to the given `Output`.
    ///
    /// When the `Output` is destroyed, this can no longer be used.
    ///
    /// # Ergonomics
    /// TODO Put in module documentation
    ///
    /// To make this easier for you, I would suggest putting the `OutputCursor` in your
    /// `OutputHandler` implementor's state so that when the `Output` is removed you
    /// just don't have to think about it and it will clean itself up by itself.
    pub fn new<'output>(output: &'output mut Output) -> Option<OutputCursor> {
        unsafe {
            let output_handle = output.weak_reference();
            let cursor = wlr_output_cursor_create(output.as_ptr());
            if cursor.is_null() {
                None
            } else {
                Some(OutputCursor { cursor,
                                    output_handle })
            }
        }
    }

    /// Sets the hardware cursor's image.
    pub fn set_image(&mut self, image: &Image) -> bool {
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
                Err(UpgradeHandleErr::AlreadyDropped) => false,
                err @ Err(UpgradeHandleErr::AlreadyBorrowed) => panic!(err)
            }
        }
    }

    /// Sets the hardware cursor's surface.
    pub fn set_surface(&mut self, surface: Surface, hotspot_x: i32, hotspot_y: i32) {
        unsafe {
            let cursor = self.cursor;
            let res = self.output_handle.run(|_| {
                                                 wlr_output_cursor_set_surface(cursor,
                                                                               surface.as_ptr(),
                                                                               hotspot_x,
                                                                               hotspot_y)
                                             });
            match res {
                Ok(_) | Err(UpgradeHandleErr::AlreadyDropped) => {}
                err @ Err(UpgradeHandleErr::AlreadyBorrowed) => panic!(err)
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
                Err(UpgradeHandleErr::AlreadyDropped) => false,
                err @ Err(UpgradeHandleErr::AlreadyBorrowed) => panic!(err)
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
}

impl Drop for OutputCursor {
    fn drop(&mut self) {
        unsafe { wlr_output_cursor_destroy(self.cursor) }
    }
}
