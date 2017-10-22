use wlroots_sys::{wlr_input_device, wlr_input_device__bindgen_ty_1, wlr_input_device_type};

/// Wrapper for wlr_input_device
#[derive(Debug)]
pub struct Device {
    device: *mut wlr_input_device
}

// TODO We are assuming the device is live in these functions,
// but we need some way to ensure that.
// E.g we need to control access to the "Device",
// probably only in certain methods.

impl Device {
    /// Get the type of the device
    pub fn dev_type(&self) -> wlr_input_device_type {
        unsafe { (*self.device).type_ }
    }

    // TODO Fix name
    // TODO Wrapper around the union
    pub unsafe fn dev_union(&self) -> wlr_input_device__bindgen_ty_1 {
        (*self.device).__bindgen_anon_1
    }

    pub unsafe fn from_ptr(device: *mut wlr_input_device) -> Self {
        Device { device }
    }

    pub unsafe fn to_ptr(&self) -> *mut wlr_input_device {
        self.device
    }
}
