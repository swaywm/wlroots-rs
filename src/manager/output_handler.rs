//! Handler for outputs

use Output;
use compositor::{Compositor, COMPOSITOR_PTR};
use libc;
use wlroots_sys::wlr_output;

pub trait OutputHandler {
    /// Called every time the output frame is updated.
    fn on_frame(&mut self, &mut Compositor, &mut Output) {}

    /// Called every time the output mode changes.
    fn on_mode_change(&mut self, &mut Compositor, &mut Output) {}

    /// Called every time the output is enabled.
    fn on_enable(&mut self, &mut Compositor, &mut Output) {}

    /// Called every time the output scale changes.
    fn on_scale_change(&mut self, &mut Compositor, &mut Output) {}

    /// Called every time the output transforms.
    fn on_transform(&mut self, &mut Compositor, &mut Output) {}

    /// Called every time the buffers are swapped on an output.
    fn on_buffers_swapped(&mut self, &mut Compositor, &mut Output) {}
}

wayland_listener!(UserOutput, (Output, Box<OutputHandler>), [
    frame_listener => frame_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref mut output, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.on_frame(compositor, output);
        output.set_lock(false);
    };
    mode_listener => mode_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref mut output, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.on_mode_change(compositor, output);
        output.set_lock(false);
    };
    enable_listener => enable_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref mut output, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.on_enable(compositor, output);
        output.set_lock(false);
    };
    scale_listener => scale_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref mut output, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.on_scale_change(compositor, output);
        output.set_lock(false);
    };
    transform_listener => transform_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref mut output, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.on_transform(compositor, output);
        output.set_lock(false);
    };
    swap_buffers_listener => swap_buffers_notify: |this: &mut UserOutput,
                                                   _output: *mut libc::c_void,| unsafe {
        let (ref mut output, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        output.set_lock(true);
        manager.on_buffers_swapped(compositor, output);
        output.set_lock(false);
    };
]);

impl UserOutput {
    pub(crate) fn output_mut(&mut self) -> &mut Output {
        &mut self.data.0
    }

    pub(crate) unsafe fn output_ptr(&self) -> *mut wlr_output {
        self.data.0.as_ptr()
    }
}
