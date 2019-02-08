use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wl_display, wlr_screenshooter,wlr_screenshooter_create,
                  wlr_screenshooter_destroy};

#[derive(Debug)]
pub struct Screenshooter {
    screenshooter: *mut wlr_screenshooter
}

impl Screenshooter {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let screenshooter_raw = wlr_screenshooter_create(display as *mut wl_display);

        if !screenshooter_raw.is_null() {
            Some(Screenshooter { screenshooter: screenshooter_raw })
        } else {
            None
        }
    }
}

impl Drop for Screenshooter {
    fn drop(&mut self) {
        unsafe { wlr_screenshooter_destroy(self.screenshooter) }
    }
}
