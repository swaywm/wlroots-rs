//! Handler for outputs

use {Output, OutputHandle};
use compositor::{compositor_handle, CompositorHandle};
use libc;
use wlroots_sys::wlr_output;

pub trait OutputHandler {
    /// Called every time the output frame is updated.
    fn on_frame(&mut self, CompositorHandle, OutputHandle) {}

    /// Called every time the output mode changes.
    fn on_mode_change(&mut self, CompositorHandle, OutputHandle) {}

    /// Called every time the output is enabled.
    fn on_enable(&mut self, CompositorHandle, OutputHandle) {}

    /// Called every time the output scale changes.
    fn on_scale_change(&mut self, CompositorHandle, OutputHandle) {}

    /// Called every time the output transforms.
    fn on_transform(&mut self, CompositorHandle, OutputHandle) {}

    /// Called every time the buffers are swapped on an output.
    fn on_buffers_swapped(&mut self, CompositorHandle, OutputHandle) {}

    /// Called every time the buffers need to be swapped on an output.
    fn needs_swap(&mut self, CompositorHandle, OutputHandle) {}
}

wayland_listener!(UserOutput, (Output, Box<OutputHandler>), [
    frame_listener => frame_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_frame(compositor, output.weak_reference());
    };
    mode_listener => mode_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_mode_change(compositor, output.weak_reference());
    };
    enable_listener => enable_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_enable(compositor, output.weak_reference());
    };
    scale_listener => scale_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_scale_change(compositor, output.weak_reference());
    };
    transform_listener => transform_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_transform(compositor, output.weak_reference());
    };
    swap_buffers_listener => swap_buffers_notify: |this: &mut UserOutput,
                                                   _output: *mut libc::c_void,|
    unsafe {

        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_buffers_swapped(compositor, output.weak_reference());
    };
    need_swap_listener => need_swap_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.needs_swap(compositor, output.weak_reference());
    };
]);

impl UserOutput {
    pub(crate) fn output_mut(&mut self) -> OutputHandle {
        self.data.0.weak_reference()
    }

    pub(crate) unsafe fn output_ptr(&self) -> *mut wlr_output {
        self.data.0.as_ptr()
    }
}
