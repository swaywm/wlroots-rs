//! This crate provides safe bindings to
//! [wlroots](https://github.com/swaywm/wlroots).
//!
//! Start your [Compositor](struct.Compositor.html) by implementing an [input
//! manager](manager/struct.InputManager.html) and an [output
//! manager](manager/struct.OutputManager.html) on two separate structs.
//!
//! # Example
//! ```rust,no_run
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
mod errors;
pub mod events;
pub mod types;
pub mod extensions;
pub mod render;
pub mod utils;

pub use self::compositor::{terminate, Compositor, CompositorBuilder};
pub use self::events::{key_events, pointer_events, touch_events, wl_shell_events,
                       xdg_shell_v6_events};
pub use self::manager::{InputManagerHandler, KeyboardHandler, OutputBuilder, OutputBuilderResult,
                        OutputHandler, OutputManagerHandler, PointerHandler, TouchHandler,
                        WlShellHandler, WlShellManagerHandler};
pub use self::types::area::*;
pub use self::types::cursor::*;
pub use self::types::input_device::*;
pub use self::types::keyboard::*;
pub use self::types::output::output::*;
pub use self::types::output::output_layout::*;
pub use self::types::pointer::*;
pub use self::types::seat::*;
pub use self::types::shell::*;
pub use self::types::surface::*;
pub use self::types::touch::*;
pub use key_events::Key;
pub use pointer_events::ButtonState;

pub use self::render::{matrix_identity, matrix_mul, matrix_rotate, matrix_scale, matrix_texture,
                       matrix_transform, matrix_translate, GenericRenderer, Renderer, Texture,
                       TextureFormat};

pub use self::errors::*;
