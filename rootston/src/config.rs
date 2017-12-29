//! Parsing logic for the ini file format used in rootston.

use std::ffi::CStr;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;
use std::process::exit;
use std::str::{ParseBoolError, Utf8Error};

use self::wl_output_transform::*;
use wlroots::{Area, Key};
use wlroots::types::keyboard::KeyboardModifier;
use wlroots::utils::safe_as_cstring;
use wlroots::wlroots_sys::{wl_output_transform, xkb_keysym_from_name};
use wlroots::wlroots_sys::xkb_keysym_flags::XKB_KEYSYM_NO_FLAGS;
use wlroots::xkbcommon::xkb::keysyms::KEY_NoSymbol;

use clap::App;

use ini::{ini, Ini};

pub static DEFAULT_CONFIG_NAME: &'static str = "rootston.ini";

static OUTPUT_PREFIX: &'static str = "output:";
static DEVICE_PREFIX: &'static str = "device:";
static KEYBOARD_PREFIX: &'static str = "keyboard:";
static CURSOR_PREFIX: &'static str = "cursor:";

static DEFAULT_SEAT_NAME: &'static str = "seat0";

/// Complete configuration for rootston reference compositor.
#[derive(Debug, Clone, PartialEq)]
pub struct MainConfig {
    config_path: PathBuf,
    startup_cmd: Option<String>,
    xwayland: bool,
    outputs: Vec<OutputConfig>,
    devices: Vec<DeviceConfig>,
    bindings: Vec<BindingConfig>,
    keyboards: Vec<KeyboardConfig>,
    cursors: Vec<CursorConfig>
}

/// Configuration for an output in rootston.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputConfig {
    name: String,
    transform: wl_output_transform,
    x: i32,
    y: i32,
    scale: f32,
    mode: ModeConfig
}

/// Configuration for an output's mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModeConfig {
    width: i32,
    height: i32,
    refresh_rate: f32
}

/// Configuration for a generic device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeviceConfig {
    name: String,
    seat: Option<String>,
    mapped_output: Option<String>,
    tap_enabled: Option<bool>,
    mapped_box: Option<Area>
}

/// Configuration for a keyboard binding.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BindingConfig {
    modifiers: u32,
    keysyms: Vec<Key>,
    command: String
}

/// Configuration for a keyboard device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyboardConfig {
    name: String,
    seat: Option<String>,
    meta_key: Option<u32>,
    rules: Option<String>,
    model: Option<String>,
    layout: Option<String>,
    variant: Option<String>,
    options: Option<String>,
    repeat_rate: Option<i32>,
    repeat_delay: Option<i32>
}

/// Configuration for a cursor.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CursorConfig {
    seat: String,
    mapped_output: Option<String>,
    mapped_box: Option<Area>,
    theme: Option<String>,
    default_image: Option<String>
}

/// Possible error conditions from parsing configuration file.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseError {
    /// The file could not be found in the path.
    ConfigNotFound(PathBuf),
    /// Could not parse the contents of the file.
    /// Contains the reason why the parsing failed.
    BadParse(String)
}

impl From<ParseBoolError> for ParseError {
    fn from(err: ParseBoolError) -> Self {
        ParseError::BadParse(format!("Colud not parse as boolean: {:#?}", err))
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::BadParse(format!("Could not parse as int number: {:#?}", err))
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        ParseError::BadParse(format!("Could not parse as float number: {:#?}", err))
    }
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        ParseError::BadParse(format!("Could not convert value to utf8 string: {:#?}", err))
    }
}

pub type ParseResult = Result<MainConfig, ParseError>;

impl Default for ModeConfig {
    fn default() -> Self {
        ModeConfig { width: 0,
                     height: 0,
                     refresh_rate: 0.0 }
    }
}

impl DeviceConfig {
    pub fn new(name: String) -> Self {
        DeviceConfig { name,
                       seat: None,
                       mapped_output: None,
                       tap_enabled: None,
                       mapped_box: None }
    }
}

