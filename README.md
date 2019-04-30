# wlroots-rs
[![Crates.io](https://img.shields.io/crates/v/wlroots.svg)](https://crates.io/crates/wlroots)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/wlroots-rs/)

**This is no longer maintained.** As an alternative consider
[smithay](https://github.com/Smithay/smithay).

Safe Rust bindings for [wlroots](https://github.com/SirCmpwn/wlroots).

This library is currently tracking the wlroots version for its minor version. Patch versions are wlroots-rs specific.

# [Documentation](http://way-cooler.org/docs/wlroots/index.html)

# Building
To build wlroots-rs you have to init the wlroots submodule first and have all wlroots dependencies.

    git submodule update --init
    cargo build

If you want to compile against wlroots statically, add the `"static"` flag.

If you want use unstable wlroots features then add the `"unstable"` flag.

# Examples
See [the examples directory](https://github.com/swaywm/wlroots-rs/tree/master/examples) for basic examples.

You can run an example using the following command:
```bash
cargo run --example <name of the example>
```
