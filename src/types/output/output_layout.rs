//! TODO Documentation

use wlroots_sys::{wlr_output_effective_resolution, wlr_output_layout, wlr_output_layout_create,
                  wlr_output_layout_destroy, wlr_output_layout_output, wlr_output_layout_remove};

use {Origin, Output};

#[derive(Debug)]
pub struct OutputLayout {
    layout: *mut wlr_output_layout
}

/// The coordinate information of an `Output` within an `OutputLayout`.
#[derive(Debug)]
pub struct OutputLayoutOutput {
    layout_output: *mut wlr_output_layout_output
}

impl OutputLayout {
    pub fn new() -> Option<Self> {
        unsafe {
            let layout = wlr_output_layout_create();
            if layout.is_null() {
                None
            } else {
                Some(OutputLayout { layout })
            }
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_layout {
        self.layout
    }

    /// Remove an output from this layout.
    ///
    /// If the output was not in the layout, does nothing.
    pub fn remove(&mut self, output: &mut Output) {
        unsafe {
            wlr_output_layout_remove(self.layout, output.as_ptr())
        }
    }
}

impl Drop for OutputLayout {
    fn drop(&mut self) {
        unsafe { wlr_output_layout_destroy(self.layout) }
    }
}

impl OutputLayoutOutput {
    /// Get the absolute top left edge coordinate of this output in the output
    /// layout.
    pub fn top_left_edge(&self) -> Origin {
        unsafe { Origin::new((*self.layout_output).x, (*self.layout_output).y) }
    }

    /// Get the absolute top right edge coordinate of this output in the output
    /// layout.
    pub fn top_right_edge(&self) -> Origin {
        unsafe {
            let (mut width, mut _height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut width, &mut _height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x + width, y)
        }
    }

    pub fn bottom_left_edge(&self) -> Origin {
        unsafe {
            let (mut _width, mut height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut _width, &mut height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x, y + height)
        }
    }

    pub fn bottom_right_edge(&self) -> Origin {
        unsafe {
            let (mut width, mut height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut width, &mut height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x + height, y + height)
        }
    }
}
