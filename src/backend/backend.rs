//! A backend contains information about how clients connected and rendered.
//!
//! In nested environments such as Wayland and X11 the compositor is a client to
//! the hosh process.
//!
//! In the headless backend clients aren't rendered and the only OS resources
//! used are the Wayland file descriptor. This is useful for testing.
//!
//! On the DRM backend the compositor controls an entire TTY. This is the most
//! invasive, but also the most useful backend. If there's an infinite loop here
//! then it is very easy to get into a place where the only course of action is
//! restarting the computer.
//!
//! On the multi backend multiple backends could be running at the same time.

use crate::libc;
use wlroots_sys::{
    self, wlr_backend, wlr_backend_get_session, wlr_backend_is_drm, wlr_backend_is_headless,
    wlr_backend_is_libinput, wlr_backend_is_multi, wlr_backend_is_wl, wlr_backend_is_x11
};

use crate::backend;

/// A custom function to set up the renderer.
pub type UnsafeRenderSetupFunction = unsafe extern "C" fn(
    egl: *mut wlroots_sys::wlr_egl,
    platform: u32,
    remote_display: *mut libc::c_void,
    config_attribs: *mut i32,
    visual_id: i32
) -> *mut wlroots_sys::wlr_renderer;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Backend {
    Wayland(backend::Wayland),
    X11(backend::X11),
    DRM(backend::Drm),
    Headless(backend::Headless),
    LibInput(backend::Libinput),
    Multi(backend::Multi)
}

impl Backend {
    /// Obtains the wlr_session reference from this backend if there is any.
    ///
    /// Might return None for backends that don't use a session.
    pub fn get_session<'backend>(&'backend mut self) -> Option<backend::Session<'backend>> {
        unsafe {
            let session_ptr = wlr_backend_get_session(self.as_ptr());
            if session_ptr.is_null() {
                None
            } else {
                Some(backend::Session::from_ptr(session_ptr))
            }
        }
    }

    /// Create a backend from a `*mut wlr_backend`.
    pub unsafe fn from_backend(backend: *mut wlr_backend) -> Self {
        if wlr_backend_is_wl(backend) {
            Backend::Wayland(backend::Wayland { backend })
        } else if wlr_backend_is_x11(backend) {
            Backend::X11(backend::X11 { backend })
        } else if wlr_backend_is_drm(backend) {
            Backend::DRM(backend::Drm { backend })
        } else if wlr_backend_is_headless(backend) {
            Backend::Headless(backend::Headless { backend })
        } else if wlr_backend_is_multi(backend) {
            Backend::Multi(backend::Multi { backend })
        } else if wlr_backend_is_libinput(backend) {
            Backend::LibInput(backend::Libinput { backend })
        } else {
            panic!("Unknown backend {:p}", backend)
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_backend {
        match *self {
            Backend::Wayland(backend::Wayland { backend }) |
            Backend::X11(backend::X11 { backend }) |
            Backend::DRM(backend::Drm { backend }) |
            Backend::Headless(backend::Headless { backend }) |
            Backend::Multi(backend::Multi { backend }) |
            Backend::LibInput(backend::Libinput { backend }) => backend
        }
    }
}