impl KeyboardConfig {
    pub fn new<S: Into<String>>(name: S) -> Self {
        KeyboardConfig { name: name.into(),
                         layout: None,
                         meta_key: None,
                         model: None,
                         options: None,
                         repeat_rate: None,
                         repeat_delay: None,
                         rules: None,
                         seat: None,
                         variant: None }
    }

    pub fn new_with_meta<S: Into<String>>(name: S, meta_key: u32) -> Self {
        KeyboardConfig { name: name.into(),
                         layout: None,
                         meta_key: Some(meta_key),
                         model: None,
                         options: None,
                         repeat_rate: None,
                         repeat_delay: None,
                         rules: None,
                         seat: None,
                         variant: None }
    }
}

impl CursorConfig {
    pub fn new(seat: String) -> Self {
        CursorConfig { seat,
                       mapped_output: None,
                       mapped_box: None,
                       theme: None,
                       default_image: None }
    }
}

impl OutputConfig {
    pub fn new(name: String) -> Self {
        OutputConfig { name,
                       transform: WL_OUTPUT_TRANSFORM_NORMAL,
                       scale: 1.0,
                       x: 0,
                       y: 0,
                       mode: ModeConfig::default() }
    }
}

impl BindingConfig {
    pub fn parse<S: Into<String>>(combination: S, command: S) -> Result<Self, ParseError> {
        let mut keysyms = Vec::new();
        let mut modifiers = 0;
        let combination = combination.into();
        let symnames = combination.split("+").map(|string| safe_as_cstring(string));
        for symname in symnames {
            let modifier = parse_modifier(symname.as_c_str());
            match modifier {
                0 => {
                    // FIXME Unsafe, though maybe this is such an edge case it shouldn't matter...
                    let sym = unsafe {
                        xkb_keysym_from_name(symname.as_ptr(), XKB_KEYSYM_NO_FLAGS)
                    };
                    if sym == KEY_NoSymbol {
                        return Err(ParseError::BadParse(format!("got unknown key binding \
                                                                 symbol: {:?}",
                                                                symname)))
                    }
                    keysyms.push(sym);
                }
                modifier => modifiers |= modifier
            }
        }
        Ok(BindingConfig { modifiers,
                           keysyms,
                           command: command.into() })
    }
}

