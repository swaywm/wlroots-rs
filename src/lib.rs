pub extern crate wlroots_sys;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate wayland_sys;
#[macro_use] extern crate error_chain;

pub mod macros;
mod output;
mod session;
mod backend;

pub use session::{Session, SessionErr};
pub use backend::Backend;
