extern crate gcc;
extern crate bindgen;
#[cfg(feature = "static")]
extern crate meson;
extern crate gl_generator;

use std::env;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
use std::fs::File;
use gl_generator::{Registry, Api, Profile, Fallbacks, StaticGenerator};

fn main() {
    let generated = bindgen::builder()
        .derive_debug(true)
        .derive_default(true)
        .generate_comments(true)
        .header("src/wlroots.h")
        .whitelisted_type(r"^wlr_.*$")
        .whitelisted_type(r"^xkb.*$")
        .whitelisted_function(r"^wlr_.*$")
        .ctypes_prefix("libc")
        .clang_arg("-Iwlroots/include")
        .clang_arg("-Iwlroots/include/wlr")
        .clang_arg("-Iwlroots/include/xcursor")
        .clang_arg("-I/usr/include/pixman-1")
        // Work around bug https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .hide_type("FP_NAN")
        .hide_type("FP_INFINITE")
        .hide_type("FP_ZERO")
        .hide_type("FP_SUBNORMAL")
        .hide_type("FP_NORMAL")
        .generate().unwrap();

    println!("cargo:rustc-link-lib=dylib=X11");
    println!("cargo:rustc-link-lib=dylib=X11-xcb");
    println!("cargo:rustc-link-lib=dylib=xkbcommon");
    println!("cargo:rustc-link-lib=dylib=xcb");
    println!("cargo:rustc-link-lib=dylib=cap");
    println!("cargo:rustc-link-lib=dylib=wayland-egl");
    println!("cargo:rustc-link-lib=dylib=wayland-client");
    println!("cargo:rustc-link-lib=dylib=wayland-server");
    println!("cargo:rustc-link-lib=dylib=EGL");
    println!("cargo:rustc-link-lib=dylib=GL");
    println!("cargo:rustc-link-lib=dylib=gbm");
    println!("cargo:rustc-link-lib=dylib=drm");
    println!("cargo:rustc-link-lib=dylib=input");
    println!("cargo:rustc-link-lib=dylib=udev");
    println!("cargo:rustc-link-lib=dylib=systemd");
    println!("cargo:rustc-link-lib=dylib=dbus-1");
    println!("cargo:rustc-link-lib=dylib=pixman-1");

    if !cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=dylib=wlroots");
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    }

    // generate the bindings
    generated.write_to_file("src/gen.rs").unwrap();

    meson();

    if cfg!(feature = "example") {

        // Example Khronos building stuff
        let dest = env::var("OUT_DIR").unwrap();
        let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();
        Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
            .write_bindings(StaticGenerator, &mut file)
            .unwrap();

        // Build share.d for examples
        let example_generated = bindgen::builder()
            .derive_debug(true)
            .derive_default(true)
            .generate_comments(true)
            .header("wlroots/examples/shared.h")
            .clang_arg("-Iwlroots/include")
            .clang_arg("-Iwlroots/include/wlr")
            .clang_arg("-Iwlroots/include/xcursor")
            .clang_arg("-I/usr/include/pixman-1")
            // Work around bug https://github.com/rust-lang-nursery/rust-bindgen/issues/687
            .hide_type("FP_NAN")
            .hide_type("FP_INFINITE")
            .hide_type("FP_ZERO")
            .hide_type("FP_SUBNORMAL")
            .hide_type("FP_NORMAL")
            .generate().unwrap();
        example_generated.write_to_file(format!("{}/shared.rs", dest)).unwrap();

        // Build shared.c for examples
        let mut config = gcc::Build::new();
        config.flag("-Wall");
        config.flag("-Wpedantic");
        config.flag("-Iwlroots/include");
        config.flag("-Iwlroots/include/wlr");
        config.file("wlroots/examples/shared.c");
        config.compile("libshared.a");

        // Link against libpam
        println!("cargo:rustc-flags=-l shared")
    }
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
    if cfg!(feature = "static") {
        println!("cargo:rustc-link-search=native={}/util/", build_path_str);
        println!("cargo:rustc-link-search=native={}/types/", build_path_str);
        println!("cargo:rustc-link-search=native={}/protocol/", build_path_str);
        println!("cargo:rustc-link-search=native={}/xcursor/", build_path_str);
        println!("cargo:rustc-link-search=native={}/xwayland/", build_path_str);
        println!("cargo:rustc-link-search=native={}/backend/", build_path_str);
        println!("cargo:rustc-link-search=native={}/render/", build_path_str);

        println!("cargo:rustc-link-lib=static=wlr_util");
        println!("cargo:rustc-link-lib=static=wlr_types");
        println!("cargo:rustc-link-lib=static=wlr_xcursor");
        println!("cargo:rustc-link-lib=static=wlr_xwayland");
        println!("cargo:rustc-link-lib=static=wlr_backend");
        println!("cargo:rustc-link-lib=static=wlr_render");
        // wayland protocols
        println!("cargo:rustc-link-lib=static=wl_protos");
    }

    meson::build("wlroots", build_path_str);
}
