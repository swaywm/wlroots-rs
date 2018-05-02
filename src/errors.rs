//! All the errors used in wlroots-rs.

use std::error::Error;
use std::fmt;

/// The result of trying to upgrade a handle, either using `run` or
/// `with_handles!`.
pub type HandleResult<T> = Result<T, HandleErr>;

/// The types of ways upgrading a handle can fail.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HandleErr {
    /// Attempting a handle that already has a mutable borrow to its
    /// backing structure.
    AlreadyBorrowed,
    /// Tried to upgrade a handle for a structure that has already been dropped.
    AlreadyDropped
}

impl fmt::Display for HandleErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HandleErr::*;
        match *self {
            AlreadyBorrowed => write!(f, "AlreadyBorrowed"),
            AlreadyDropped => write!(f, "AlreadyDropped")
        }
    }
}

impl Error for HandleErr {
    fn description(&self) -> &str {
        use HandleErr::*;
        match *self {
            AlreadyBorrowed => "Structure is already mutably borrowed",
            AlreadyDropped => "Structure has already been dropped"
        }
    }
}
