//! Utility functions for use within wlroots-rs

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process::exit;

/// Converts a Rust string into C string without error handling.
/// If any error occurs, it is logged and then the program is immediantly
/// aborted.
pub fn safe_as_cstring<S>(string: S) -> CString
    where S: Into<Vec<u8>>
{
    match CString::new(string) {
        Ok(string) => string,
        Err(err) => {
            wlr_log!(L_ERROR,
                     "Error occured while trying to convert a Rust string to a C string {:?}",
                     err);
            exit(1)
        }
    }
}

/// Converts a C string into a Rust string without error handling.
/// The pointer passed to this function _must_ be valid.
pub unsafe fn c_to_rust_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        Some(CStr::from_ptr(c_str).to_string_lossy().into_owned())
    }
}
