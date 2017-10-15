//! Manager that is called when an output is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during initialization.

use super::io_manager::IOManager;
use wlroots_sys::{wlr_output, wl_listener};
use libc;
use std::ops::{Deref, DerefMut};


/// Handles output addition and removal.
pub trait OutputManagerHandler {
    // TODO Wrapper for wlr_output
    /// Called whenever an output is added.
    fn output_added(&mut self, *mut wlr_output);
    // TODO Wrapper for wlr_output
    /// Called whenever an output is removed.
    fn output_removed(&mut self, *mut wlr_output);
}

#[repr(C)]
/// Holds the user-defined output manager.
/// Pass this to the `Compositor` during initialization.
pub struct OutputManager(IOManager<Box<OutputManagerHandler>>);

impl OutputManager {
    pub fn new(output_manager: Box<OutputManagerHandler>) -> Self {
        OutputManager(IOManager::new(output_manager,
                                     Self::output_add_notify,
                                     Self::output_remove_notify))
    }

    unsafe extern "C" fn output_add_notify(listener: *mut wl_listener,
                                          data: *mut libc::c_void) {
        let device = data as *mut wlr_output;
        let output_wrapper = container_of!(listener, OutputManager, add_listener);
        let output_manager = &mut (*output_wrapper).manager;
        output_manager.output_added(device)
    }

    unsafe extern "C" fn output_remove_notify(listener: *mut wl_listener,
                                             data: *mut libc::c_void) {
        let device = data as *mut wlr_output;
        let output_wrapper = container_of!(listener, OutputManager, remove_listener);
        let output_manager = &mut (*output_wrapper).manager;
        output_manager.output_removed(device)
    }

}

impl Deref for OutputManager {
    type Target = IOManager<Box<OutputManagerHandler>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OutputManager {
    fn deref_mut(&mut self) -> &mut IOManager<Box<OutputManagerHandler>> {
        &mut self.0
    }
}
