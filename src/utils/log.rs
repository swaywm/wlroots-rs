//! Hooks into the wlroots logging functionality. Internally many events are
//! logged and are reported on standard error. The verbosity of the logs is
//! determined by [the verbosity level](type.LogVerbosity.html) when
//! [initializing the logs](fn.init_logging.html).
//!
//! To log using this system please utilize the
//! [`wlr_log!`](../../macro.wlr_log.html) macro.

use crate::libc::c_char;
use vsprintf::vsprintf;
use wlroots_sys::{__va_list_tag, _wlr_log, wlr_log_importance, wlr_log_init};

use crate::utils::c_to_rust_string;

use std::ffi::CString;

// Export these so it can be used in `wlr_log!`.
pub use self::wlr_log_importance::{WLR_DEBUG, WLR_ERROR, WLR_INFO, WLR_SILENT};

/// How verbose you want the logging. Lower levels prints more.
pub type LogVerbosity = wlr_log_importance;

/// The signature for the callback function you can hook into the logging
/// functionality of wlroots.
///
/// `message` is the formatted string ready to be displayed on the screen.
pub type LogCallback = fn(verbosity: LogVerbosity, message: String);

static mut RUST_LOGGING_FN: LogCallback = dummy_callback;

/// Initialize wlroots logging at a certain level of verbosity with
/// an optional callback that will be called for every log.
///
/// To log using this system, use the
/// [`wlr_log!`](../../macro.wlr_log.html) macro.
pub fn init_logging<F>(verbosity: LogVerbosity, callback: F)
where
    F: Into<Option<LogCallback>>
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
unsafe extern "C" fn log_callback(
    importance: wlr_log_importance,
    fmt: *const c_char,
    va_list: *mut __va_list_tag
) {
    let message =
        vsprintf(fmt, va_list).unwrap_or_else(|_| c_to_rust_string(fmt).unwrap_or_else(|| "".into()));
    RUST_LOGGING_FN(importance, message);
}

pub struct Logger;

static LOGGER: Logger = Logger;

impl Logger {
    /// Attempts to initialize the global logger with a Logger around _wlr_log.
    ///
    /// This should be called early in the execution of the program, as all log
    /// events that occur before initialization with be ignored.
    ///
    /// # Errors
    ///
    /// This function will fail if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn init<F>(level: log::LevelFilter, callback: F)
    where
        F: Into<Option<LogCallback>>
    {
        init_logging(
            match level {
                log::LevelFilter::Off => WLR_SILENT,
                log::LevelFilter::Warn | log::LevelFilter::Error => WLR_ERROR,
                log::LevelFilter::Info => WLR_INFO,
                log::LevelFilter::Debug | log::LevelFilter::Trace => WLR_DEBUG
            },
            callback
        );

        log::set_logger(&LOGGER).expect(
            "Attempted to set a logger after the logging system was \
             already initialized"
        );
        log::set_max_level(level);
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let wlr_level = match record.level() {
                log::Level::Warn | log::Level::Error => WLR_ERROR,
                log::Level::Info => WLR_INFO,
                log::Level::Debug | log::Level::Trace => WLR_DEBUG
            };

            let formatted_msg = match (record.file(), record.line()) {
                (Some(file), Some(line)) => format!("[{}:{}] {}", file, line, record.args()),
                (Some(file), None) => format!("[{}] {}", file, record.args()),
                (None, _) => format!("{}", record.args())
            };
            let msg = CString::new(formatted_msg).expect("Could not convert log message to CString");

            unsafe {
                _wlr_log(wlr_level, msg.as_ptr());
            }
        }
    }

    fn flush(&self) {}
}
