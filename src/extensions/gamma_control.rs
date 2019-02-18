//! Support for the wlroots Gamma Control Protocol

use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wl_display, wlr_gamma_control_manager_v1, wlr_gamma_control_manager_v1_create, 
    wlr_gamma_control_manager_v1_destroy};

#[derive(Debug)]
/// Manager that can adjust gamma controls for an output
pub struct Manager {
    manager: *mut wlr_gamma_control_manager_v1
}

impl Manager {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_gamma_control_manager_v1_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(Manager { manager: manager_raw })
        } else {
            None
        }
    }

}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe { wlr_gamma_control_manager_v1_destroy(self.manager) }
    }
}
