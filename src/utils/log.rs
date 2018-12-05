//! Hooks into the wlroots logging functionality. Internally many events are logged
//! and are reported on standard error. The verbosity of the logs is determined by
//! [the verbosity level](type.LogVerbosity.html) when
//! [initializing the logs](fn.init_logging.html).
//!
//! To log using this system please utilize the [`wlr_log!`](../../macro.wlr_log.html) macro.

use libc::c_char;
pub use wlroots_sys::wlr_log_importance::{self, WLR_SILENT, WLR_ERROR, WLR_INFO,
                                          WLR_DEBUG, WLR_LOG_IMPORTANCE_LAST};
use wlroots_sys::{__va_list_tag, wlr_log_init};
use vsprintf::vsprintf;

use utils::c_to_rust_string;

/// How verbose you want the logging. Lower levels prints more.
pub type LogVerbosity = wlr_log_importance;

/// The signature for the callback function you can hook into the logging
/// functionality of wlroots.
pub type LogCallback = fn(LogVerbosity, String);

static mut RUST_LOGGING_FN: LogCallback = dummy_callback;

/// Initialize wlroots logging at a certain level of verbosity with
/// an optional callback that will be called for every log.
///
/// To log using this system, use the `wlr_log!` macro.
pub fn init_logging<F>(verbosity: LogVerbosity, callback: F)
where F: Into<Option<LogCallback>>
{
    unsafe {
        match callback.into() {
            None => wlr_log_init(verbosity, None),
            Some(callback) => {
                RUST_LOGGING_FN = callback;
                wlr_log_init(verbosity, Some(log_callback));
            }
        }
    }
}

/// Dummy callback to fill in RUST_LOGGING_FN when it's not in use.
fn dummy_callback(_: LogVerbosity, _: String) {}

/// Real hook into the logging callback, calls the real user-supplied callback
/// with nice Rust inputs.
unsafe extern "C" fn log_callback(importance: wlr_log_importance,
                                  fmt: *const c_char,
                                  va_list: *mut __va_list_tag) {
    let message = vsprintf(fmt, va_list).unwrap_or_else(|_| {
        c_to_rust_string(fmt).unwrap_or_else(|| "".into())
    });
    RUST_LOGGING_FN(importance, message);
}