impl MainConfig {
    pub fn parse_config(config_path: PathBuf, startup_cmd: Option<String>) -> ParseResult {
        let mut config = MainConfig { config_path: config_path.clone(),
                                      startup_cmd,
                                      xwayland: true,
                                      outputs: Vec::new(),
                                      devices: Vec::new(),
                                      bindings: Vec::new(),
                                      keyboards: Vec::new(),
                                      cursors: Vec::new() };
        let ini = Ini::load_from_file(config_path.clone())
            .map_err(|_| ParseError::ConfigNotFound(config_path))?;
        for (sec, prop) in ini.iter() {
            let sec = match *sec {
                Some(ref sec) => sec,
                None => continue
            };
            match sec.as_str() {
                "core" => {
                    for (key, value) in prop.iter() {
                        if key.as_str() == "xwayland" {
                            config.xwayland = value.parse()?;
                        } else {
                            wlr_log!(L_ERROR, "got unknown core config: {}", key);
                        }
                    }
                }
                "cursor" => config = parse_cursor(config, DEFAULT_SEAT_NAME, prop)?,
                "keyboard" => config = parse_keyboard(config, "", prop)?,
                "bindings" => {
                    for (key, value) in prop.iter() {
                        config.bindings
                              .push(BindingConfig::parse(key.clone(), value.clone())?)
                    }
                }
                sec => {
                    if sec.starts_with(OUTPUT_PREFIX) {
                        let output_name =
                            sec.split(":").skip(1)
                               .next()
                               .ok_or_else(|| ParseError::BadParse(format!("Bad section name")))?;
                        let output_config = match config.outputs
                                                        .iter()
                                                        .position(|output| {
                                                                      output.name == output_name
                                                                  }) {
                            Some(index) => &mut config.outputs[index],
                            None => {
                                config.outputs.push(OutputConfig::new(output_name.into()));
                                config.outputs.last_mut().unwrap()
                            }
                        };
                        for (key, value) in prop.iter() {
                            match key.as_str() {
                                "x" => output_config.x = value.parse()?,
                                "y" => output_config.y = value.parse()?,
                                "scale" => output_config.scale = value.parse()?,
                                "rotate" => {
                                    match value.as_str() {
                                        "90" => output_config.transform = WL_OUTPUT_TRANSFORM_90,
                                        "180" => output_config.transform = WL_OUTPUT_TRANSFORM_180,
                                        "270" => output_config.transform = WL_OUTPUT_TRANSFORM_270,
                                        "flipped" => {
                                            output_config.transform = WL_OUTPUT_TRANSFORM_FLIPPED
                                        }
                                        "flipped-90" => {
                                            output_config.transform = WL_OUTPUT_TRANSFORM_FLIPPED_90
                                        }
                                        "flipped-180" => {
                                            output_config.transform =
                                                WL_OUTPUT_TRANSFORM_FLIPPED_180
                                        }
                                        "flipped-270" => {
                                            output_config.transform =
                                                WL_OUTPUT_TRANSFORM_FLIPPED_270
                                        }
                                        val => {
                                            wlr_log!(L_ERROR,
                                                     "Got unknown transform value: {}",
                                                     val)
                                        }
                                    }
                                }
                                "mode" => {
                                    let (x, y, mode) = parse_mode_config(value)?;
                                    output_config.mode = mode;
                                    output_config.x = x;
                                    output_config.y = y;
                                }
                                key => wlr_log!(L_ERROR, "Unknown entry in output: {}", key)
                            }
                        }
                    } else if sec.starts_with(CURSOR_PREFIX) {
                        let cursor_name =
                            sec.split(":").skip(1)
                               .next()
                               .ok_or_else(|| ParseError::BadParse(format!("Bad section name")))?;
                        config = parse_cursor(config, cursor_name, prop)?;
                    } else if sec.starts_with(DEVICE_PREFIX) {
                        let device_name = sec.split(":").skip(1).next()
                            .ok_or_else(|| ParseError::BadParse(format!("Bad device section name")))?;
                        let device_config = match config.devices
                                                        .iter()
                                                        .position(|device| {
                                                                      device.name == device_name
                                                                  }) {
                            Some(index) => &mut config.devices[index],
                            None => {
                                config.devices
                                      .push(DeviceConfig::new(String::from(device_name)));
                                config.devices.last_mut().unwrap()
                            }
                        };
                        for (key, value) in prop.iter() {
                            match key.as_str() {
                                "map-to-output" => {
                                    device_config.mapped_output = Some(value.clone())
                                }
                                "geometry" => {
                                    device_config.mapped_box = Some(parse_geometry(value)?)
                                }
                                "seat" => device_config.seat = Some(value.clone()),
                                "tap_enabled" => device_config.tap_enabled = Some(value.parse()?),
                                name => wlr_log!(L_ERROR, "got unknown device config: {}", name)
                            }
                        }
                    } else if sec.starts_with(KEYBOARD_PREFIX) {
                        let device_name = sec.split(":").skip(1).next()
                            .ok_or_else(|| ParseError::BadParse(format!("Bad keyboard section name")))?;
                        config = parse_keyboard(config, device_name, prop)?;
                    } else {
                        wlr_log!(L_ERROR, "got unknown config section: {}", sec)
                    }
                }
            }
        }
        Ok(config)
    }

    pub fn sensible_defaults(config_path: PathBuf,
                             startup_cmd: Option<String>)
                             -> Result<Self, ParseError> {
        let keyboards =
            vec![KeyboardConfig::new_with_meta("", KeyboardModifier::WLR_MODIFIER_LOGO.bits())];
        let bindings = vec![BindingConfig::parse("Logo+Shift+E", "exit")?,
                            BindingConfig::parse("Ctrl+q", "close")?,
                            BindingConfig::parse("Alt+Tab", "next_window")?];
        Ok(MainConfig { config_path,
                        startup_cmd,
                        xwayland: true,
                        outputs: Vec::new(),
                        devices: Vec::new(),
                        keyboards,
                        cursors: Vec::new(),
                        bindings })
    }
}

