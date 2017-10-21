//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use libc;
use output::Output;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_output, wlr_output_mode, wlr_output_set_mode};


/// Handles output addition and removal.
pub trait OutputManagerHandler {
    /// Called whenever an output is added.
    fn output_added(&mut self, output: Output) {
        wlr_log!(L_DEBUG, "output added {:?}", output);
        // TODO Shouldn't require unsafety here
        unsafe {
            if (*output.modes()).length > 0 {
                let first_mode_ptr = (*output.modes()).items.offset(0) as *mut wlr_output_mode;
                wlr_output_set_mode(output.to_ptr(), first_mode_ptr);
            }
        }
    }

    /// Called whenever an output is removed.
    fn output_removed(&mut self, Output) {
        // TODO
    }
    /// Called every time the output frame is updated.
    fn output_frame(&mut self, Output) {
        // TODO
    }
    /// Called every time the output resolution is updated.
    fn output_resolution(&mut self, Output) {
        // TODO
    }
}

wayland_listener!(OutputManager, Box<OutputManagerHandler>, [
    add_listener => add_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        let data = data as *mut wlr_output;
        // Add the output frame event to this manager
        wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                      this.frame_listener() as _);
        // Add the output resolution event to this manager
        wl_signal_add(&mut (*data).events.resolution as *mut _ as _,
                      this.resolution_listener() as _);
        // TODO Ensure safety
        this.data.output_added(Output::from_ptr(data as *mut wlr_output))
    };
    remove_listener => remove_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        // TODO Ensure safety
        this.data.output_removed(Output::from_ptr(data as *mut wlr_output))
    };
    frame_listener => frame_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        // TODO Ensure safety
        this.data.output_frame(Output::from_ptr(data as *mut wlr_output))
    };
    resolution_listener => resolution_notify: |this: &mut OutputManager, data: *mut libc::c_void,| unsafe {
        // TODO Ensure safety
        this.data.output_resolution(Output::from_ptr(data as *mut wlr_output))
    };
]);
