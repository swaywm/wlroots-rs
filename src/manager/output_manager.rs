//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use compositor::{Compositor, COMPOSITOR_PTR};
use libc;
use manager::{OutputHandler, UserOutput};
use types::Output;

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_output;

use std::panic;

/// Used to ensure the output sets the mode before doing any other
/// operation on the Output.
pub struct OutputBuilder<'output> {
    output: &'output mut Output
}

/// Used to ensure that the builder is used to construct
/// the OutputHandler instance.
pub struct OutputBuilderResult<'output> {
    pub output: &'output mut Output,
    result: Box<OutputHandler>
}

/// Wrapper around Output destruction so that you can't call
/// unsafe methods (e.g anything like setting the mode).
pub struct OutputDestruction<'output>(&'output mut Output);

/// Handles output addition and removal.
pub trait OutputManagerHandler {
    /// Called whenever an output is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn output_added<'output>(&mut self,
                             &mut Compositor,
                             _: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        None
    }

    /// Called whenever an output is removed.
    fn output_removed(&mut self, &mut Compositor, OutputDestruction) {
        // TODO
    }
}

impl<'output> OutputBuilder<'output> {
    pub fn build_best_mode<T: OutputHandler + 'static>(self,
                                                       data: T)
                                                       -> OutputBuilderResult<'output> {
        self.output.choose_best_mode();
        OutputBuilderResult { output: self.output,
                              result: Box::new(data) }
    }
}

impl<'output> OutputDestruction<'output> {
    // TODO Functions which are safe to use
}

wayland_listener!(OutputManager, (Vec<Box<UserOutput>>, Box<OutputManagerHandler>), [
    add_listener => add_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        let (ref mut outputs, ref mut manager) = this.data;
        let data = data as *mut wlr_output;
        let mut output = Output::new(data as *mut wlr_output);
        // NOTE
        // This clone is required because we pass it mutably to the output builder,
        // but due to lack of NLL there's no way to tell Rust it's safe to use it in
        // in the if branch.
        //
        // Thus, we need to clone it here and then drop the original once at the end.
        //
        // This is not a real clone, but an pub(crate) unsafe one we added, so it doesn't
        // break safety concerns in user code. Just an unfortunate hack we have to put here.
        let output_clone = output.clone();
        let builder = OutputBuilder { output: &mut output };
        let compositor = &mut *COMPOSITOR_PTR;
        output_clone.set_lock(true);
        let res = panic::catch_unwind(
            panic::AssertUnwindSafe(||manager.output_added(compositor, builder)));
        let build_result = match res {
            Ok(res) => res,
            // NOTE
            // Either Wayland or wlroots does not handle failure to set up output correctly.
            // Calling wl_display_terminate does not work if output is incorrectly set up.
            //
            // Instead, execution keeps going with an eventual segfault (if lucky).
            //
            // To fix this, we abort the process if there was a panic in output setup.
            Err(_) => ::std::process::abort()
        };
        if let Some(OutputBuilderResult {result: output_ptr, .. }) = build_result {
            output_clone.set_lock(false);
            let mut output = UserOutput::new((output_clone, output_ptr));
            // Add the output frame event to this manager
            wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                        output.frame_listener() as _);
            // Add the output resolution event to this manager
            wl_signal_add(&mut (*data).events.resolution as *mut _ as _,
                        output.resolution_listener() as _);
            // Store the user UserOutput, free later in remove listener
            outputs.push(output);
        }
    };
    remove_listener => remove_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        let (ref mut outputs, ref mut manager) = this.data;
        let data = data as *mut wlr_output;
        // NOTE
        // We get it from the list so that we can get the Rc'd `Output`, because there's
        // no way to re-construct that using just the raw pointer.
        if let Some(output) = outputs.iter_mut().find(|output| output.output_ptr() == data) {
            let output = output.output_mut();
            let compositor = &mut *COMPOSITOR_PTR;
            output.set_lock(true);
            manager.output_removed(compositor, OutputDestruction(output));
            // NOTE We don't remove the lock because we are removing it
            if let Some(layout) = output.layout() {
                layout.borrow_mut().remove(output);
            }
        }
        // Remove user output data
        if let Some(index) = outputs.iter().position(|output| output.output_ptr() == data) {
            let mut removed_output = outputs.remove(index);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_output.frame_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_output.resolution_listener()).link as *mut _ as _);

        }
    };
]);
