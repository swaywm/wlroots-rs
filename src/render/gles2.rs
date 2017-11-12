use wlroots_sys::{wlr_backend, wlr_gles2_renderer_create, wlr_renderer, wlr_renderer_destroy};

/// Renderer for GLES2
pub struct GLES2Renderer {
    renderer: *mut wlr_renderer
}

impl GLES2Renderer {
    pub(crate) unsafe fn new(backend: *mut wlr_backend) -> Option<Self> {
        if backend.is_null() {
            wlr_log!(L_ERROR, "Backend was null");
            return None;
        }
        let renderer = wlr_gles2_renderer_create(backend);
        if renderer.is_null() {
            wlr_log!(L_ERROR, "Could not construct GLES2 renderer");
            None
        } else {
            Some(GLES2Renderer { renderer })
        }
    }
}


impl Drop for GLES2Renderer {
    fn drop(&mut self) {
        unsafe { wlr_renderer_destroy(self.renderer) }
    }
}
