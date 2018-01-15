use wlroots_sys::{wlr_input_device, wlr_input_device_pointer, wlr_input_device_type};

/// Wrapper for wlr_input_device
#[derive(Debug)]
pub struct InputDevice {
    device: *mut wlr_input_device
}

impl InputDevice {
    /// Just like `std::clone::Clone`, but unsafe.
    ///
    /// # Unsafety
    /// This is unsafe because the user should not be able to clone
    /// this type out because it isn't bound by anything but the underlying
    /// pointer could be removed at any time.
    ///
    /// This isn't exposed to the user, but still marked as `unsafe` to reduce
    /// possible bugs from using this.
    pub(crate) unsafe fn clone(&self) -> Self {
        InputDevice { device: self.device }
    }

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
