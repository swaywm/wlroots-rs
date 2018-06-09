use wlroots_sys::{wlr_backend, wlr_backend_autocreate, wl_display, wlr_multi_backend_add,
                  wlr_multi_backend_remove, wlr_multi_is_empty, wlr_multi_get_session};

use super::{Session, UnsafeRenderSetupFunction};

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

    /// Adds the given backend to the multi backend.
    ///
    /// # Safety
    ///
    /// This should be done before the new backend is started.
    pub unsafe fn add_backend(&self, new_backend: *mut wlr_backend) -> bool {
        wlr_multi_backend_add(self.backend, new_backend)
    }

    /// Removes the backend.
    ///
    /// # Safety
    ///
    /// Doesn't check if that backend is valid.
    pub unsafe fn remove_backend(&self, backend: *mut wlr_backend) {
        wlr_multi_backend_remove(self.backend, backend)
    }

    pub fn wlr_multi_get_session(&self) -> Option<Session> {
        unsafe {
            let session = wlr_multi_get_session(self.backend);
            if session.is_null() {
                None
            } else {
                Some(Session::from_ptr(session))
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            wlr_multi_is_empty(self.backend)
        }
    }
}
