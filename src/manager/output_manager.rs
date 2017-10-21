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
}

define_listener!(OutputManager, Box<OutputManagerHandler>, [
    add_listener, add_notify: |output_manager: &mut Box<OutputManagerHandler>, data: *mut libc::c_void,| unsafe {
        output_manager.output_added(Output::from_ptr(data as *mut wlr_output))
    };
    remove_listener, remove_notify: |output_manager: &mut Box<OutputManagerHandler>, data: *mut libc::c_void,| unsafe {
        output_manager.output_removed(Output::from_ptr(data as *mut wlr_output))
    };
]);

/// The default output handler that most compostiors can use as a drop-in.
pub struct DefaultOutputHandler {
    output: Output,
    frame: wl_listener,
    resolution: wl_listener,
    last_frame: i32,
    link: wl_list,
    data: *mut libc::c_void
}

impl OutputManagerHandler for DefaultOutputHandler {
    fn output_added(&mut self, output: Output) {
        // TODO Logging using macro
        unsafe {
            if (*output.modes()).length > 0 {
                let first_mode_ptr = (*output.modes()).items.offset(0) as *mut wlr_output_mode;
                wlr_output_set_mode(output.to_ptr(), first_mode_ptr);
            }
            let mut frame_event = output.events().frame;
            let mut resolution_event = output.events().resolution;
            // NOTE We are moving output here, but the pointers
            // to the events are fine because Output doesn't own wlr_output.
            self.output = output;
            // TODO FIXME Punting, somehow we need to reference compositor...pass it in?
            // That _should_ be possible
            // self.compositor = ....
            ptr::write(&mut self.frame.notify, Some(Self::frame_notify));
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_init,
                          &mut self.frame.link as *mut _ as _);
            wl_signal_add(&mut frame_event as *mut _ as _,
                          &mut self.frame as *mut _ as _);
            ptr::write(&mut self.resolution.notify, Some(Self::resolution_notify));
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_init,
                          &mut self.resolution.link as *mut _ as _);
            wl_signal_add(&mut resolution_event as *mut _ as _,
                          &mut self.resolution as *mut _ as _);
            // TODO Add this output to some list of outputs
            // probably in passed in compositor state?

            // TODO Call a user defined callback?
            // I think we can do that by overriding the impl on this?
            // not sure, I'd have to double check...probably not
        }
    }
    fn output_removed(&mut self, output: Output) {
        // TODO
    }
}

impl DefaultOutputHandler {
    // TODO Should be able to define this safely, do the same thing as with
    // output_added
    unsafe extern "C" fn frame_notify(listener: *mut wl_listener, data: *mut libc::c_void) {
        // TODO implement
    }
    unsafe extern "C" fn resolution_notify(listener: *mut wl_listener, data: *mut libc::c_void) {
        // TODO implement
        // TODO call callbmack
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
