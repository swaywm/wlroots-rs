use wlroots_sys::{wl_shm_format, wlr_texture, wlr_texture_get_matrix, wlr_texture_upload_pixels};

pub struct Texture {
    texture: *mut wlr_texture
}

impl Texture {
    pub(crate) unsafe fn from_ptr(texture: *mut wlr_texture) -> Self {
        Texture { texture }
    }

    pub(crate) unsafe fn to_ptr(&self) -> *mut wlr_texture {
        self.texture
    }

    // TODO Different Formats!
    // FIXME Safety of providing the size data? Can we return an Err?
    /// Uploads pixels from the buffer to the texture
    pub fn upload_pixels(&mut self, stride: i32, width: i32, height: i32, bytes: &[u8]) -> bool {
        unsafe {
            wlr_texture_upload_pixels(self.texture,
                                      wl_shm_format::WL_SHM_FORMAT_ABGR8888,
                                      stride,
                                      width,
                                      height,
                                      bytes.as_ptr())
        }
    }

    pub fn get_matrix(&self, projection: &[f32; 16], x: i32, y: i32) -> [f32; 16] {
        unsafe {
            let mut matrix: [f32; 16] = [0.0; 16];
            wlr_texture_get_matrix(self.texture, &mut matrix, projection, x, y);
            matrix
        }
    }
}
