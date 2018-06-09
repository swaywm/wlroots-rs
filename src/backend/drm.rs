use std::ptr;

use libc;
use wlroots_sys::{wlr_backend, wl_display, wlr_drm_backend_create, wlr_output_is_drm,
                  wlr_drm_backend_get_session};

use Output;
use super::{UnsafeRenderSetupFunction, Session};

/// When the compositor is ran on a TTY and has full control of the system resources.
///
/// This is primarily the backend that end users will use, as they usually want the
/// compositor to run standalone.
///
/// Note that because you have full control of the TTY (and the keyboard, the mouse, and
/// just about everything else) that if there's an infinite loop then you could hard-lock
/// yourself out of the system. At that point you must reboot your computer (or use
/// SysRq).
///
/// Note that if the process exits for any reason (a panic, an abort, or a clean exit)
/// all of the resource handles will automatically be cleaned up properly by the OS.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct DRMBackend {
    pub(crate) backend: *mut wlr_backend
}

impl DRMBackend {
     /// Creates a DRM backend using the specified GPU file descriptor (typically from
     /// a device node in /dev/dri).
     ///
     /// To slave this to another DRM backend, pass it as the parent (which _must_ be
     /// a DRM backend, other kinds of backends raise SIGABRT).
    pub unsafe fn new(display: *mut wl_display,
                      session: Session,
                      gpu_fd: libc::c_int,
                      parent: Option<DRMBackend>,
                      render_setup_func: Option<UnsafeRenderSetupFunction>)
                      -> Self {
        let parent_ptr = parent.map(|backend| backend.as_ptr()).unwrap_or_else(|| ptr::null_mut());
        let backend = wlr_drm_backend_create(display,
                                             session.as_ptr(),
                                             gpu_fd,
                                             parent_ptr,
                                             render_setup_func);
        if backend.is_null() {
            panic!("Could not construct X11 backend");
        }
        DRMBackend { backend }
    }

    pub fn output_is_drm(&self, output: &Output) -> bool {
        unsafe {
            wlr_output_is_drm(output.as_ptr())
        }
    }

    pub fn session(&self) -> Session {
        unsafe {
            let session_ptr = wlr_drm_backend_get_session(self.backend);
            if session_ptr.is_null() {
                panic!("Session was null");
            }
            Session::from_ptr(session_ptr)
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut wlr_backend {
        self.backend
    }
}
