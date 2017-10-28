//! Utility functions for use within wlroots-rs

use std::ffi::CString;
use std::process::exit;

/// Converts a Rust string into C string without error handling.
/// If any error occurs, it is logged and then the program is immediantly
/// aborted.
pub fn safe_as_cstring<S>(string: S) -> CString
where
    S: Into<Vec<u8>>
{
    match CString::new(string) {
        Ok(string) => string,
        Err(err) => {
            wlr_log!(L_ERROR,
                     "Error occured while trying to convert a \
                      Rust string to a C string {:?}",
                     err);
            exit(1)
        }
    }
}
