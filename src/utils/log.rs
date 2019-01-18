//! Hooks into the wlroots logging functionality. Internally many events are logged
//! and are reported on standard error. The verbosity of the logs is determined by
//! [the verbosity level](type.LogVerbosity.html) when
//! [initializing the logs](fn.init_logging.html).
//!
//! To log using this system please utilize the [`wlr_log!`](../../macro.wlr_log.html) macro.

use libc::c_char;
use log::{info, max_level, set_boxed_logger, set_max_level, Level, LevelFilter, Metadata, Record};
use vsprintf::vsprintf;
use wlroots_sys::{wlr_log_importance, __va_list_tag, wlr_log_init, _wlr_log};

use utils::c_to_rust_string;

use std::ffi::CString;

// Export these so it can be used in `wlr_log!`.
pub use self::wlr_log_importance::{WLR_SILENT, WLR_ERROR, WLR_INFO,
                                   WLR_DEBUG};

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

pub struct Logger;

impl Logger {
    pub fn init(level: LevelFilter) {
        init_logging(
            match level {
                LevelFilter::Off => WLR_SILENT,
                LevelFilter::Warn | LevelFilter::Error => WLR_ERROR,
                LevelFilter::Info => WLR_INFO,
                LevelFilter::Debug | LevelFilter::Trace => WLR_DEBUG,
            },
            None,
        );

        let _ = set_boxed_logger(Box::new(Logger)).map(|_| set_max_level(level));

        info!("Logger initialized!");
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let wlr_level = match record.level() {
                Level::Warn | Level::Error => WLR_ERROR,
                Level::Info => WLR_INFO,
                Level::Debug | Level::Trace => WLR_DEBUG,
            };

            let msg = CString::new(if let Some(file) = record.file() {
                if let Some(line) = record.line() {
                    format!("[{}:{}] {}", file, line, record.args())
                } else {
                    format!("[{}] {}", file, record.args())
                }
            } else {
                format!("{}", record.args())
            })
            .expect("Could not convert log message to CString");

            unsafe {
                _wlr_log(wlr_level, msg.as_ptr());
            }
        }
    }

    fn flush(&self) {}
}
