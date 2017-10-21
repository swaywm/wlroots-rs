//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use libc;
use output::Output;
use std::{mem, ptr};
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wl_list, wl_listener, wlr_output, wlr_output_mode, wlr_output_set_mode};


/// Handles output addition and removal.
pub trait OutputManagerHandler {
    /// Called whenever an output is added.
    fn output_added(&mut self, Output);
    /// Called whenever an output is removed.
    fn output_removed(&mut self, Output);
    /// Called every time the output frame is updated.
    fn output_frame(&mut self, Output);
    /// Called every time the output resolution is updated.
    fn output_resolution(&mut self, Output);
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

/// The default output handler that most compostiors can use as a drop-in.
pub struct DefaultOutputHandler {
    output: Output,
    last_frame: i32,
    link: wl_list,
    data: *mut libc::c_void
}

impl OutputManagerHandler for DefaultOutputHandler {
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
    fn output_removed(&mut self, output: Output) {
        // TODO
    }

    fn output_frame(&mut self, output: Output) {
        wlr_log!(L_DEBUG, "OUTPUT FRAME");
        // TODO
    }

    fn output_resolution(&mut self, output: Output) {
        wlr_log!(L_DEBUG, "OUTPUT RESOLUTION");
        // TODO
    }
}

impl DefaultOutputHandler {
    pub fn new() -> DefaultOutputHandler {
        unsafe {
            // NOTE Rationale for zero-ing memory:
            // FIXME There is no rational, that's just stupid
            let mut default_handler: DefaultOutputHandler = mem::zeroed();
            // FIXME This is very, very stupid
            ptr::write(&mut default_handler.output,
                       Output::from_ptr(ptr::null_mut()));
            default_handler
        }
    }
}
