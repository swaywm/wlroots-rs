# wlroots-rs
[![Crates.io](https://img.shields.io/crates/v/wlroots.svg)](https://crates.io/crates/wlroots)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/wlroots-rs/)

Safe Rust bindings for [wlroots](https://github.com/SirCmpwn/wlroots).

# [Documentation](https://docs.rs/wlroots/)

# Building
To build wlroots-rs you have to init the wlroots submodule first and have all wlroots dependencies.

    git submodule update --init
    cargo build

If you don't want to compile against wlroots statically, add the `--no-default-features` flag.

# Examples
See [the examples directory](https://github.com/swaywm/wlroots-rs/tree/master/examples) for basic examples using this library and at [Way Cooler the primary user of this library](https://github.com/way-cooler/way-cooler).
