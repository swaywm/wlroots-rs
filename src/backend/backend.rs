//! A backend contains information about how clients connected and rendered.
//!
//! In nested environments such as Wayland and X11 the compositor is a client to the hosh
//! process.
//!
//! In the headless backend clients aren't rendered and the only OS resources used are
//! the Wayland file descriptor. This is useful for testing.
//!
//! On the DRM backend the compositor controls an entire TTY. This is the most invasive,
//! but also the most useful backend. If there's an infinite loop here then it is very
//! easy to get into a place where the only course of action is restarting the computer.
//!
//! On the multi backend multiple backends could be running at the same time.

use wlroots_sys::{wlr_backend, wlr_backend_is_wl, wlr_backend_is_x11, wlr_backend_is_drm,
                  wlr_backend_is_headless, wlr_backend_is_multi};

use super::{WaylandBackend, X11Backend, DRMBackend, HeadlessBackend, MultiBackend};

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Backend {
    Wayland(WaylandBackend),
    X11(X11Backend),
    DRM(DRMBackend),
    Headless(HeadlessBackend),
    Multi(MultiBackend)
}

impl Backend {
    pub(crate) unsafe fn new(backend: *mut wlr_backend) -> Self {
        if wlr_backend_is_wl(backend) {
            Backend::Wayland(WaylandBackend{ backend })
        } else if wlr_backend_is_x11(backend) {
            Backend::X11(X11Backend{ backend })
        } else if wlr_backend_is_drm(backend) {
            Backend::DRM(DRMBackend{ backend })
        } else if wlr_backend_is_headless(backend) {
            Backend::Headless(HeadlessBackend{ backend })
        } else if wlr_backend_is_multi(backend) {
            Backend::Multi(MultiBackend{ backend })
        } else {
            panic!("Unknown backend {:p}", backend)
        }
    }
}
