//! Support for the DMABuf type

use wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{wl_display, wlr_renderer, wlr_linux_dmabuf_v1, 
    wlr_linux_dmabuf_v1_create, wlr_linux_dmabuf_v1_destroy};

#[derive(Debug)]
pub struct Dmabuf {
    dmabuf: *mut wlr_linux_dmabuf_v1
}

impl Dmabuf {
    pub(crate) unsafe fn new(display: *mut wl_server_display, renderer: *mut wlr_renderer) -> Option<Self> {
        let dmabuf_raw = wlr_linux_dmabuf_v1_create(display as *mut wl_display, renderer);
        if !dmabuf_raw.is_null() {
            Some(Dmabuf { dmabuf: dmabuf_raw })
        } else {
            None
        }
    }
}

impl Drop for Dmabuf {
    fn drop(&mut self) {
        unsafe { wlr_linux_dmabuf_v1_destroy(self.dmabuf) }
    }
}
