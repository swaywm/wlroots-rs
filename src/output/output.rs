use wlroots_sys::wlr_output;

use std::fmt;

/// A wrapper around [wlr_output](../../../wlroots_sys/struct.wlr_output.html).
pub struct Output {
    pub inner: *mut wlr_output
}

impl Output {
    pub fn new(inner: *mut wlr_output) -> Self {
        Output { inner }
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Output: {:?}", self.inner)
    }
}

unsafe impl Send for Output {}
unsafe impl Sync for Output {}
