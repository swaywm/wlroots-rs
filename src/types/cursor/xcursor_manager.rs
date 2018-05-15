use std::marker::PhantomData;
use std::ptr;
use types::{Cursor, XCursor, XCursorTheme};
use utils::{c_to_rust_string, safe_as_cstring};
use wlroots_sys::{wlr_xcursor_manager, wlr_xcursor_manager_create, wlr_xcursor_manager_destroy,
                  wlr_xcursor_manager_get_xcursor, wlr_xcursor_manager_load,
                  wlr_xcursor_manager_set_cursor_image, wlr_xcursor_manager_theme};

/// An `XCursorTheme` at a particular scale factor of the base size.
#[derive(Debug)]
pub struct XCursorManagerTheme<'manager> {
    theme: *mut wlr_xcursor_manager_theme,
    phantom: PhantomData<&'manager XCursorManager>
}

/// XCursorManager dynamically loads xcursor themes at sizes necessary for use on outputs at
/// arbitrary scale factors. You should call `load` for each output you will show your cursor on,
/// with the scale factor parameter set to that output's scale factor.
#[derive(Debug)]
pub struct XCursorManager {
    manager: *mut wlr_xcursor_manager
}

impl<'manager> XCursorManagerTheme<'manager> {
    fn new(theme: *mut wlr_xcursor_manager_theme) -> Self {
        XCursorManagerTheme { theme: theme,
                              phantom: PhantomData }
    }

    /// Get the scale factor of the theme.
    pub fn scale(&self) -> f32 {
        unsafe { (*self.theme).scale }
    }

    /// Get the underlying `XCursorTheme` for this managed theme.
    pub fn theme(self) -> XCursorTheme {
        unsafe { XCursorTheme::new((*self.theme).theme) }
    }
}

impl XCursorManager {
    /// Create a new `XCursorManager`.
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

    /// Get the name of the theme this `XCursorManager` manages.
    pub fn name(&self) -> String {
        unsafe { c_to_rust_string((*self.manager).name).expect("Could not parse make as UTF-8") }
    }

    /// Get the base size (when scale = 1) in pixels for the theme.
    pub fn size(&self) -> u32 {
        unsafe { (*self.manager).size }
    }

    /// Retrieves a `XCursor` for the given cursor name at the given scale factor, or None if this
    /// `XCursorManager` has not loaded a cursor theme at the requested scale.
    pub fn get_xcursor<'manager, T: Into<Option<String>>>(&'manager self,
                                                          name: T,
                                                          scale: f32)
                                                          -> Option<XCursor<'manager>> {
        let name_str = name.into().map(safe_as_cstring);
        let name_ptr = name_str.map(|s| s.as_ptr()).unwrap_or(ptr::null_mut());
        unsafe {
            let xcursor = wlr_xcursor_manager_get_xcursor(self.manager, name_ptr, scale);
            if xcursor.is_null() {
                None
            } else {
                Some(XCursor::from_ptr(xcursor))
            }
        }
    }

    /// Get a list of all the scaled `XCursorManagerTheme`s managed by this
    /// manager.
    pub fn scaled_themes<'manager>(&'manager self) -> Vec<XCursorManagerTheme<'manager>> {
        unsafe {
            let mut result = vec![];

            wl_list_for_each!((*self.manager).scaled_themes,
                              link,
                              (theme: wlr_xcursor_manager_theme) => {
                result.push(XCursorManagerTheme::new(theme))
            });

            result
        }
    }

    /// Ensures an xcursor theme at the given scale factor is loaded in the manager.
    ///
    /// Returns false if the scaled theme was successfully loaded and true otherwise
    pub fn load(&self, scale: f32) -> bool {
        unsafe {
            match wlr_xcursor_manager_load(self.manager, scale) {
                0 => false,
                _ => true
            }
        }
    }

    /// Set a `Cursor`'s cursor image to the specified cursor name for all scale
    /// factors. The `Cursor` will take over from this point and ensure the correct
    /// cursor is used on each output, assuming an `OutputLayout` is attached to
    /// it.
    pub fn set_cursor_image(&mut self, name: String, cursor: &Cursor) {
        let name_str = safe_as_cstring(name);
        unsafe {
            wlr_xcursor_manager_set_cursor_image(self.manager, name_str.as_ptr(), cursor.as_ptr());
        }
    }
}

impl Drop for XCursorManager {
    fn drop(&mut self) {
        unsafe { wlr_xcursor_manager_destroy(self.manager) }
    }
}
