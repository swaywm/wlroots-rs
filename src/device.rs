use wlroots_sys::{wlr_input_device, wlr_input_device_events, wlr_input_device_type};

/// Wrapper for wlr_input_device
#[derive(Debug)]
pub struct Device {
    device: *mut wlr_input_device
}

impl Device {
    /// Get the type of the device
    pub fn dev_type(&self) -> wlr_input_device_type {
        unsafe { (*self.device).type_ }
    }

    // TODO Wrapper around the union
    pub unsafe fn dev_union(&self) -> wlr_input_device_events {
        (*self.device).__bindgen_anon_1
    }

    pub unsafe fn from_ptr(device: *mut wlr_input_device) -> Self {
        Device { device }
    }

    pub unsafe fn to_ptr(&self) -> *mut wlr_input_device {
        self.device
    }
}
