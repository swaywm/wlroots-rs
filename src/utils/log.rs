
use libc::c_char;
pub use wlroots_sys::wlr_log_importance::{self, *};
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
// TODO Wrap the callback function type
pub fn init_logging<T>(verbosity: LogVerbosity, callback: T)
where T: Into<Option<LogCallback>>
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
