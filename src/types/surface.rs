//! TODO

use wlroots_sys::wlr_surface;

pub struct Surface {
    surface: *mut wlr_surface
}

impl Surface {
    pub unsafe fn as_ptr(&self) -> *mut wlr_surface {
        self.surface
    }

    pub unsafe fn from_ptr(surface: *mut wlr_surface) -> Self {
        Surface { surface }
    }
}
