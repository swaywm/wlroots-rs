use libc::c_int;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wl_display, wlr_compositor, wlr_xwayland, wlr_xwayland_create,
                  wlr_xwayland_destroy, pid_t, wl_client};

use super::{XWaylandManager, XWaylandManagerHandler};

pub struct XWaylandServer {
    xwayland: *mut wlr_xwayland,
    manager: Box<XWaylandManager>
}

impl XWaylandServer {
    pub(crate) unsafe fn new(display: *mut wl_display,
                             compositor: *mut wlr_compositor,
                             manager: Box<XWaylandManagerHandler>)
                             -> Self {
        let xwayland = wlr_xwayland_create(display, compositor);
        let mut manager = XWaylandManager::new(manager);
        wl_signal_add(&mut (*xwayland).events.ready as *mut _ as _,
                      manager.on_ready_listener() as *mut _ as _);
        wl_signal_add(&mut (*xwayland).events.new_surface as *mut _ as _,
                      manager.new_surface_listener() as *mut _ as _);
        if xwayland.is_null() {
            panic!("Could not start XWayland server")
        }
        XWaylandServer { xwayland, manager }
    }

    /// Get the PID of the XWayland server.
    pub fn pid(&self) -> pid_t {
        unsafe { (*self.xwayland).pid }
    }

    pub fn display(&self) -> c_int {
        unsafe { (*self.xwayland).display }
    }

    pub fn x_fd(&self) -> [c_int; 2] {
        unsafe { (*self.xwayland).x_fd }
    }

    pub fn wl_fd(&self) -> [c_int; 2] {
        unsafe { (*self.xwayland).wl_fd }
    }

    pub fn wm_fd(&self) -> [c_int; 2] {
        unsafe {(*self.xwayland).wm_fd }
    }

    pub fn wl_client(&self) -> *mut wl_client {
        unsafe { (*self.xwayland).client }
    }
}

impl Drop for XWaylandServer {
    fn drop(&mut self) {
        unsafe { wlr_xwayland_destroy(self.xwayland) }
    }
}
