use wlroots_sys::wlr_output;

/// A wrapper around [wlr_output](../../../wlroots_sys/struct.wlr_output.html).
pub struct Output {
    pub inner: wlr_output
}

unsafe impl Send for Output {}
unsafe impl Sync for Output {}
