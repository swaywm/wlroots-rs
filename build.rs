//! This is only for the example
//! Should probably eventually be removed

extern crate gl_generator;

use gl_generator::{Api, Fallbacks, Profile, Registry, StaticGenerator};

use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    // Example Khronos building stuff.
    // TODO Put behind feature flag.
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
}