pub fn roots_config_from_args(app: App) -> MainConfig {
    let matches = app.get_matches();
    let maybe_path: Option<PathBuf> = matches.value_of("config").map(|s| s.into());
    let config_path: PathBuf =
        maybe_path.unwrap_or_else(|| {
                                      use std::env::current_dir;
                                      match current_dir() {
                                          Ok(mut dir) => {
                                              dir.push(DEFAULT_CONFIG_NAME);
                                              dir
                                          }
                                          Err(err) => {
                                              wlr_log!(L_ERROR, "could not get cwd");
                                              wlr_log!(L_ERROR, "{:#?}", err);
                                              exit(1);
                                          }
                                      }
                                  });
    let startup_cmd = matches.value_of("command").map(|s| s.to_string());
    generate_config(config_path, startup_cmd)
}

pub fn generate_config(config_path: PathBuf, startup_cmd: Option<String>) -> MainConfig {
    match MainConfig::parse_config(config_path, startup_cmd.clone()) {
        Ok(config) => config,
        Err(ParseError::ConfigNotFound(config_path)) => {
            wlr_log!(L_DEBUG, "No config file found. Using sensible defaults.");
            MainConfig::sensible_defaults(config_path, startup_cmd).expect("Sensible defaults \
                                                                            were not so sensible!")
        }
        Err(ParseError::BadParse(config_path)) => {
            wlr_log!(L_ERROR, "Could not parse config file {:?}", config_path);
            exit(1);
        }
    }
}

fn parse_modifier(symname: &CStr) -> u32 {
    match symname.to_str().expect("Could not parse modifier") {
        "Shift" => KeyboardModifier::WLR_MODIFIER_SHIFT,
        "Ctrl" => KeyboardModifier::WLR_MODIFIER_CTRL,
        "Caps" => KeyboardModifier::WLR_MODIFIER_CAPS,
        "Alt" => KeyboardModifier::WLR_MODIFIER_ALT,
        "Mod2" => KeyboardModifier::WLR_MODIFIER_MOD2,
        "Mod3" => KeyboardModifier::WLR_MODIFIER_MOD3,
        "Logo" => KeyboardModifier::WLR_MODIFIER_LOGO,
        "Mod5" => KeyboardModifier::WLR_MODIFIER_MOD5,
        _ => KeyboardModifier::empty()
    }.bits()
}

/// Parses mode configuration from the INI file.
///
/// Expected format: "{width}x{height}+{x}+{y}"
fn parse_mode_config(input: &str) -> Result<(i32, i32, ModeConfig), ParseError> {
    let mut mode = ModeConfig::default();
    if !(input.contains("x")) {
        return Err(ParseError::BadParse(format!("Can't find 'x' separator \
                                                 in geometry: \"{}\"",
                                                input)))
    }
    let (width, height, x, y) = {
        let mut iter = input.split("+");
        let area_half = iter.next().ok_or_else(|| ParseError::BadParse(format!("Bad mode format: {:#?}", input)))?;
        let mut area_iter = area_half.split("x");
        let width =
            area_iter.next()
                     .ok_or_else(|| {
                                     ParseError::BadParse(format!("Bad mode format: {:#?}", input))
                                 })?;
        let height =
            area_iter.next()
                     .ok_or_else(|| {
                                     ParseError::BadParse(format!("Bad mode format: {:#?}", input))
                                 })?;
        // X and y are optional.
        let x = iter.next().and_then(|x| x.parse().ok());
        let y = iter.next().and_then(|y| y.parse().ok());
        (width.parse()?, height.parse()?, x, y)
    };
    mode.width = width;
    mode.height = height;
    Ok((x.unwrap_or(0), y.unwrap_or(0), mode))
}

