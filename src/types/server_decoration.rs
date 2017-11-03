use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wlr_server_decoration_manager, wlr_server_decoration_manager_create,
                  wlr_server_decoration_manager_set_default_mode,
                  wlr_server_decoration_manager_destroy, wl_display};

#[derive(Debug)]
pub struct ServerDecorationManager {
    manager: *mut wlr_server_decoration_manager
}

impl ServerDecorationManager {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_server_decoration_manager_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(ServerDecorationManager { manager: manager_raw })
        } else {
            None
        }
    }

    pub fn set_default_mode(&mut self, mode: u32) {
        unsafe { wlr_server_decoration_manager_set_default_mode(self.manager, mode) }
    }
}

impl Drop for ServerDecorationManager {
    fn drop(&mut self) {
        unsafe { wlr_server_decoration_manager_destroy(self.manager) }
    }
}

pub enum ServerDecorationMode {
    None = 0,
    Client = 1,
    Server = 2
}

impl Into<u32> for ServerDecorationMode {
    fn into(self) -> u32 {
        self as u32
    }
}