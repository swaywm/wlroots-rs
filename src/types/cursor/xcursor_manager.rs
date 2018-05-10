use wlroots_sys::{wlr_xcursor_manager, wlr_xcursor_manager_create, wlr_xcursor_manager_load, wlr_xcursor_manager_set_cursor_image};
use types::Cursor;
use std::ptr;
use utils::safe_as_cstring;

#[derive(Debug)]
pub struct XCursorManager {
    manager: *mut wlr_xcursor_manager
}

impl XCursorManager {
    pub fn create<T: Into<Option<String>>>(name: T, size: u32) -> Option<Self> {
        unsafe {
            let name_str = name.into().map(safe_as_cstring);
            let name_ptr = name_str.map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
            let manager = wlr_xcursor_manager_create(name_ptr, size);
            if manager.is_null() {
                None
            } else {
                Some(XCursorManager { manager })
            }
        }
    }

    /// returns 0 if the scaled theme was successfully loaded and 1 otherwise
    pub fn load(&self, scale: f32) -> i32 {
        unsafe {
            wlr_xcursor_manager_load(self.manager, scale)
        }
    }

    pub fn set_cursor_image<T: Into<String>>(&self, name: T, cursor: &Cursor) {
        let name_ptr = safe_as_cstring(name.into()).as_ptr();
        unsafe {
            wlr_xcursor_manager_set_cursor_image(self.manager, name_ptr, cursor.as_ptr());
        }
    }
}
