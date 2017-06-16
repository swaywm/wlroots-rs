extern crate wlroots;

use std::ptr::null_mut;
use std::env;
use std::time::Instant;
use std::os::raw::{c_void, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

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
    {
        let state = Mutex::new(State::new());
        // NOTE that because we pass `done` to a timeout callback,
        // it must be declared before session to ensure they are dropped
        // in the correct order.
        // Try reordering `done` to after `session` and see the error.
        let done = AtomicBool::new(false);
        let mut session = wlroots::Session::new()
            .expect("Could not start session");
        // init output
        wlroots::output::init(&mut session);
        // set loop to break after 20 seconds.
        session.set_timeout(&done,
                            |done: &AtomicBool| {
                                done.store(true, Ordering::Relaxed)
                            },
                            20000);
        // Set the outputs to turn off at 5 seconds
        session.set_timeout(&state,
                            disable_outputs,
                            5000);
        // Set the outputs to turn on at 5 seconds
        session.set_timeout(&state,
                            enable_outputs,
                            10000);
        // Finish initializing the backend
        session.backend.init().expect("Backend could not initalize");
        while ! done.load(Ordering::Relaxed) {
            match session.dispatch_event_loop() {
                0 => {}
                err_code => {
                    println!("Error: {:?}", err_code);
                    break;
                }
            }
        }
        // TODO Ensure that this all cleaned up properly in RAII
    }

}


fn disable_outputs(_state: &Mutex<State>) {
    let mut outputs = wlroots::output::OUTPUTS.try_lock()
        .expect("Output lock was already acquired");
    for mut output in &mut *outputs {
        output.disable()
    }
}

fn enable_outputs(_state: &Mutex<State>) {
    let mut outputs = wlroots::output::OUTPUTS.try_lock()
    .expect("Output lock was already acquired");
    for mut output in &mut *outputs {
        output.enable()
    }
}
