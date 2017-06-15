//! The backend that a session uses to display content to the screen.
//!
//! There are three main backends:
//! * DRM
//! * Wayland
//! * X11
use wlroots_sys::{wl_display, wlr_session, wlr_backend,
                  wlr_session_start, wlr_backend_autocreate, wlr_backend_init};
use ::{Session, SessionErr};

pub struct Backend(pub *mut wlr_backend);

impl Backend {
    /// Auto-create a new backend using information for the provided session.
    pub fn autocreate_backend(display: *mut wl_display,
                              session: *mut wlr_session)
                              -> Result<Self, ::SessionErr> {
        unsafe {
            let backend = wlr_backend_autocreate(display, session);
            if backend.is_null() {
                Err(SessionErr::BackendFailed)
            } else {
                if ! wlr_backend_init(backend) {
                    // TODO Make a more specific error
                    Err(SessionErr::BackendFailed)
                } else {
                    Ok(Backend(backend))
                }
            }
        }
    }
}
