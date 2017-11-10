//! Handler for outputs

use libc;
use types::output;
use wlroots_sys::wlr_output;

pub trait OutputHandler {
    /// Called every time the output frame is updated.
    fn output_frame(&mut self, &mut output::OutputHandle) {}

    /// Called every time the output resolution changes.
    fn output_resolution(&mut self, &mut output::OutputHandle) {}
}

wayland_listener!(UserOutput, (*mut wlr_output, Box<OutputHandler>), [
    frame_listener => frame_notify: |this: &mut UserOutput, data: *mut libc::c_void,| unsafe {
        let manager = &mut this.data.1;
        manager.output_frame(&mut output::OutputHandle::from_ptr(data as *mut wlr_output))
    };
    resolution_listener => resolution_notify: |this: &mut UserOutput, data: *mut libc::c_void,|
    unsafe {
        let manager = &mut this.data.1;
        manager.output_resolution(&mut output::OutputHandle::from_ptr(data as *mut wlr_output))
    };
]);

impl UserOutput {
    pub unsafe fn output_ptr(&self) -> *mut wlr_output {
        self.data.0
    }
}
