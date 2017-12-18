//! INI configuration reader for wlroots-rs examples.
//! Modeled after the ini reader from wlroots.

extern crate clap;
extern crate ini;
#[macro_use]
extern crate wlroots;
extern crate wlroots_sys;

use std::env;
use std::path::PathBuf;

use wlroots_sys::wl_output_transform;

use clap::{App, Arg};

use ini::Ini;

use wlroots::{Area, Origin, Size};

/// Main example configuration, holds the configuration sections for
/// outputs and devices.
pub struct ExampleConfig {
    pub config_path: PathBuf,
    pub cursor_mapped_output: String,
    pub cursor_mapped_box: Area,
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

impl OutputConfig {
    pub fn new(name: String) -> Self {
        OutputConfig { name,
                       transform: wl_output_transform::WL_OUTPUT_TRANSFORM_NORMAL,
                       x: 0,
                       y: 0 }
    }
}

/// Configuration section for devices.
pub struct DevConfig {
    pub name: String,
    pub mapped_output: String,
    pub mapped_box: Area
}

impl DevConfig {
    pub fn new(name: String) -> Self {
        DevConfig { name,
                    mapped_output: "TODO".into(),
                    mapped_box: Area::default() /* TODO */ }
    }
}

/// Parses the args using the provided Clap App.
///
/// The user of this function/lib should provide an `App` that details what the
/// example should do.
pub fn parse_args(app: App) -> Option<ExampleConfig> {
    use wl_output_transform::*;
    let matches = app.arg(Arg::with_name("config").short("c")
                                                  .long("config")
                                                  .value_name("FILE")
                                                  .help("Sets a custom config file")
                                                  .takes_value(true)).get_matches();
    let config_path: PathBuf =
        matches.value_of("config")
               .map(|path| path.into())
               .unwrap_or_else(|| env::current_dir().unwrap().join("wlr-example.ini"));
    let ini = match Ini::load_from_file(config_path.clone()) {
        Ok(ini) => ini,
        Err(err) => {
            wlr_log!(L_ERROR,
                     "Could not load config file because \"{}\"",
                     err.msg);
            wlr_log!(L_ERROR, "Defaulting to blank configuration");
            return None
        }
    };
    let mut config = ExampleConfig { config_path,
                                     cursor_mapped_output: "".into(),
                                     cursor_mapped_box: Area::default(),
                                     outputs: vec![],
                                     devs: vec![] };
    for (sec, prop) in ini.iter() {
        let sec = match *sec {
            Some(ref sec) => sec,
            None => continue
        };
        if sec.starts_with("output:") {
            let output_name = sec.split(":").skip(1)
                                 .next()
                                 .expect("Output section had no name following the colon");
            let output = match config.outputs
                                     .iter()
                                     .position(|output| output.name == output_name)
            {
                Some(index) => &mut config.outputs[index],
                None => {
                    config.outputs.push(OutputConfig::new(output_name.into()));
                    config.outputs.last_mut().unwrap()
                }
            };
            for (key, value) in prop.iter() {
                match key.as_str() {
                    "x" => output.x = value.parse().expect("Bad numerical val for x"),
                    "y" => output.y = value.parse().expect("Bad numerical val for y"),
                    "rotate" => {
                        match value.as_str() {
                            "90" => output.transform = WL_OUTPUT_TRANSFORM_90,
                            "180" => output.transform = WL_OUTPUT_TRANSFORM_180,
                            "270" => output.transform = WL_OUTPUT_TRANSFORM_270,
                            "flipped" => output.transform = WL_OUTPUT_TRANSFORM_FLIPPED,
                            "flipped-90" => output.transform = WL_OUTPUT_TRANSFORM_FLIPPED_90,
                            "flipped-180" => output.transform = WL_OUTPUT_TRANSFORM_FLIPPED_180,
                            "flipped-270" => output.transform = WL_OUTPUT_TRANSFORM_FLIPPED_270,
                            val => wlr_log!(L_ERROR, "Got unknown transform value: {}", val)
                        }
                    }
                    key => wlr_log!(L_ERROR, "Got unknown key \"{}\" in output section", key)
                }
            }
        } else if sec.starts_with("device:") {
            let device_name = sec.split(":").skip(1)
                                 .next()
                                 .expect("Device section had no name following the colon");
            let device = match config.devs
                                     .iter_mut()
                                     .position(|dev| dev.name == device_name)
            {
                Some(index) => &mut config.devs[index],
                None => {
                    config.devs.push(DevConfig::new(device_name.into()));
                    config.devs.last_mut().unwrap()
                }
            };
            for (key, value) in prop.iter() {
                match key.as_str() {
                    "map-to-output" => device.mapped_output = value.clone(),
                    "geometry" => {
                        device.mapped_box = parse_geometry(value).expect("Could not parse geometry")
                    }
                    val => wlr_log!(L_ERROR, "Got unknown device config: {}", val)
                }
            }
        } else if sec == "cursor" {
            for (key, value) in prop.iter() {
                match key.as_str() {
                    "map-to-output" => config.cursor_mapped_output = value.clone(),
                    "geometry" => {
                        config.cursor_mapped_box =
                            parse_geometry(value).expect("Could not parse geometry")
                    }
                    val => wlr_log!(L_ERROR, "Got unknown cursor config: {}", val)
                }
            }
        } else {
            wlr_log!(L_ERROR, "Got unknown config section: {}", sec)
        }
    }
    Some(config)
}

/// Parses geometry from the INI file.
///
/// Expected format: "{width}x{height}+{x}+{y}"
fn parse_geometry(input: &str) -> Option<Area> {
    let mut area = Area::default();
    if !(input.contains("x")) {
        wlr_log!(L_ERROR,
                 "Can't find 'x' separator in geometry: \"{}\"",
                 input);
        return None
    }
    let (width, height, x, y) = {
        let mut iter = input.split("+");
        let area_half = iter.next()?;
        let mut area_iter = area_half.split("x");
        let width = area_iter.next()?;
        let height = area_iter.next()?;
        // X and y are optional.
        let x = iter.next().and_then(|x| x.parse().ok());
        let y = iter.next().and_then(|y| y.parse().ok());
        (width.parse().ok()?, height.parse().ok()?, x, y)
    };
    area.width = width;
    area.height = height;
    if let Some(x) = x {
        area.x = x;
    }
    if let Some(y) = y {
        area.y = y;
    }
    Some(area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_geometry_parse() {
        let geometry = "800x600";
        let expected_area = Some(Area::new(Origin::default(),
                                           Size { width: 800,
                                                  height: 600 }));
        assert_eq!(expected_area, parse_geometry(geometry));
    }

    #[test]
    fn geometry_with_origin_parse() {
        let geometry = "800x600+256+127";
        let expected_area = Some(Area::new(Origin { x: 256, y: 127 },
                                           Size { width: 800,
                                                  height: 600 }));
        assert_eq!(expected_area, parse_geometry(geometry))
    }
}
