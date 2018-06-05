use std::ptr;

use wlroots_sys::{wlr_backend, wl_display, wlr_wl_backend_create,
                  wlr_wl_output_create, wlr_input_device_is_wl, wlr_output_is_wl};

use {OutputHandle, InputDevice, Output};
use utils::safe_as_cstring;
use super::UnsafeRenderSetupFunction;

/// When the compositor is running in a nested Wayland environment.
/// e.g. your compositor is executed while the user is running Gnome+Mutter or Weston.
///
/// This is useful for testing and iterating on the design of the compositor.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct WaylandBackend {
    pub(crate) backend: *mut wlr_backend
}

impl WaylandBackend {
    /// Creates a new wlr_wl_backend. This backend will be created with no outputs;
    /// you must use wlr_wl_output_create to add them.
    ///
    /// The `remote` argument is the name of the host compositor wayland socket. Set
    /// to `None` for the default behaviour (WAYLAND_DISPLAY env variable or wayland-0
    /// default)
    pub unsafe fn new(display: *mut wl_display,
                      remote: Option<String>,
                      render_setup_func: Option<UnsafeRenderSetupFunction>)
                      -> Self {
        let remote_cstr = remote.map(|remote| safe_as_cstring(remote));
        let remote_ptr = remote_cstr.map(|s| s.as_ptr()).unwrap_or_else(|| ptr::null_mut());
        let backend = wlr_wl_backend_create(display, remote_ptr, render_setup_func);
        if backend.is_null() {
            panic!("Could not construct Wayland backend");
        }
        WaylandBackend { backend }
    }


    /// Adds a new output to this backend.
    ///
    /// You may remove outputs by destroying them.
    ///
    /// Note that if called before initializing the backend, this will return None
    /// and your outputs will be created during initialization (and given to you via
    /// the output_add signal).
    pub fn create_output(&self) -> Option<OutputHandle> {
        unsafe {
            let output_ptr = wlr_wl_output_create(self.backend);
            if output_ptr.is_null() {
                None
            } else {
                Some(OutputHandle::from_ptr(output_ptr))
            }

        }
    }

    /// True if the given input device is a wlr_wl_input_device.
    pub fn is_wl_input_device(&self, input_device: &InputDevice) -> bool {
        unsafe {
            wlr_input_device_is_wl(input_device.as_ptr())
        }
    }

    /// True if the given output is a wlr_wl_output.
    pub fn is_wl_output_device(&self, output: &Output) -> bool {
        unsafe {
            wlr_output_is_wl(output.as_ptr())
        }
    }
}
