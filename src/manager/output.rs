//! Handler for outputs

use libc;
use output;
use wlroots_sys::wlr_output;

pub trait OutputHandler {
    /// Called every time the output frame is updated.
    fn output_frame(&mut self, &mut output::Output) {
        // TODO
    }

    /// Called every time the output resolution changes.
    fn output_resolution(&mut self, &mut output::Output) {
        // TODO
    }
}

wayland_listener!(Output, Box<OutputHandler>, [
    frame_listener => frame_notify: |this: &mut Output, data: *mut libc::c_void,| unsafe {
        // TODO Ensure safety
        this.data.output_frame(&mut output::Output::from_ptr(data as *mut wlr_output))
    };
    resolution_listener => resolution_notify: |this: &mut Output, data: *mut libc::c_void,| unsafe {
        // TODO Ensure safety
        this.data.output_resolution(&mut output::Output::from_ptr(data as *mut wlr_output))
    };
]);
