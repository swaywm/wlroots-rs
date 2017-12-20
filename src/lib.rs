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
//! impl wlroots::OutputManagerHandler for OutputManager {}
//! impl wlroots::InputManagerHandler for InputManager {}
//!
//! fn main() {
//!     wlroots::CompositorBuilder::new()
//!          .build_auto((), // Dummy state
//!                      Box::new(InputManager),
//!                      Box::new(OutputManager))
//!          .run()
//! }
//! ```

#![allow(unused_unsafe)]
#[macro_use]
extern crate bitflags;
extern crate lazy_static;
extern crate libc;
#[macro_use]
pub extern crate wayland_sys;
pub extern crate wlroots_sys;
pub extern crate xkbcommon;

#[macro_use]
mod macros;
mod manager;
mod compositor;
pub mod events;
pub mod types;
pub mod extensions;
pub mod render;
mod utils;

pub use self::compositor::{terminate, Compositor, CompositorBuilder};
pub use self::events::key_events::*;
pub use self::events::pointer_events::*;
pub use self::manager::{InputManagerHandler, KeyboardHandler, OutputBuilder, OutputBuilderResult,
                        OutputHandler, OutputManagerHandler, PointerHandler};
pub use self::types::area::*;
pub use self::types::cursor::*;
pub use self::types::input_device::*;
pub use self::types::keyboard::*;
pub use self::types::output::*;
