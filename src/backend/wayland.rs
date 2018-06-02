use wlroots_sys::{wlr_backend};

/// When the compositor is running in a nested Wayland environment.
/// e.g. your compositor is executed while the user is running Gnome+Mutter or Weston.
///
/// This is useful for testing and iterating on the design of the compositor.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct WaylandBackend {
    pub(crate) backend: *mut wlr_backend
}
