#![allow(non_camel_case_types, non_upper_case_globals)]

extern crate libc;
pub extern crate wayland_server;
#[macro_use]
pub extern crate wayland_sys;

// For graphical functions
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod generated {
    use libc;
    include!("gen.rs");

    // XXX: If you add another protocols, take a look at wayland_protocol! macro
    // from `wayland-rs/wayland-protocols/src/protocol_macro.rs`.
    pub mod protocols {
        pub mod server_decoration {
            #![allow(unused_imports)]
            pub mod server {
                mod interfaces {
                    pub(crate) use wayland_server::protocol_interfaces::wl_surface_interface;
                    include!(concat!(env!("OUT_DIR"), "/server_decoration_interfaces.rs"));
                }

                use wayland_server::{Client, EventLoopHandle, EventResult, Implementable,
                                     Liveness, Resource};
                use wayland_server::protocol::wl_surface;
                include!(concat!(env!("OUT_DIR"), "/server_decoration_server_api.rs"));
            }
        }
    }
}
pub use self::generated::*;

pub type wlr_output_events = self::generated::wlr_output__bindgen_ty_1;
pub type wlr_input_device_pointer = self::generated::wlr_input_device__bindgen_ty_1;

impl wl_output_transform {
    /// Invert the transform.
    pub fn invert(self) -> Self {
        unsafe { wlr_output_transform_invert(self) }
    }
}
