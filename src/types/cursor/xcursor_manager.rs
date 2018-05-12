use wlroots_sys::{wlr_xcursor_manager, wlr_xcursor_manager_create, wlr_xcursor_manager_load,
                  wlr_xcursor_manager_set_cursor_image, wlr_xcursor_manager_destroy,
                  wlr_xcursor_manager_get_xcursor};
use types::{Cursor, XCursor};
use std::ptr;
use utils::safe_as_cstring;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct XCursorManager<'manager> {
    manager: *mut wlr_xcursor_manager,
    phantom: PhantomData<XCursor<'manager>>
}

impl<'manager> XCursorManager<'manager> {
    pub fn create<T: Into<Option<String>>>(name: T, size: u32) -> Option<Self> {
        unsafe {
            let name_str = name.into().map(safe_as_cstring);
            let name_ptr = name_str.map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
            let manager = wlr_xcursor_manager_create(name_ptr, size);
            if manager.is_null() {
                None
            } else {
                Some(XCursorManager { manager: manager, phantom: PhantomData })
            }
        }
    }

    pub fn size(&self) -> u32 {
        unsafe { (*self.manager).size }
    }

    pub fn get_xcursor<T: Into<Option<String>>>(&'manager self, name: T, scale: f32) -> Option<XCursor<'manager>> {
        let name_str = name.into().map(safe_as_cstring);
        let name_ptr = name_str.map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
        unsafe {
            let xcursor = wlr_xcursor_manager_get_xcursor(self.manager, name_ptr, scale);
            if xcursor.is_null() {
                None
            } else {
                Some(XCursor::new(xcursor))
            }
        }
    }

    /// returns 0 if the scaled theme was successfully loaded and 1 otherwise
    pub fn load(&self, scale: f32) -> i32 {
        unsafe {
            wlr_xcursor_manager_load(self.manager, scale)
        }
    }

    pub fn set_cursor_image(&mut self, name: String, cursor: &Cursor) {
        let name_ptr = safe_as_cstring(name).as_ptr();
        unsafe {
            wlr_xcursor_manager_set_cursor_image(self.manager, name_ptr, cursor.as_ptr());
        }
    }
}

impl<'manager> Drop for XCursorManager<'manager> {
    fn drop(&mut self) {
        unsafe { wlr_xcursor_manager_destroy(self.manager) }
    }
}
