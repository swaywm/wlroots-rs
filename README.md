# wlroots-rs
[![Crates.io](https://img.shields.io/crates/v/wlroots.svg)](https://crates.io/crates/wlroots)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/wlroots-rs/)

Safe Rust bindings for [wlroots](https://github.com/SirCmpwn/wlroots).

This library is currently unversioned as wlroots is still unstable. In the future it will be versioned and released on crates.io. It will track the current version of wlroots, with additional versions for each Rust only change (what that will look like is still being decided).

# [Documentation](https://docs.rs/wlroots/)

# Building
To build wlroots-rs you have to init the wlroots submodule first and have all wlroots dependencies.

    git submodule update --init
    cargo build

If you don't want to compile against wlroots statically, add the `--no-default-features` flag.

If you want unstable wlroots features then add `--features=unstable`.

# Examples
See [the examples directory](https://github.com/swaywm/wlroots-rs/tree/master/examples) for basic examples using this library and at [Way Cooler the primary user of this library](https://github.com/way-cooler/way-cooler).

You can run an example using the following command:
```bash
cargo run --example <name of the example>
```
