use libc::c_int;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{pid_t, wl_client, wl_display, wlr_compositor, wlr_xwayland, wlr_xwayland_create,
                  wlr_xwayland_destroy, wlr_xwayland_set_cursor};

use xwayland;

#[allow(dead_code)]
pub struct Server {
    xwayland: *mut wlr_xwayland,
    manager: &'static mut xwayland::Manager
}

impl Server {
    pub(crate) unsafe fn new(display: *mut wl_display,
                             compositor: *mut wlr_compositor,
                             builder: xwayland::ManagerBuilder,
                             lazy: bool)
                             -> Self {
        let xwayland = wlr_xwayland_create(display, compositor, lazy);
        if xwayland.is_null() {
            panic!("Could not start XWayland server")
        }
        let manager = xwayland::Manager::build(builder);
        wl_signal_add(&mut (*xwayland).events.ready as *mut _ as _,
                      (&mut manager.on_ready_listener) as *mut _ as _);
        wl_signal_add(&mut (*xwayland).events.new_surface as *mut _ as _,
                      (&mut manager.new_surface_listener) as *mut _ as _);
        Server { xwayland, manager }
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
        unsafe { (*self.xwayland).wm_fd }
    }

    pub fn wl_client(&self) -> *mut wl_client {
        unsafe { (*self.xwayland).client }
    }

    pub fn set_cursor(&mut self,
                      bytes: &mut [u8],
                      stride: u32,
                      width: u32,
                      height: u32,
                      hotspot_x: i32,
                      hotspot_y: i32) {
        unsafe {
            wlr_xwayland_set_cursor(self.xwayland,
                                    bytes.as_mut_ptr(),
                                    stride,
                                    width,
                                    height,
                                    hotspot_x,
                                    hotspot_y)
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        unsafe { wlr_xwayland_destroy(self.xwayland) }
    }
}
