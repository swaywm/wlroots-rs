//! INI configuration reader for wlroots-rs examples.
//! Modeled after the ini reader from wlroots.

extern crate ini;
extern crate wlroots;
extern crate wlroots_sys;

use std::path::PathBuf;

use wlroots_sys::wl_output_transform;

use wlroots::types::Area;

/// Main example configuration, holds the configuration sections for
/// outputs and devices.
pub struct ExampleConfig {
    pub config_path: PathBuf,
    pub mapped_output: String,
    pub mapped_box: Area,
    pub outputs: Vec<OutputConfig>,
    pub devs: Vec<DevConfig>
}

/// Configuration section for outputs.
pub struct OutputConfig {
    pub name: String,
    pub transform: wl_output_transform,
    pub x: i32,
    pub y: i32
}

/// Configuration section for devices.
pub struct DevConfig {
    pub name: String,
    pub mapped_output: String,
    pub mapped_box: Area
}

pub fn parse_args(args: &[&str]) -> Option<ExampleConfig> {
    unimplemented!()
}
