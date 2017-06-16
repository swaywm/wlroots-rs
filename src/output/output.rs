use wlroots_sys::{wlr_output, wlr_output_enable};

use std::fmt;

/// A wrapper around [wlr_output](../../../wlroots_sys/struct.wlr_output.html).
pub struct Output {
    pub inner: *mut wlr_output
}

impl Output {
    pub fn new(inner: *mut wlr_output) -> Self {
        Output { inner }
    }

    pub fn disable(&mut self) {
        unsafe {
            wlr_output_enable(self.inner, false)
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            wlr_output_enable(self.inner, true)
        }
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Output: {:?}", self.inner)
    }
}

unsafe impl Send for Output {}
unsafe impl Sync for Output {}
