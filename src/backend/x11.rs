use wlroots_sys::wlr_backend;

/// When the compositor is running in a nested X11 environment.
/// e.g. your compositor is executed while the user is running an X11 window manager.
///
/// This is useful for testing and iteration on the design of the compositor.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct X11Backend {
    pub(crate) backend: *mut wlr_backend
}
