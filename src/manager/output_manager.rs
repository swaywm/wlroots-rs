//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use {Output, OutputHandle};
use compositor::{Compositor, COMPOSITOR_PTR};
use errors::HandleErr;
use libc;
use manager::{OutputHandler, UserOutput};

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
    /// Get a handle to the output this is building.
    ///
    /// This is so you can use this output later.
    pub fn handle(&self) -> OutputHandle {
        self.output.weak_reference()
    }

    /// Build the output with the best mode.
    ///
    /// To complete construction, return this in your implementation of
    /// `OutputManagerHandler::output_added`.
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
        let remove_listener = this.remove_listener()  as *mut _ as _;
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
        compositor.lock.set(true);
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
        compositor.lock.set(false);
        if let Some(OutputBuilderResult {result: output_ptr, .. }) = build_result {
            output_clone.set_lock(false);
            let mut output = UserOutput::new((output_clone, output_ptr));
            // Add the output frame event to this manager
            wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                          output.frame_listener() as _);
            // Add the output mode event to this manager
            wl_signal_add(&mut (*data).events.mode as *mut _ as _,
                          output.mode_listener() as _);
            // Add the output enable event to this manager
            wl_signal_add(&mut (*data).events.enable as *mut _ as _,
                          output.enable_listener() as _);
            // Add the output scale change event to this manager
            wl_signal_add(&mut (*data).events.scale as *mut _ as _,
                          output.scale_listener() as _);
            // Add the output transform event to this manager
            wl_signal_add(&mut (*data).events.transform as *mut _ as _,
                          output.transform_listener() as _);
            // Add the output buffer swap event to this manager
            wl_signal_add(&mut (*data).events.swap_buffers as *mut _ as _,
                          output.swap_buffers_listener() as _);
            // Add the output need swap event to this manager
            wl_signal_add(&mut (*data).events.needs_swap as *mut _ as _,
                          output.need_swap_listener() as _);
            // Add the output destroy event to this manager
            wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                          remove_listener);
            // Store the user UserOutput, free later in remove listener
            outputs.push(output);
        }
    };
    remove_listener => remove_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        let (ref mut outputs, ref mut manager) = this.data;
        let data = data as *mut wlr_output;
        if COMPOSITOR_PTR.is_null() {
            // We are shutting down, do nothing.
            return;
        }
        let compositor = &mut *COMPOSITOR_PTR;
        compositor.lock.set(true);
        // NOTE
        // We get it from the list so that we can get the Rc'd `Output`, because there's
        // no way to re-construct that using just the raw pointer.
        if let Some(output) = outputs.iter_mut().find(|output| output.output_ptr() == data) {
            let output = output.output_mut();
            output.set_lock(true);
            manager.output_removed(compositor, OutputDestruction(output));
            // NOTE We don't remove the lock because we are removing it
            if let Some(mut layout) = output.layout() {
                match layout.run(|layout| layout.remove(output)) {
                    Ok(_) | Err(HandleErr::AlreadyDropped) => {},
                    Err(HandleErr::AlreadyBorrowed) => {
                        compositor.lock.set(false);
                        panic!("Tried to remove layout from output, but it's already borrowed");
                    }
                }
            }
        }
        compositor.lock.set(false);
        // Remove user output data
        if let Some(index) = outputs.iter().position(|output| output.output_ptr() == data) {
            let mut removed_output = outputs.remove(index);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_output.frame_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_output.mode_listener()).link as *mut _ as _);
            // TODO Remove the rest of them
        }
    };
]);
