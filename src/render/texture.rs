use wlroots_sys::{wl_shm_format, wlr_texture, wlr_texture_get_matrix, wlr_texture_upload_pixels};

/// Wrapper around wl_shm_format, to make it easier and nicer to type.
#[repr(u32)]
pub enum TextureFormat {
    ARGB8888 = wl_shm_format::WL_SHM_FORMAT_ARGB8888 as u32,
    XRGB8888 = wl_shm_format::WL_SHM_FORMAT_XRGB8888 as u32,
    C8 = wl_shm_format::WL_SHM_FORMAT_C8 as u32,
    RGB332 = wl_shm_format::WL_SHM_FORMAT_RGB332 as u32,
    BGR233 = wl_shm_format::WL_SHM_FORMAT_BGR233 as u32,
    XRGB4444 = wl_shm_format::WL_SHM_FORMAT_XRGB4444 as u32,
    XBGR4444 = wl_shm_format::WL_SHM_FORMAT_XBGR4444 as u32,
    RGBX4444 = wl_shm_format::WL_SHM_FORMAT_RGBX4444 as u32,
    BGRX4444 = wl_shm_format::WL_SHM_FORMAT_BGRX4444 as u32,
    ARGB4444 = wl_shm_format::WL_SHM_FORMAT_ARGB4444 as u32,
    ABGR4444 = wl_shm_format::WL_SHM_FORMAT_ABGR4444 as u32,
    RGBA4444 = wl_shm_format::WL_SHM_FORMAT_RGBA4444 as u32,
    BGRA4444 = wl_shm_format::WL_SHM_FORMAT_BGRA4444 as u32,
    XRGB1555 = wl_shm_format::WL_SHM_FORMAT_XRGB1555 as u32,
    XBGR1555 = wl_shm_format::WL_SHM_FORMAT_XBGR1555 as u32,
    RGBX5551 = wl_shm_format::WL_SHM_FORMAT_RGBX5551 as u32,
    BGRX5551 = wl_shm_format::WL_SHM_FORMAT_BGRX5551 as u32,
    ARGB1555 = wl_shm_format::WL_SHM_FORMAT_ARGB1555 as u32,
    ABGR1555 = wl_shm_format::WL_SHM_FORMAT_ABGR1555 as u32,
    RGBA5551 = wl_shm_format::WL_SHM_FORMAT_RGBA5551 as u32,
    BGRA5551 = wl_shm_format::WL_SHM_FORMAT_BGRA5551 as u32,
    RGB565 = wl_shm_format::WL_SHM_FORMAT_RGB565 as u32,
    BGR565 = wl_shm_format::WL_SHM_FORMAT_BGR565 as u32,
    RGB888 = wl_shm_format::WL_SHM_FORMAT_RGB888 as u32,
    BGR888 = wl_shm_format::WL_SHM_FORMAT_BGR888 as u32,
    XBGR8888 = wl_shm_format::WL_SHM_FORMAT_XBGR8888 as u32,
    RGBX8888 = wl_shm_format::WL_SHM_FORMAT_RGBX8888 as u32,
    BGRX8888 = wl_shm_format::WL_SHM_FORMAT_BGRX8888 as u32,
    ABGR8888 = wl_shm_format::WL_SHM_FORMAT_ABGR8888 as u32,
    RGBA8888 = wl_shm_format::WL_SHM_FORMAT_RGBA8888 as u32,
    BGRA8888 = wl_shm_format::WL_SHM_FORMAT_BGRA8888 as u32,
    XRGB2101010 = wl_shm_format::WL_SHM_FORMAT_XRGB2101010 as u32,
    XBGR2101010 = wl_shm_format::WL_SHM_FORMAT_XBGR2101010 as u32,
    RGBX1010102 = wl_shm_format::WL_SHM_FORMAT_RGBX1010102 as u32,
    BGRX1010102 = wl_shm_format::WL_SHM_FORMAT_BGRX1010102 as u32,
    ARGB2101010 = wl_shm_format::WL_SHM_FORMAT_ARGB2101010 as u32,
    ABGR2101010 = wl_shm_format::WL_SHM_FORMAT_ABGR2101010 as u32,
    RGBA1010102 = wl_shm_format::WL_SHM_FORMAT_RGBA1010102 as u32,
    BGRA1010102 = wl_shm_format::WL_SHM_FORMAT_BGRA1010102 as u32,
    YUYV = wl_shm_format::WL_SHM_FORMAT_YUYV as u32,
    YVYU = wl_shm_format::WL_SHM_FORMAT_YVYU as u32,
    UYVY = wl_shm_format::WL_SHM_FORMAT_UYVY as u32,
    VYUY = wl_shm_format::WL_SHM_FORMAT_VYUY as u32,
    AYUV = wl_shm_format::WL_SHM_FORMAT_AYUV as u32,
    NV12 = wl_shm_format::WL_SHM_FORMAT_NV12 as u32,
    NV21 = wl_shm_format::WL_SHM_FORMAT_NV21 as u32,
    NV16 = wl_shm_format::WL_SHM_FORMAT_NV16 as u32,
    NV61 = wl_shm_format::WL_SHM_FORMAT_NV61 as u32,
    YUV410 = wl_shm_format::WL_SHM_FORMAT_YUV410 as u32,
    YVU410 = wl_shm_format::WL_SHM_FORMAT_YVU410 as u32,
    YUV411 = wl_shm_format::WL_SHM_FORMAT_YUV411 as u32,
    YVU411 = wl_shm_format::WL_SHM_FORMAT_YVU411 as u32,
    YUV420 = wl_shm_format::WL_SHM_FORMAT_YUV420 as u32,
    YVU420 = wl_shm_format::WL_SHM_FORMAT_YVU420 as u32,
    YUV422 = wl_shm_format::WL_SHM_FORMAT_YUV422 as u32,
    YVU422 = wl_shm_format::WL_SHM_FORMAT_YVU422 as u32,
    YUV444 = wl_shm_format::WL_SHM_FORMAT_YUV444 as u32,
    YVU444 = wl_shm_format::WL_SHM_FORMAT_YVU444 as u32
}

impl Into<wl_shm_format> for TextureFormat {
    fn into(self) -> wl_shm_format {
        // NOTE Rationale for transmute:
        // * Easiest way to convert to the value type
        // * Is safe because of the definitions above linking them together.
        unsafe { ::std::mem::transmute(self as u32) }
    }
}

pub struct Texture {
    texture: *mut wlr_texture
}

impl Texture {
    pub(crate) unsafe fn from_ptr(texture: *mut wlr_texture) -> Self {
        Texture { texture }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_texture {
        self.texture
    }

    pub fn upload_pixels(&mut self,
                         texture_format: TextureFormat,
                         stride: i32,
                         width: i32,
                         height: i32,
                         bytes: &[u8])
                         -> bool {
        unsafe {
            wlr_texture_upload_pixels(self.texture,
                                      texture_format.into(),
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
