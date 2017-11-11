use wlroots_sys::{wlr_input_device, wlr_pointer};

/// A wlr_input_device that is guaranteed to be a pointer.
pub struct PointerHandle {
    /// The device that refers to this pointer
    device: *mut wlr_input_device,
    /// The underlying pointer data
    pointer: *mut wlr_pointer
}

impl PointerHandle {
    /// Tries to convert an input device to a pointer
    ///
    /// Returns none if it is of a different input varient.
    pub unsafe fn from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_POINTER => {
                let pointer = (*device).__bindgen_anon_1.pointer;
                Some(PointerHandle { device, pointer })
            }
            _ => None,
        }
    }

    /// Gets the wlr_input_device associated with this Pointer
    pub unsafe fn input_device(&self) -> *mut wlr_input_device {
        self.device
    }

    pub unsafe fn pointer(&self) -> *mut wlr_pointer {
        self.pointer
    }
}
