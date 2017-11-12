use render::Texture;
use types::OutputHandle;

use wlroots_sys::{wlr_backend, wlr_gles2_renderer_create, wlr_render_texture_create,
                  wlr_render_with_matrix, wlr_renderer, wlr_renderer_begin, wlr_renderer_destroy,
                  wlr_renderer_end};

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

    // TODO This should probably make a wrapper type or something
    // Because you shouldn't be able to call these methods otherwise.
    // Make this a GLES2
    // Make the thing in the callback a GLES2Renderer

    pub fn render<F>(&mut self, output: &mut OutputHandle, f: F)
    where
        F: Fn(&mut GLES2Renderer, &mut OutputHandle),
    {

        unsafe {
            output.make_current();
            wlr_renderer_begin(self.renderer, output.to_ptr());
            f(self, output);
            wlr_renderer_end(self.renderer);
        }
    }

    pub fn render_with_matrix(&mut self, texture: &Texture, matrix: &[f32; 16]) -> bool {
        unsafe { wlr_render_with_matrix(self.renderer, texture.to_ptr(), matrix) }
    }

    /// Create a texture using the GLES2 backend.
    pub fn create_texture(&mut self) -> Option<Texture> {
        unsafe {
            let texture = wlr_render_texture_create(self.renderer);
            if texture.is_null() {
                wlr_log!(L_ERROR, "Could not create texture");
                None
            } else {
                Some(Texture::from_ptr(texture))
            }
        }
    }
}


impl Drop for GLES2Renderer {
    fn drop(&mut self) {
        unsafe { wlr_renderer_destroy(self.renderer) }
    }
}
