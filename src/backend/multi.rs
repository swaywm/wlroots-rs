use wlroots_sys::wlr_backend;

/// When multiple backends are running or when the compositor writer doesn't care and
/// just used the auto create option in the `CompositorBuilder`.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct MultiBackend {
    pub(crate) backend: *mut wlr_backend
}
