use wlroots_sys::{wlr_xwayland, wlr_compositor, wlr_xwayland_create, wl_display, wlr_xwayland_destroy};

pub struct XWaylandServer {
    xwayland: *mut wlr_xwayland
}

impl XWaylandServer {
    pub(crate) unsafe fn new(display: *mut wl_display, compositor: *mut wlr_compositor) -> Self {
        let xwayland = wlr_xwayland_create(display, compositor);
        if xwayland.is_null() {
            panic!("Could not start XWayland server")
        }
        XWaylandServer { xwayland }
    }
}

impl Drop for XWaylandServer {
    fn drop(&mut self) {
        wlr_xwayland_destroy(self.xwayland)
    }
}
