use wlroots_sys::{wlr_input_device, wlr_input_device_pointer, wlr_input_device_type};

/// Wrapper for wlr_input_device
#[derive(Debug)]
pub struct InputDevice {
    device: *mut wlr_input_device
}

impl InputDevice {
    /// Get the type of the device
    pub fn dev_type(&self) -> wlr_input_device_type {
        unsafe { (*self.device).type_ }
    }

    // TODO Wrapper around the union
    pub unsafe fn dev_union(&self) -> wlr_input_device_pointer {
        (*self.device).__bindgen_anon_1
    }

    pub unsafe fn from_ptr(device: *mut wlr_input_device) -> Self {
        InputDevice { device: device }
    }

    pub unsafe fn as_ptr(&self) -> *mut wlr_input_device {
        self.device
    }
}
