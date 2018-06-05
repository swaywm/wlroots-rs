use wlroots_sys::{wlr_backend, wlr_backend_autocreate, wl_display};

use super::UnsafeRenderSetupFunction;

/// When multiple backends are running or when the compositor writer doesn't care and
/// just used the auto create option in the `CompositorBuilder`.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct MultiBackend {
    pub(crate) backend: *mut wlr_backend
}

impl MultiBackend {
    /// Auto create a backend based on the environment.
    pub unsafe fn auto_create(display: *mut wl_display,
                              render_setup_func: Option<UnsafeRenderSetupFunction>)
                              -> Self {
        let backend = wlr_backend_autocreate(display, render_setup_func);
        if backend.is_null() {
            panic!("Could not auto construct backend");
        }
        MultiBackend { backend }
    }
}
