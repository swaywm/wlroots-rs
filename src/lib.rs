//! This crate provides safe bindings to
//! [wlroots](https://github.com/swaywm/wlroots).
//!
//! Start your [Compositor](struct.Compositor.html) by implementing an [input
//! manager](manager/struct.InputManager.html) and an [output
//! manager](manager/struct.OutputManager.html) on two separate structs.

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

pub use self::compositor::{terminate, Compositor, CompositorBuilder, CompositorHandler};
pub use self::events::{key_events, pointer_events, seat_events, touch_events, wl_shell_events,
                       xdg_shell_v6_events};
pub use self::manager::{InputManagerHandler, KeyboardHandler, OutputBuilder, OutputBuilderResult,
                        OutputHandler, OutputManagerHandler, PointerHandler, TouchHandler,
                        WlShellHandler, WlShellManagerHandler, XdgV6ShellHandler,
                        XdgV6ShellManagerHandler};
pub use self::types::area::*;
pub use self::types::cursor::*;
pub use self::types::data_device::*;
pub use self::types::input::*;
pub use self::types::output::*;
pub use self::types::seat::*;
pub use self::types::shell::*;
pub use self::types::surface::*;
pub use key_events::Key;
pub use wlroots_sys::{wlr_button_state, wlr_key_state, wlr_keyboard_modifiers};

pub use self::render::{matrix_identity, matrix_multiply, matrix_projection, matrix_rotate,
                       matrix_scale, matrix_transform, matrix_translate, matrix_transpose,
                       project_box, GenericRenderer, Image, Renderer, Texture, TextureFormat};

pub use self::errors::*;
