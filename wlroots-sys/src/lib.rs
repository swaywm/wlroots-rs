#![allow(non_camel_case_types, non_upper_case_globals)]

extern crate libc;

// For graphical functions
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod generated {
    use libc;
    include!("gen.rs");
}
pub use self::generated::*;
