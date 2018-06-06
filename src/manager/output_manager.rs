//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use {Output, OutputHandle, OutputState};
use compositor::{compositor_handle, CompositorHandle};
use libc;
use manager::{OutputHandler, UserOutput};

use std::marker::PhantomData;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_output;

use std::panic;

/// Used to ensure the output sets the mode before doing any other
/// operation on the Output.
pub struct OutputBuilder<'output> {
    output: OutputHandle,
    phantom: PhantomData<&'output Output>
}

/// Used to ensure that the builder is used to construct
/// the OutputHandler instance.
pub struct OutputBuilderResult<'output> {
    pub output: OutputHandle,
    result: Box<OutputHandler>,
    phantom: PhantomData<&'output Output>
}

/// Wrapper around Output destruction so that you can't call
/// unsafe methods (e.g anything like setting the mode).
pub struct OutputDestruction(OutputHandle);

/// Handles output addition and removal.
pub trait OutputManagerHandler {
    /// Called whenever an output is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn output_added<'output>(&mut self,
                             CompositorHandle,
                             _: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        None
    }

    /// Called whenever an output is removed.
    fn output_removed(&mut self, CompositorHandle, OutputDestruction) {
        // TODO
    }
}

impl<'output> OutputBuilder<'output> {
    /// Get a handle to the output this is building.
    ///
    /// This is so you can use this output later.
    pub fn handle(&self) -> OutputHandle {
        self.output.clone()
    }

    /// Build the output with the best mode.
    ///
    /// To complete construction, return this in your implementation of
    /// `OutputManagerHandler::output_added`.
    pub fn build_best_mode<T: OutputHandler + 'static>(mut self,
                                                       data: T)
                                                       -> OutputBuilderResult<'output> {
        with_handles!([(output: {&mut self.output})] => {
            output.choose_best_mode();
        }).expect("Output was borrowed");
        OutputBuilderResult { output: self.output,
                              result: Box::new(data),
                              phantom: PhantomData }
    }
}

impl OutputDestruction {
    // TODO Functions which are safe to use
}

wayland_listener!(OutputManager, Box<OutputManagerHandler>, [
    add_listener => add_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        let ref mut manager = this.data;
        let data = data as *mut wlr_output;
        let output = Output::new(data as *mut wlr_output);
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
        let builder = OutputBuilder { output: output.weak_reference(), phantom: PhantomData };
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
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
            let mut output = UserOutput::new((output_clone, output_ptr));
            wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                          output.frame_listener() as _);
            wl_signal_add(&mut (*data).events.mode as *mut _ as _,
                          output.mode_listener() as _);
            wl_signal_add(&mut (*data).events.enable as *mut _ as _,
                          output.enable_listener() as _);
            wl_signal_add(&mut (*data).events.scale as *mut _ as _,
                          output.scale_listener() as _);
            wl_signal_add(&mut (*data).events.transform as *mut _ as _,
                          output.transform_listener() as _);
            wl_signal_add(&mut (*data).events.swap_buffers as *mut _ as _,
                          output.swap_buffers_listener() as _);
            wl_signal_add(&mut (*data).events.needs_swap as *mut _ as _,
                          output.need_swap_listener() as _);
            wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                          output.on_destroy_listener() as _);
            let output_data = (*data).data as *mut OutputState;
            (*output_data).output = Box::into_raw(output);
        }
    };
]);
