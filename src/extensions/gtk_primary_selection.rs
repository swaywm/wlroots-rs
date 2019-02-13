//! Support for the wlroots Gamma Control Protocol

use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wl_display, wlr_gtk_primary_selection_device_manager, wlr_gtk_primary_selection_device_manager_create, 
    wlr_gtk_primary_selection_device_manager_destroy};

#[derive(Debug)]
/// Manager that implements GTK primary selection
pub struct Manager {
    manager: *mut wlr_gtk_primary_selection_device_manager
}

impl Manager {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_gtk_primary_selection_device_manager_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(Manager { manager: manager_raw })
        } else {
            None
        }
    }

}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe { wlr_gtk_primary_selection_device_manager_destroy(self.manager) }
    }
}
