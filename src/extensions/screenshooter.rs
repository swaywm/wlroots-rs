use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wl_display, wlr_screenshooter_manager, wlr_screenshooter_manager_create, 
    wlr_screenshooter_manager_destroy};

#[derive(Debug)]
pub struct Manager {
    manager: *mut wlr_screenshooter_manager
}

impl Manager {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_screenshooter_manager_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(Manager { manager: manager_raw })
        } else {
            None
        }
    }

}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe { wlr_screenshooter_manager_destroy(self.manager) }
    }
}
