use wlroots_sys::{wlr_xcursor_manager, wlr_xcursor_manager_create, wlr_xcursor_manager_load,
                  wlr_xcursor_manager_set_cursor_image, wlr_xcursor_manager_destroy,
                  wlr_xcursor_manager_get_xcursor, wlr_xcursor_manager_theme};
use types::{Cursor, XCursor, XCursorTheme};
use std::ptr;
use utils::safe_as_cstring;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct XCursorManagerTheme<'manager> {
    theme: *mut wlr_xcursor_manager_theme,
    phantom: PhantomData<&'manager XCursorManager>
}

#[derive(Debug)]
pub struct XCursorManager {
    manager: *mut wlr_xcursor_manager,
}

impl<'manager> XCursorManagerTheme<'manager> {
    pub(crate) fn new(theme: *mut wlr_xcursor_manager_theme) -> Self {
        XCursorManagerTheme {
            theme: theme,
            phantom: PhantomData
        }
    }

    pub fn scale(&self) -> f32 {
        unsafe {
            (*self.theme).scale
        }
    }

    pub fn theme(self) -> XCursorTheme {
        unsafe {
            XCursorTheme::new((*self.theme).theme)
        }
    }
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
                Some(XCursorManager { manager: manager })
            }
        }
    }

    pub fn size(&self) -> u32 {
        unsafe { (*self.manager).size }
    }

    pub fn get_xcursor<'manager, T: Into<Option<String>>>(&'manager self, name: T, scale: f32) -> Option<XCursor<'manager>> {
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

    pub fn scaled_themes<'manager>(&'manager self) -> Vec<XCursorManagerTheme<'manager>> {
        unsafe {
            let mut result = vec![];

            wl_list_for_each!((*self.manager).scaled_themes, link, (theme: wlr_xcursor_manager_theme) => {
                result.push(XCursorManagerTheme::new(theme))
            });

            result
        }
    }

    /// returns false if the scaled theme was successfully loaded and true otherwise
    pub fn load(&self, scale: f32) -> bool {
        unsafe {
            match wlr_xcursor_manager_load(self.manager, scale) {
                0 => false,
                _ => true
            }
        }
    }

    pub fn set_cursor_image(&mut self, name: String, cursor: &Cursor) {
        unsafe {
            wlr_xcursor_manager_set_cursor_image(self.manager, safe_as_cstring(name).as_ptr(), cursor.as_ptr());
        }
    }
}

impl Drop for XCursorManager {
    fn drop(&mut self) {
        unsafe { wlr_xcursor_manager_destroy(self.manager) }
    }
}
