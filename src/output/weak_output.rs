use std::rc;

use wlroots_sys::wlr_output;

/// Weak reference to an [wlr_output](../../../wlroots_sys/struct.wlr_output.html).
#[derive(Clone)]
pub struct WeakOutput {
    output_ref: rc::Weak<wlr_output>
}

