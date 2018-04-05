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
pub use self::events::{key_events, pointer_events, seat_events, tablet_pad_events,
                       tablet_tool_events, touch_events, wl_shell_events, xdg_shell_v6_events};
pub use self::manager::{InputManagerHandler, KeyboardHandler, OutputBuilder, OutputBuilderResult,
                        OutputHandler, OutputManagerHandler, PointerHandler, TabletPadHandler,
                        TabletToolHandler, TouchHandler, WlShellHandler, WlShellManagerHandler,
                        XdgV6ShellHandler, XdgV6ShellManagerHandler};
pub use self::types::area::*;
pub use self::types::cursor::*;
pub use self::types::data_device::*;
pub use self::types::input::*;
pub use self::types::output::*;
pub use self::types::seat::*;
pub use self::types::shell::*;
pub use self::types::surface::*;
pub use key_events::Key;
pub use wlroots_sys::{wlr_keyboard_modifiers, wlr_tablet_tool_axes, wl_shm_format::{self, *},
                      wlr_axis_orientation::{self, *}, wlr_axis_source::{self, *},
                      wlr_button_state::{self, *}, wlr_input_device_type::{self, *},
                      wlr_key_state::{self, *}, wlr_keyboard_modifier::{self, *},
                      wlr_tablet_pad_ring_source::{self, *},
                      wlr_tablet_pad_strip_source::{self, *},
                      wlr_tablet_tool_proximity_state::{self, *}};

pub use self::render::{matrix_identity, matrix_multiply, matrix_projection, matrix_rotate,
                       matrix_scale, matrix_transform, matrix_translate, matrix_transpose,
                       project_box, GenericRenderer, Image, Renderer, Texture, TextureFormat};

pub use self::errors::*;

pub type HandleResult<T> = Result<T, UpgradeHandleErr>;
