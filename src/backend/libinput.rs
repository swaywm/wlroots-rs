use wlroots_sys::{wlr_backend, wl_display, wlr_libinput_backend_create, libinput_device,
                  wlr_libinput_get_device_handle, wlr_input_device_is_libinput};

use {backend::session::Session,
     input::InputDevice};

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct LibInputBackend {
    pub(crate) backend: *mut wlr_backend
}

impl LibInputBackend {
    pub unsafe fn new(display: *mut wl_display, session: Session) -> Self {
        let backend = wlr_libinput_backend_create(display, session.as_ptr());
        if backend.is_null() {
            panic!("Could not construct Wayland backend");
        }
        LibInputBackend { backend }
    }

    /// Get the underlying libinput_device handle for the given input device.
    pub unsafe fn device_handle(input_device: &InputDevice) -> *mut libinput_device {
        wlr_libinput_get_device_handle(input_device.as_ptr())
    }

    pub fn is_libinput_input_device(&self, input_device: &InputDevice) -> bool {
        unsafe { wlr_input_device_is_libinput(input_device.as_ptr()) }
    }
}
