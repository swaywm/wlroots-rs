extern crate libc;
pub extern crate wlroots_sys;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate wayland_sys;

#[macro_use] mod macros;
pub mod manager;
pub mod compositor;
