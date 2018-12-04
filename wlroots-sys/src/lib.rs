#![allow(non_camel_case_types, non_upper_case_globals)]

extern crate libc;
pub extern crate wayland_commons;
pub extern crate wayland_server;
#[macro_use]
pub extern crate wayland_sys;

pub use wayland_sys::{*, pid_t, gid_t, uid_t, server::{self, WAYLAND_SERVER_HANDLE}};

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod generated {
    use libc;
    include!("gen.rs");

    // XXX: If you add another protocols, take a look at wayland_protocol! macro
    // from `wayland-rs/wayland-protocols/src/protocol_macro.rs`.
    pub mod protocols {
        pub mod server_decoration {
            #![allow(unused_imports)]
            #![allow(unused_variables)]
            mod c_interfaces {
                use wayland_server::sys::protocol_interfaces::wl_surface_interface;
                include!(concat!(env!("OUT_DIR"), "/server_decoration_interfaces.rs"));
            }

            pub mod server {
                pub(crate) use wayland_server::{NewResource, Resource};
                pub(crate) use wayland_commons::{AnonymousObject, Interface, MessageGroup,
                                                 wire::{Argument, ArgumentType, Message, MessageDesc},
                                                 map::{Object, ObjectMetadata}};
                pub(crate) use wayland_sys as sys;
                use wayland_server::{*, protocol::wl_surface};
                use wayland_sys::common::{wl_interface, wl_argument};
                include!(concat!(env!("OUT_DIR"), "/server_decoration_server_api.rs"));
            }
        }
    }
}
pub use self::generated::*;

#[cfg(unstable_features)]
pub type wlr_output_events = self::generated::wlr_output__bindgen_ty_1;
#[cfg(unstable_features)]
pub type wlr_input_device_pointer = self::generated::wlr_input_device__bindgen_ty_1;

#[cfg(unstable_features)]
impl wl_output_transform {
    /// Returns the transform that, when composed with `self`, gives
    /// `WL_OUTPUT_TRANSFORM_NORMAL`.
    pub fn invert(self) -> Self {
        unsafe { wlr_output_transform_invert(self) }
    }

    /// Returns a transform that, when applied, has the same effect as applying
    /// sequentially `self` and `other`.
    pub fn compose(self, other: Self) -> Self {
        unsafe { wlr_output_transform_compose(self, other) }
    }
}
