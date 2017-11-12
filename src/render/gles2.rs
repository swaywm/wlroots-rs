use types::OutputHandle;
use wlroots_sys::{wlr_backend, wlr_gles2_renderer_create, wlr_renderer, wlr_renderer_begin,
                  wlr_renderer_destroy, wlr_renderer_end};

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

    pub fn render<F>(&mut self, output: &mut OutputHandle, f: F)
    where
        F: Fn(&mut OutputHandle, &mut GLES2Renderer),
    {

        unsafe {
            output.make_current();
            wlr_renderer_begin(self.renderer, output.to_ptr());
            f(output, self);
            wlr_renderer_end(self.renderer);
        }
    }
}


impl Drop for GLES2Renderer {
    fn drop(&mut self) {
        unsafe { wlr_renderer_destroy(self.renderer) }
    }
}