/// Parses geometry from the INI file.
///
/// Expected format: "{width}x{height}+{x}+{y}"
fn parse_geometry(input: &str) -> Result<Area, ParseError> {
    let mut area = Area::default();
    if !(input.contains("x")) {
        return Err(ParseError::BadParse(format!("Can't find 'x' separator \
                                                 in geometry: \"{}\"",
                                                input)))
    }
    let (width, height, x, y) = {
        let mut iter = input.split("+");
        let area_half = iter.next().ok_or_else(|| {
                                                    ParseError::BadParse(format!("Bad geometry \
                                                                                  format: {:#?}",
                                                                                 input))
                                                })?;
        let mut area_iter = area_half.split("x");
        let width = area_iter.next().ok_or_else(|| {
                                                     ParseError::BadParse(format!("Bad geometry \
                                                                                   format: {:#?}",
                                                                                  input))
                                                 })?;
        let height = area_iter.next().ok_or_else(|| {
                                                      ParseError::BadParse(format!("Bad geometry \
                                                                                    format: {:#?}",
                                                                                   input))
                                                  })?;
        // X and y are optional.
        let x = iter.next().and_then(|x| x.parse().ok());
        let y = iter.next().and_then(|y| y.parse().ok());
        (width.parse()?, height.parse()?, x, y)
    };
    area.width = width;
    area.height = height;
    if let Some(x) = x {
        area.x = x;
    }
    if let Some(y) = y {
        area.y = y;
    }
    Ok(area)
}

fn parse_keyboard<S: Into<String>>(mut config: MainConfig,
                                   device_name: S,
                                   prop: &ini::Properties)
                                   -> Result<MainConfig, ParseError> {
    let device_name = device_name.into();
    {
        let keyboard_config = match config.keyboards
                                          .iter()
                                          .position(|keyboard| keyboard.name == device_name)
        {
            Some(index) => &mut config.keyboards[index],
            None => {
                config.keyboards.push(KeyboardConfig::new(device_name));
                config.keyboards.last_mut().unwrap()
            }
        };
        for (key, value) in prop.iter() {
            match key.as_str() {
                "meta-key" => {
                    keyboard_config.meta_key =
                        Some(parse_modifier(safe_as_cstring(value.clone()).as_c_str()))
                }
                "rules" => keyboard_config.rules = Some(value.clone()),
                "model" => keyboard_config.model = Some(value.clone()),
                "layout" => keyboard_config.layout = Some(value.clone()),
                "variant" => keyboard_config.variant = Some(value.clone()),
                "options" => keyboard_config.options = Some(value.clone()),
                "repeate-rate" => keyboard_config.repeat_rate = Some(value.parse()?),
                "repeat-delay" => keyboard_config.repeat_delay = Some(value.parse()?),
                name => wlr_log!(L_ERROR, "got unknown keyboard config: {}", name)
            }
        }
    }
    Ok(config)
}

fn parse_cursor<S: Into<String>>(mut config: MainConfig,
                                 seat_name: S,
                                 prop: &ini::Properties)
                                 -> Result<MainConfig, ParseError> {
    let seat_name = seat_name.into();
    {
        let cursor_config = match config.cursors
                                        .iter()
                                        .position(|cursor| cursor.seat == seat_name)
        {
            Some(index) => &mut config.cursors[index],
            None => {
                config.cursors.push(CursorConfig::new(seat_name));
                config.cursors.last_mut().unwrap()
            }
        };
        for (key, value) in prop.iter() {
            match key.as_str() {
                "map-to-output" => cursor_config.mapped_output = Some(value.clone()),
                "geometry" => cursor_config.mapped_box = Some(parse_geometry(value)?),
                "theme" => cursor_config.theme = Some(value.clone()),
                "default-image" => cursor_config.default_image = Some(value.clone()),
                name => wlr_log!(L_ERROR, "got unknown cursor config: {}", name)
            }
        }
    }
    Ok(config)
}

