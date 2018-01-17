//! TODO Documentation

use wlroots_sys::{wlr_output_layout, wlr_output_layout_create, wlr_output_layout_destroy,
                  wlr_output_layout_remove};

use Output;

#[derive(Debug)]
pub struct OutputLayout {
    layout: *mut wlr_output_layout
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
        wlr_output_layout_remove(self.layout, output.as_ptr())
    }
}

impl Drop for OutputLayout {
    fn drop(&mut self) {
        unsafe { wlr_output_layout_destroy(self.layout) }
    }
}
