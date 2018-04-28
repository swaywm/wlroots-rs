use super::{XWaylandManager, XWaylandManagerHandler};
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wl_display, wlr_compositor, wlr_xwayland, wlr_xwayland_create,
                  wlr_xwayland_destroy};

pub struct XWaylandServer {
    manager: Box<XWaylandManager>,
    xwayland: *mut wlr_xwayland
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
}

impl Drop for XWaylandServer {
    fn drop(&mut self) {
        unsafe { wlr_xwayland_destroy(self.xwayland) }
    }
}
