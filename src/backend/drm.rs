use wlroots_sys::wlr_backend;

/// When the compositor is ran on a TTY and has full control of the system resources.
///
/// This is primarily the backend that end users will use, as they usually want the
/// compositor to run standalone.
///
/// Note that because you have full control of the TTY (and the keyboard, the mouse, and
/// just about everything else) that if there's an infinite loop then you could hard-lock
/// yourself out of the system. At that point you must reboot your computer (or use
/// SysRq).
///
/// Note that if the process exits for any reason (a panic, an abort, or a clean exit)
/// all of the resource handles will automatically be cleaned up properly by the OS.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct DRMBackend {
    pub(crate) backend: *mut wlr_backend
}
