use Output;
use render::Texture;

use wlroots_sys::{wlr_backend, wlr_render_texture_create, wlr_render_with_matrix, wlr_renderer,
                  wlr_renderer_begin, wlr_renderer_destroy, wlr_renderer_end,
                  wlr_gles2_renderer_create};

/// Holds the state necessary to start rendering for GLES2.
pub struct GLES2 {
    renderer: *mut wlr_renderer
}

/// Renderer for GLES2
pub struct GLES2Renderer<'output> {
    renderer: *mut wlr_renderer,
    output: &'output mut Output
}

impl GLES2 {
    pub(crate) unsafe fn new(backend: *mut wlr_backend) -> Option<Self> {
        if backend.is_null() {
            wlr_log!(L_ERROR, "Backend was null");
            return None
        }
        let renderer = wlr_gles2_renderer_create(backend);
        if renderer.is_null() {
            wlr_log!(L_ERROR, "Could not construct GLES2 renderer");
            None
        } else {
            Some(GLES2 { renderer })
        }
    }

    pub fn render<'output>(&mut self, output: &'output mut Output) -> GLES2Renderer<'output> {
        output.make_current();
        unsafe {
            wlr_renderer_begin(self.renderer, output.as_ptr());
        }
        GLES2Renderer { renderer: self.renderer,
                        output }
    }

    pub fn create_texture(&mut self) -> Option<Texture> {
        unsafe { create_texture(self.renderer) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_renderer {
        self.renderer
    }
}

impl<'output> GLES2Renderer<'output> {
    pub fn render_with_matrix(&mut self, texture: &Texture, matrix: &[f32; 16]) -> bool {
        unsafe { wlr_render_with_matrix(self.renderer, texture.as_ptr(), matrix) }
    }

    /// Create a texture using the GLES2 backend.
    pub fn create_texture(&mut self) -> Option<Texture> {
        unsafe { create_texture(self.renderer) }
    }
}

impl<'output> Drop for GLES2Renderer<'output> {
    fn drop(&mut self) {
        unsafe {
            wlr_renderer_end(self.renderer);
        }
        self.output.swap_buffers()
    }
}

impl Drop for GLES2 {
    fn drop(&mut self) {
        unsafe { wlr_renderer_destroy(self.renderer) }
    }
}

unsafe fn create_texture(renderer: *mut wlr_renderer) -> Option<Texture> {
    let texture = wlr_render_texture_create(renderer);
    if texture.is_null() {
        wlr_log!(L_ERROR, "Could not create texture");
        None
    } else {
        Some(Texture::from_ptr(texture))
    }
}
