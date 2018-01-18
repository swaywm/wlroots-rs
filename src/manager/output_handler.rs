//! Handler for outputs

use compositor::{Compositor, COMPOSITOR_PTR};
use libc;
use types::Output;
use wlroots_sys::wlr_output;

pub trait OutputHandler {
    /// Called every time the output frame is updated.
    fn output_frame(&mut self, &mut Compositor, &mut Output) {}

    /// Called every time the output resolution changes.
    fn output_resolution(&mut self, &mut Output) {}
}

wayland_listener!(UserOutput, (Output, Box<OutputHandler>), [
    frame_listener => frame_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let output = &mut this.data.0;
        let manager = &mut this.data.1;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.output_frame(compositor, output);
        output.set_lock(false);
    };
    resolution_listener => resolution_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let output = &mut this.data.0;
        let manager = &mut this.data.1;
        output.set_lock(true);
        manager.output_resolution(output);
        output.set_lock(false);
    };
]);

impl UserOutput {
    pub(crate) fn output_mut(&mut self) -> &mut Output {
        &mut self.data.0
    }

    pub unsafe fn output_ptr(&self) -> *mut wlr_output {
        self.data.0.as_ptr()
    }
}
