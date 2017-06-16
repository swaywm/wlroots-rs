#[macro_use] extern crate wlroots;

use std::ptr::null_mut;
use std::env;
use std::time::Instant;
use std::os::raw::{c_void, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;


fn main() {
    if env::var("DISPLAY").is_ok() {
        panic!("Detected that X is running. Run this in its own virtual terminal.")
    } else if env::var("WAYLAND_DISPLAY").is_ok() {
        panic!("Detected that Wayland is running. Run this in its own virtual terminal")
    }
    let empty = ();
    {
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
        session.set_timeout(&empty,
                            disable_outputs,
                            5000);
        // Set the outputs to turn on at 5 seconds
        session.set_timeout(&empty,
                            enable_outputs,
                            10000);
        // Finish initializing the backend
        session.backend.init().expect("Backend could not initalize");
        // Now that the outputs are registered add a callback per frame.
        unsafe {
            add_frame_callbacks();
        }
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

fn disable_outputs(_: &()) {
    let mut outputs = wlroots::output::OUTPUTS.try_lock()
        .expect("Output lock was already acquired");
    for mut output in &mut *outputs {
        output.disable()
    }
}

fn enable_outputs(_: &()) {
    let mut outputs = wlroots::output::OUTPUTS.try_lock()
    .expect("Output lock was already acquired");
    for mut output in &mut *outputs {
        output.enable()
    }
}

/*
NOTE: This is the only unsafe part of the program.

This draws directly to the screen, but the majority of the unsafe part is
unexposed machinery. We don't wrap this well, because in a real program
we will probably not need to do this.
*/
extern crate wayland_sys;
use wlroots::utils::wl_listener;
use wayland_sys::server;

// For graphical functions
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
struct State {
    /// The color on the screen.
    color: [f32; 3],
    dec: i32,
    /// How long since the last frame was rendered.
    last_frame: Instant
}

impl State {
    unsafe fn new() -> Self {
        State {
            color: [1.0, 0.0, 0.0],
            dec: 0,
            last_frame: Instant::now()
        }
    }
}

// This is so wl_container_of works.
// repr(C) means the struct packing in deterministic.

#[repr(C)]
struct OutputWrapper {
    state: *mut State,
    output: wl_listener
}

unsafe fn add_frame_callbacks() {
    // We store the wl_listener in a struct, and grab it again
    // using the magic of wl_container_of.
    let mut output_frame_listener =wl_listener::new(draw_frame);
    // Keep a raw pointer to make sure wl_container_of works.
    //let mut output_raw = Box::into_raw(output_frame_listener);
    let mut state = Box::new(State::new());
    let mut state_raw = Box::into_raw(state);
    let mut output_wrapper = Box::new(OutputWrapper {
        state: state_raw,
        output: output_frame_listener as _
    });
    let outputs = wlroots::output::OUTPUTS.try_lock()
        .expect("Output lock was already acquired");
    for output in &*outputs {
        wlroots::utils::wl_signal_add(&mut (*output.inner).events.frame,
                                      &mut output_wrapper.output as &mut _);
    }
    ::std::mem::forget(output_wrapper);
}

unsafe extern "C" fn draw_frame(listener: *mut server::wl_listener,
                                data: *mut c_void) {
    // SUPER unsafe
    let output_wrapper = wl_container_of!(listener,
                                          OutputWrapper,
                                          output) as *mut OutputWrapper;
    let state = &mut *(*output_wrapper).state;
    let now = Instant::now();
    let delta = now.duration_since(state.last_frame);
    let seconds_delta= delta.as_secs();
    let nano_delta = delta.subsec_nanos() as u64;
    let ms = (seconds_delta * 1000) + nano_delta / 1000000;
    let inc = (state.dec + 1) % 3;
    state.color[inc as usize] += ms as f32 / 2000.0;
    state.color[state.dec as usize] -= ms as f32 / 2000.0;

    if state.color[state.dec as usize] < 0.0 {
        state.color[inc as usize] = 1.0;
        state.color[state.dec as usize] = 0.0;
        state.dec = inc;
    }
    state.last_frame = now;
    gl::ClearColor(state.color[0], state.color[1], state.color[2], 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

}
