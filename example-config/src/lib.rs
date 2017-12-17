//! INI configuration reader for wlroots-rs examples.
//! Modeled after the ini reader from wlroots.

extern crate ini;
extern crate wlroots;
extern crate wlroots_sys;

use wlroots_sys::wl_output_transform;

//use wlroots::Area;

pub struct OutputConfig {
    name: String,
    transform: wl_output_transform,
    x: i32,
    y: i32
}

pub struct DevConfig {
    name: String,
    mapped_output: String,
    //mapped_box: Area
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
