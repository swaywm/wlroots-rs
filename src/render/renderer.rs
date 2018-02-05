//! TODO Documentation

use Output;
use render::Texture;
use wlroots_sys::{wlr_backend, wlr_render_colored_ellipse, wlr_render_colored_quad,
                  wlr_render_texture_create, wlr_render_with_matrix, wlr_renderer,
                  wlr_renderer_begin, wlr_renderer_destroy, wlr_renderer_end,
                  wlr_gles2_renderer_create};

/// A generic interface for rendering to the screen.
///
/// Note that it will technically be possible to have multiple renderers
/// at the same time.
#[derive(Debug)]
pub struct GenericRenderer {
    renderer: *mut wlr_renderer
}

/// The state machine type that allows you to manipulate a screen and
/// its buffer.
///
/// When this structure is dropped it automatically calls wlr_renderer_end
/// and swaps the buffers.
#[derive(Debug)]
pub struct Renderer<'output> {
    renderer: *mut wlr_renderer,
    pub output: &'output mut Output
}

impl GenericRenderer {
    /// Make a gles2 renderer.
    pub(crate) unsafe fn gles2_renderer(backend: *mut wlr_backend) -> Self {
        let renderer = wlr_gles2_renderer_create(backend);
        if renderer.is_null() {
            panic!("Could not construct GLES2 renderer");
        }
        GenericRenderer { renderer }
    }

    /// Make the `Renderer` state machine type.
    ///
    /// This automatically makes the given output the current output.
    pub fn render<'output>(&mut self, output: &'output mut Output) -> Renderer<'output> {
        unsafe {
            output.make_current();
            wlr_renderer_begin(self.renderer, output.as_ptr());
            Renderer { renderer: self.renderer,
                       output }
        }
    }

    /// Create a texture using this renderer.
    pub fn create_texture(&mut self) -> Option<Texture> {
        unsafe { create_texture(self.renderer) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_renderer {
        self.renderer
    }
}

impl Drop for GenericRenderer {
    fn drop(&mut self) {
        unsafe { wlr_renderer_destroy(self.renderer) }
    }
}

impl<'output> Renderer<'output> {
    /// Create a texture using this renderer
    pub fn create_texture(&mut self) -> Option<Texture> {
        unsafe { create_texture(self.renderer) }
    }

    /// Renders the requested texture using the provided matrix. A typical texture
    /// rendering goes like so:
    ///
    /// TODO FIXME Show how the typical rendering goes in Rust.
    ///
    /// ```c
    /// struct wlr_renderer *renderer;
    /// struct wlr_texture *texture;
    /// float projection[16];
    /// float matrix[16];
    /// wlr_texture_get_matrix(texture, &matrix, &projection, 123, 321);
    /// wlr_render_with_matrix(renderer, texture, &matrix);
    /// ```
    ///
    /// This will render the texture at <123, 321>.
    pub fn render_with_matrix(&mut self, texture: &Texture, matrix: &[f32; 16]) -> bool {
        unsafe { wlr_render_with_matrix(self.renderer, texture.as_ptr(), matrix) }
    }

    /// Renders a solid quad in the specified color.
    pub fn render_colored_quad(&mut self, color: &[f32; 4], matrix: &[f32; 16]) {
        unsafe { wlr_render_colored_quad(self.renderer, color, matrix) }
    }

    /// Renders a solid ellipse in the specified color.
    pub fn render_colored_ellipse(&mut self, color: &[f32; 4], matrix: &[f32; 16]) {
        unsafe { wlr_render_colored_ellipse(self.renderer, color, matrix) }
    }
}

impl<'output> Drop for Renderer<'output> {
    fn drop(&mut self) {
        unsafe {
            wlr_renderer_end(self.renderer);
            self.output.swap_buffers();
        }
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
