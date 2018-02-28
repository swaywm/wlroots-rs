# wlroots-rs
[![Gitter](https://badges.gitter.im/way-cooler/way-cooler.svg)](https://gitter.im/way-cooler/way-cooler?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![Crates.io](https://img.shields.io/crates/v/wlroots.svg)](https://crates.io/crates/wlroots)
[![Build Status](https://travis-ci.org/swaywm/wlroots-rs.svg?branch=master)](https://travis-ci.org/swaywm/wlroots-rs/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/wlroots-rs/)

Safe Rust bindings for [wlroots](https://github.com/SirCmpwn/wlroots).

# [Documentation](https://docs.rs/wlroots/)

# Building
To build wlroots-rs you have to init the wlroots submodule first

    git submodule update --init
    cargo build

# Examples
See [the examples directory](https://github.com/swaywm/wlroots-rs/tree/master/examples) for basic examples using this library and at [rootston, our clone of the wlroots reference compositor for a more comprehensive example](https://github.com/swaywm/wlroots-rs/tree/master/rootston)
