//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during initialization.

use super::io_manager::IOManager;
use wlroots_sys::{wlr_input_device, wl_listener};
use libc;
use std::ops::{Deref, DerefMut};

pub trait InputManagerHandler {
    // TODO Wrapper for wlr_input_device
    fn input_added(&mut self, *mut wlr_input_device);
    // TODO Wrapper for wlr_input_device
    fn input_removed(&mut self, *mut wlr_input_device);
}

#[repr(C)]
pub struct InputManager(IOManager<Box<InputManagerHandler>>);

impl InputManager {
    pub fn new(input_manager: Box<InputManagerHandler>) -> Self {
        InputManager(IOManager::new(input_manager,
                       InputManager::input_add_notify,
                       InputManager::input_remove_notify))
    }

    unsafe extern "C" fn input_add_notify(listener: *mut wl_listener,
                                          data: *mut libc::c_void) {
        let device = data as *mut wlr_input_device;
        let input_wrapper = container_of!(listener, InputManager, add_listener);
        let input_manager = &mut (*input_wrapper).manager;
        input_manager.input_added(device)
    }

    unsafe extern "C" fn input_remove_notify(listener: *mut wl_listener,
                                             data: *mut libc::c_void) {
        let device = data as *mut wlr_input_device;
        let input_wrapper = container_of!(listener, InputManager, remove_listener);
        let input_manager = &mut (*input_wrapper).manager;
        input_manager.input_removed(device)
    }

}

impl Deref for InputManager {
    type Target = IOManager<Box<InputManagerHandler>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InputManager {
    fn deref_mut(&mut self) -> &mut IOManager<Box<InputManagerHandler>> {
        &mut self.0
    }
}
