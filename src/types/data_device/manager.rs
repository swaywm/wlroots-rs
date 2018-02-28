//! TODO Documentation

use wlroots_sys::{wl_display, wlr_data_device_manager, wlr_data_device_manager_create,
                  wlr_data_device_manager_destroy};

/// Global for the data device manager global for a certain display.
#[derive(Debug)]
pub struct DataDeviceManager {
    manager: *mut wlr_data_device_manager
}

impl DataDeviceManager {
    /// Create a wl data device manager global for this display.
    pub(crate) unsafe fn new(display: *mut wl_display) -> Option<Self> {
        let manager = wlr_data_device_manager_create(display);
        if manager.is_null() {
            None
        } else {
            Some(DataDeviceManager { manager })
        }
    }
}

impl Drop for DataDeviceManager {
    fn drop(&mut self) {
        unsafe { wlr_data_device_manager_destroy(self.manager) }
    }
}
