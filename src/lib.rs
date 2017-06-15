pub extern crate wlroots_sys;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate wayland_sys;
#[macro_use] extern crate error_chain;

mod output;
mod session;
mod backend;
pub mod utils;

pub use session::{Session, SessionErr};
pub use backend::Backend;
pub use utils::*;
