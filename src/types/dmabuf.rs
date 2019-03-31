//! Support for the DMABuf type

use wlroots_sys::{wl_display, wlr_linux_dmabuf_v1, wlr_linux_dmabuf_v1_create, 
                wlr_linux_dmabuf_v1_destroy};
use {compositor::Compositor,
    render::GenericRenderer};


#[derive(Debug)]
pub struct Dmabuf {
    dmabuf: *mut wlr_linux_dmabuf_v1
}

impl Dmabuf {
    pub fn new(compositor: &Compositor) -> Option<Self> {
        unsafe {
            // Get the renderer from compositor
            let renderer: Option<GenericRenderer> = compositor.renderer;
            // Unwrap it and check if it's null
            let renderer_raw = renderer.unwrap().as_ptr();
            if !renderer_raw.is_null() {
                // We found a renderer
                let dmabuf_raw = wlr_linux_dmabuf_v1_create(compositor.display as *mut wl_display, renderer_raw);
                if !dmabuf_raw.is_null() {
                    Some(Dmabuf { dmabuf: dmabuf_raw })
                } else {
                    None
                }
            }
            else {
                 // No renderer found so return none
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
