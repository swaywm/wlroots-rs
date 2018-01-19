//! All the errors used in wlroots-rs.

use std::error::Error;
use std::fmt;

pub type UpgradeHandleResult<T> = Result<T, UpgradeHandleErr>;

/// The types of ways upgrading a handle can fail.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UpgradeHandleErr {
    /// Attempting a handle that already has a mutable borrow to its
    /// backing structure.
    AlreadyBorrowed,
    /// Trying to do a double upgrade (e.g downgrading and then upgrading
    /// again).
    DoubleUpgrade
}

impl fmt::Display for UpgradeHandleErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use UpgradeHandleErr::*;
        match *self {
            AlreadyBorrowed => write!(f, "AlreadyBorrowed"),
            DoubleUpgrade => write!(f, "DoubleUpgrade")
        }
    }
}

impl Error for UpgradeHandleErr {
    fn description(&self) -> &str {
        use UpgradeHandleErr::*;
        match *self {
            AlreadyBorrowed => "Structure is already mutably borrowed",
            DoubleUpgrade => "Cannot upgrade a downgraded upgrade"
        }
    }
}
