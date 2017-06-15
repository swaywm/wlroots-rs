extern crate wlroots_sys;
#[macro_use] extern crate wayland_sys;
extern crate wlroots;

use std::ptr::null_mut;
use std::env;
use std::time::Instant;
use std::os::raw::{c_void, c_int};


struct State {
    /// The color on the screen.
    color: [f32; 3],
    dec: i32,
    /// How long since the last frame was rendered.
    last_frame: Instant,
}

impl State {
    fn new() -> Self {
        State {
            color: [1.0, 0.0, 0.0],
            dec: 0,
            last_frame: Instant::now()
        }
    }
}


fn main() {
    if env::var("DISPLAY").is_ok() {
        panic!("Detected that X is running. Run this in its own virtual terminal.")
    } else if env::var("WAYLAND_DISPLAY").is_ok() {
        panic!("Detected that Wayland is running. Run this in its own virtual terminal")
    }
    let mut session = wlroots::Session::new()
        .expect("Could not start session");
    unsafe {
        wlroots::output::init(&mut session);
    }
    session.backend.init().expect("Backend could not initalize");
    //loop {
        match session.dispatch_event_loop() {
            0 => {}
            err_code => {
                println!("Error: {:?}", err_code);
            }
        }
   // }
    // TODO Ensure that this all cleaned up properly in RAII

}
