use wlroots_sys::wlr_backend;

/// In this backend the only resource the compositor uses is the Wayland file descriptor.
/// It doesn't try to grab actual keyboard/pointers and it doesn't render anything.
///
/// This backend is useful for testing as you can easily add "fake" inputs and outputs.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct HeadlessBackend {
    pub(crate) backend: *mut wlr_backend
}
