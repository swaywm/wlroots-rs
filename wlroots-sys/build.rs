extern crate bindgen;
#[cfg(feature = "static")]
extern crate meson;
extern crate gl_generator;

use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use gl_generator::{Registry, Api, Profile, Fallbacks, StaticGenerator};

// TODO these are wrong
static LIBRARIES: &'static [&'static str] =
    &["wlr-common", "wlr-backend", "wlr-session", "wlr-types"];

fn main() {
    let generated = bindgen::builder()
        .derive_debug(true)
        .generate_comments(true)
        .header("src/wlroots.h")
        .whitelisted_type(r"^wlr_.*$")
        .whitelisted_function(r"^wlr_.*$")
        .ctypes_prefix("libc")
        .clang_arg("-Iwlroots/include")
        .clang_arg("-Iwlroots/include/wlr")
        .clang_arg("-Iwlroots/include/xcursor")
        .clang_arg("-I/usr/include/pixman-1")
        .generate().unwrap();

    if cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=dylib=wayland-server");
        println!("cargo:rustc-link-lib=dylib=EGL");
        println!("cargo:rustc-link-lib=dylib=GL");
        println!("cargo:rustc-link-lib=dylib=gbm");
        println!("cargo:rustc-link-lib=dylib=drm");
        println!("cargo:rustc-link-lib=dylib=input");
        println!("cargo:rustc-link-lib=dylib=udev");
        println!("cargo:rustc-link-lib=dylib=systemd");
        println!("cargo:rustc-link-lib=dylib=dbus-1");
        println!("cargo:rustc-link-lib=dylib=pixman");
    } else {
        for library in LIBRARIES {
            println!("cargo:rustc-link-lib=dylib={}", library);
        }
    }

    // generate the bindings
    generated.write_to_file("src/gen.rs").unwrap();

    meson();

    // Example Khronos building stuff
    // TODO Put behind feature flag?
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();
    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
}

#[cfg(not(feature = "static"))]
fn meson() {}

#[cfg(feature = "static")]
fn meson() {
    let build_path = PathBuf::from(env::var("OUT_DIR")
        .expect("Could not get OUT_DIR env variable"));
    build_path.join("build");
    let build_path_str = build_path.to_str()
        .expect("Could not turn build path into a string");
    println!("cargo:rustc-link-search=native=wlroots");
    println!("cargo:rustc-link-search=native={}/lib", build_path_str);
    println!("cargo:rustc-link-search=native={}/lib64", build_path_str);
    println!("cargo:rustc-link-search=native={}/build/", build_path_str);

    meson::build("wlroots", build_path_str);
}