#[cfg(test)]
mod test {
    static DEFAULT_CONFIG_PATH: &'static str = "../rootston.ini.example";
    use ::*;
    use std::ffi::CStr;
    use wlroots::wlroots_sys::xkb_keysym_flags::XKB_KEYSYM_NO_FLAGS;
    use wlroots::wlroots_sys::xkb_keysym_from_name;

    #[test]
    fn fallback_test() {
        let keyboards = vec![
            config::KeyboardConfig {
                name: "".into(),
                layout: None,
                meta_key: Some(config::KeyboardModifier::WLR_MODIFIER_LOGO.bits()),
                model: None,
                options: None,
                repeat_delay: None,
                repeat_rate: None,
                rules: None,
                seat: None,
                variant: None
            }
        ];
        let bindings = unsafe {
            vec![
                config::BindingConfig {
                    command: "exit".into(),
                    keysyms: vec![xkb_keysym_from_name(c_str!("E"), XKB_KEYSYM_NO_FLAGS)],
                    modifiers: config::parse_modifier(CStr::from_ptr(c_str!("Logo")))
                        | config::parse_modifier(CStr::from_ptr(c_str!("Shift")))
                },
                config::BindingConfig {
                    command: "close".into(),
                    keysyms: vec![xkb_keysym_from_name(c_str!("q"), XKB_KEYSYM_NO_FLAGS)],
                    modifiers: config::parse_modifier(CStr::from_ptr(c_str!("Ctrl")))
                },
                config::BindingConfig {
                    command: "next_window".into(),
                    keysyms: vec![xkb_keysym_from_name(c_str!("Tab"), XKB_KEYSYM_NO_FLAGS)],
                    modifiers: config::parse_modifier(CStr::from_ptr(c_str!("Alt")))
                }]
        };
        let expected = config::MainConfig { config_path: "./".into(),
                                            startup_cmd: None,
                                            xwayland: true,
                                            outputs: Vec::new(),
                                            devices: Vec::new(),
                                            bindings,
                                            keyboards,
                                            cursors: Vec::new() };
        assert_eq!(config::generate_config("./".into(), None), expected)
    }

    #[test]
    fn fallback_from_file_test() {
        let keyboards = vec![
            config::KeyboardConfig {
                name: "".into(),
                layout: None,
                meta_key: Some(config::KeyboardModifier::WLR_MODIFIER_LOGO.bits()),
                model: None,
                options: None,
                repeat_delay: None,
                repeat_rate: None,
                rules: None,
                seat: None,
                variant: None
            }
        ];
        let bindings = unsafe {
            vec![
                config::BindingConfig {
                    command: "exit".into(),
                    keysyms: vec![xkb_keysym_from_name(c_str!("E"), XKB_KEYSYM_NO_FLAGS)],
                    modifiers: config::parse_modifier(CStr::from_ptr(c_str!("Logo")))
                        | config::parse_modifier(CStr::from_ptr(c_str!("Shift")))
                },
                config::BindingConfig {
                    command: "close".into(),
                    keysyms: vec![xkb_keysym_from_name(c_str!("q"), XKB_KEYSYM_NO_FLAGS)],
                    modifiers: config::parse_modifier(CStr::from_ptr(c_str!("Ctrl")))
                },
                config::BindingConfig {
                    command: "next_window".into(),
                    keysyms: vec![xkb_keysym_from_name(c_str!("Tab"), XKB_KEYSYM_NO_FLAGS)],
                    modifiers: config::parse_modifier(CStr::from_ptr(c_str!("Alt")))
                }]
        };
        let expected = config::MainConfig { config_path: DEFAULT_CONFIG_PATH.into(),
                                            startup_cmd: None,
                                            xwayland: true,
                                            outputs: Vec::new(),
                                            devices: Vec::new(),
                                            bindings,
                                            keyboards,
                                            cursors: Vec::new() };
        let actual = config::generate_config(DEFAULT_CONFIG_PATH.into(), None);
        assert_eq!(actual, expected)
    }
}
