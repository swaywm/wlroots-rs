use std::ptr;

use wlroots_sys::{wlr_backend, wlr_x11_backend_create, wlr_x11_output_create,
                  wlr_input_device_is_x11, wlr_output_is_x11, wl_display};

use {OutputHandle, InputDevice, Output};
use super::UnsafeRenderSetupFunction;
use utils::safe_as_cstring;

/// When the compositor is running in a nested X11 environment.
/// e.g. your compositor is executed while the user is running an X11 window manager.
///
/// This is useful for testing and iteration on the design of the compositor.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct X11Backend {
    pub(crate) backend: *mut wlr_backend
}

impl X11Backend {
    pub unsafe fn new(display: *mut wl_display,
                      x11_display: Option<String>,
                      render_setup_func: Option<UnsafeRenderSetupFunction>)
                      -> Self {
        let x11_display_cstr = x11_display.map(|remote| safe_as_cstring(remote));
        let x11_display_ptr = x11_display_cstr.map(|s| s.as_ptr()).unwrap_or_else(|| ptr::null_mut());
        let backend = wlr_x11_backend_create(display, x11_display_ptr, render_setup_func);
        if backend.is_null() {
            panic!("Could not construct X11 backend");
        }
        X11Backend { backend }
    }

    pub fn create_output(&self) -> Option<OutputHandle> {
        unsafe {
            let output_ptr = wlr_x11_output_create(self.backend);
            if output_ptr.is_null() {
                None
            } else {
                Some(OutputHandle::from_ptr(output_ptr))
            }

        }
    }

    pub fn is_x11_input_device(&self, input_device: &InputDevice) -> bool {
        unsafe {
            wlr_input_device_is_x11(input_device.as_ptr())
        }
    }

    pub fn is_x11_output_device(&self, output: &Output) -> bool {
        unsafe {
            wlr_output_is_x11(output.as_ptr())
        }
    }
}
