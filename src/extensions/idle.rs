//! Support for the KDE Idle Protocol

use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wl_display, wlr_idle, wlr_idle_create,
                  wlr_idle_destroy, wlr_idle_notify_activity,
                  wlr_idle_set_enabled};
pub use wlroots_sys::protocols::idle
::server::org_kde_kwin_idle::Request;


#[derive(Debug)]
pub struct Manager {
    manager: *mut wlr_idle
}

impl Manager {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_idle_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(Manager { manager: manager_raw })
        } else {
            None
        }
    }

    /// Restart the timers for the seat
    pub fn notify_activity(&mut self, Seat: Request::GetIdleTimeout::seat) {
        unsafe { wlr_idle_notify_activity(self.manager, Seat) } 
    }

    /// If we are passed a null pointer,
    /// update timers for all seats.
    pub fn set_enabled(&mut self, Seat: Request::GetIdleTimeout::seat, enabled: bool) {
        wlr_log!(WLR_INFO, "Idle timer status: {:?}", enabled);
        unsafe { wlr_idle_set_enabled(self.manager, Seat, enabled) }
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe { wlr_idle_destroy(self.manager) }
    }
}
