//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.


use libc;
use manager::{Output, OutputHandler};
use types::output;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_output;

/// Handles output addition and removal.
pub trait OutputManagerHandler {
    /// Called whenever an output is added.
    fn output_added(&mut self, _: &mut output::Output) -> Option<Box<OutputHandler>> {
        None
    }

    /// Called whenever an output is removed.
    fn output_removed(&mut self, &mut output::Output) {
        // TODO
    }
    /// Called every time the output frame is updated.
    fn output_frame(&mut self, &mut output::Output) {}
    /// Called every time the output resolution is updated.
    fn output_resolution(&mut self, &mut output::Output) {}
}

wayland_listener!(OutputManager, Box<OutputManagerHandler>, [
    add_listener => add_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        let data = data as *mut wlr_output;
        let mut output = output::Output::from_ptr(data as *mut wlr_output);
        if let Some(output) = this.data.output_added(&mut output) {
            let mut output = Output::new(output);
            // Add the output frame event to this manager
            wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                        output.frame_listener() as _);
            // Add the output resolution event to this manager
            wl_signal_add(&mut (*data).events.resolution as *mut _ as _,
                        output.resolution_listener() as _);
            ::std::mem::forget(output);
        }
    };
    remove_listener => remove_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        this.data.output_removed(&mut output::Output::from_ptr(data as *mut wlr_output))
    };
]);
