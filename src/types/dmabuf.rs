//! Support for the DMABuf type

use wlroots_sys::{wl_display, wlr_linux_dmabuf_v1, wlr_linux_dmabuf_v1_create, 
                wlr_linux_dmabuf_v1_destroy};
use {crate::compositor::Compositor,
    crate::render::GenericRenderer};


#[derive(Debug)]
pub struct Dmabuf {
    dmabuf: *mut wlr_linux_dmabuf_v1
}

impl Dmabuf {
    pub fn new(compositor: &Compositor) -> Option<Self> {
        unsafe {
            // Get the renderer from compositor
            let renderer: Option<&GenericRenderer> = compositor.renderer.as_ref();
            let dmabuf_raw = wlr_linux_dmabuf_v1_create(compositor.display as *mut wl_display, renderer.unwrap().as_ptr());
            if !dmabuf_raw.is_null() {
                Some(Dmabuf { dmabuf: dmabuf_raw })
            } else {
                None
            }
        }
    }
    // TODO wrap rest of dmabuf
}

impl Drop for Dmabuf {
    fn drop(&mut self) {
        unsafe { wlr_linux_dmabuf_v1_destroy(self.dmabuf) }
    }
}
