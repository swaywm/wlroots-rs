#[cfg(feature = "unstable")]
#[allow(clippy::module_inception)]
mod cursor;
pub mod xcursor;
#[cfg(feature = "unstable")]
pub(crate) mod xcursor_manager;

#[cfg(feature = "unstable")]
pub use self::cursor::*;
