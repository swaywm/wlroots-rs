//! This crate provides safe bindings to
//! [wlroots](https://github.com/swaywm/wlroots).
//!
//! Start your [Compositor](struct.Compositor.html) by implementing an [input
//! manager](manager/struct.InputManager.html) and an [output
//! manager](manager/struct.OutputManager.html) on two separate structs.
//!
//! # Example
//! ```rust,no_run
//! extern crate wlroots;
//!
//! struct InputManager;
//! struct OutputManager;
//!
//! impl wlroots::manager::OutputManagerHandler for OutputManager {}
//! impl wlroots::manager::InputManagerHandler for InputManager {}
//!
//! fn main() {
//!     wlroots::compositor::Compositor::new(Box::new(InputManager),
//!     Box::new(OutputManager)).run()
//! }
//! ```

#![allow(unused_unsafe)]
extern crate libc;
pub extern crate wlroots_sys;
extern crate lazy_static;
#[macro_use]
extern crate wayland_sys;
pub extern crate xkbcommon;


#[macro_use]
mod macros;
mod manager;
mod compositor;
pub mod events;
pub mod types;
mod utils;

pub type NotifyFunc = unsafe extern "C" fn(*mut wlroots_sys::wl_listener, *mut libc::c_void);


pub use self::compositor::{Compositor, terminate};
pub use self::events::key_events::*;
pub use self::events::pointer_events::*;
pub use self::manager::{InputManagerHandler, KeyboardHandler, OutputHandler, OutputManagerHandler,
                        PointerHandler};
pub use self::types::cursor::*;
pub use self::types::input_device::*;
pub use self::types::keyboard::*;
pub use self::types::output::*;
