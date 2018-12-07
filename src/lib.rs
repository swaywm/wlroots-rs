//! This crate provides safe bindings to
//! [wlroots](https://github.com/swaywm/wlroots).
#![cfg_attr(not(feature = "unstable"), allow(unused_imports, unused_macros))]

#![allow(unused_unsafe)]
#[macro_use]
extern crate bitflags;
extern crate vsprintf;
#[macro_use]
pub extern crate wlroots_sys;
extern crate wlroots_dehandle;
#[cfg(feature = "unstable")]
pub extern crate xkbcommon;

#[cfg(feature = "unstable")]
pub use wlroots_dehandle::wlroots_dehandle;
pub(crate) use wlroots_sys::wayland_sys;
pub(crate) use wlroots_sys::libc;

#[macro_use]
mod macros;
#[cfg(feature = "unstable")]
pub(crate) mod manager;
#[cfg(feature = "unstable")]
pub mod compositor;
#[cfg(feature = "unstable")]
mod errors;
#[cfg(feature = "unstable")]
pub(crate) mod events;
mod types;
#[cfg(feature = "unstable")]
pub mod extensions;
#[cfg(feature = "unstable")]
pub mod render;
pub mod utils;
#[cfg(feature = "unstable")]
pub mod xwayland;
#[cfg(feature = "unstable")]
pub mod backend;

pub use types::*;

#[cfg(feature = "unstable")]
pub use wlroots_sys::{wlr_keyboard_modifiers as KeyboardModifiers,
                      wlr_tablet_tool_axes as TabletToolAxes,
                      wl_shm_format::{self, *},
                      wlr_axis_orientation::{self, *}, wlr_axis_source::{self, *},
                      wlr_button_state::{self, *}, wlr_input_device_type::{self, *},
                      wlr_key_state::{self, *}, wlr_keyboard_modifier::{self, *},
                      wlr_tablet_pad_ring_source::{self, *},
                      wlr_tablet_pad_strip_source::{self, *},
                      wlr_tablet_tool_proximity_state::{self, *}};

#[cfg(feature = "unstable")]
pub use self::errors::*;

