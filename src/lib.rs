#![allow(unused_unsafe)]
extern crate libc;
pub extern crate wlroots_sys;
extern crate lazy_static;
#[macro_use]
extern crate wayland_sys;
pub extern crate xkbcommon;


#[macro_use]
mod macros;
pub mod manager;
pub mod compositor;
pub mod cursor;
pub mod output;
pub mod device;
pub mod pointer;
pub mod key_event;
mod utils;

pub type NotifyFunc = unsafe extern "C" fn(*mut wlroots_sys::wl_listener, *mut libc::c_void);


pub use self::compositor::terminate;
