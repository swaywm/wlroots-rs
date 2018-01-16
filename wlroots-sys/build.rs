extern crate bindgen;
extern crate gcc;
extern crate gl_generator;
#[cfg(feature = "static")]
extern crate meson;
extern crate wayland_scanner;

use gl_generator::{Api, Fallbacks, Profile, Registry, StaticGenerator};
use std::env;
use std::fs::File;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};

fn main() {
    meson();
    let target_dir = env::var("OUT_DIR").expect("$OUT_DIR not set!");
    let generated = bindgen::builder()
        .derive_debug(true)
        .derive_default(true)
        .generate_comments(true)
        .header("src/wlroots.h")
        .whitelisted_type(r"^wlr_.*$")
        .whitelisted_type(r"^xkb_.*$")
        .whitelisted_type(r"^XKB_.*$")
        .whitelisted_function(r"^_?wlr_.*$")
        .whitelisted_function(r"^xkb_.*$")
        .ctypes_prefix("libc")
        .clang_arg("-Iwlroots/include")
        .clang_arg("-Iwlroots/include/wlr")
        // NOTE Necessary because they use the out directory to put
        // pragma information on what features are available in a header file
        // titled "config.h"
        .clang_arg(format!("-I{}{}", target_dir, "/include/"))
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

    generate_protocols();

    // Example Khronos building stuff
    let mut file = File::create(&Path::new(&target_dir).join("bindings.rs")).unwrap();
    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
}

#[cfg(not(feature = "static"))]
fn meson() {}

#[cfg(feature = "static")]
fn meson() {
    let build_path =
        PathBuf::from(env::var("OUT_DIR").expect("Could not get OUT_DIR env variable"));
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
        println!("cargo:rustc-link-search=native={}/protocol/",
                 build_path_str);
        println!("cargo:rustc-link-search=native={}/xcursor/", build_path_str);
        println!("cargo:rustc-link-search=native={}/xwayland/",
                 build_path_str);
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

fn generate_protocols() {
    let output_dir_str = env::var("OUT_DIR").unwrap();

    let output_dir = Path::new(&output_dir_str);

    let protocols = &[("./wlroots/protocol/server-decoration.xml", "server_decoration")];

    for protocol in protocols {
        wayland_scanner::generate_code(protocol.0,
                                       output_dir.join(format!("{}_server_api.rs", protocol.1)),
                                       wayland_scanner::Side::Server);
        wayland_scanner::generate_code(protocol.0,
                                       output_dir.join(format!("{}_client_api.rs", protocol.1)),
                                       wayland_scanner::Side::Client);
        wayland_scanner::generate_interfaces(protocol.0,
                                             output_dir.join(format!("{}_interfaces.rs",
                                                                     protocol.1)));
    }
}
