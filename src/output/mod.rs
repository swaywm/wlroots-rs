//! Methods and structs that control the physical outputs and what they display.


mod output;

use std::sync::Mutex;

pub use self::output::Output;
use ::Session;
use utils::{wl_listener, wl_signal_add};

use wayland_sys::server;
use wlroots_sys::{wl_list, wlr_output, wlr_output_set_mode};
use std::os::raw::c_void;

lazy_static! {
    pub static ref OUTPUTS: Mutex<Vec<Output>> =
        Mutex::new(Vec::with_capacity(128));
}

/// Sets up the session so that it listens for and automatically manages adding
/// and removing outputs.
pub unsafe fn init(session: &mut Session) {
    let backend = &mut (*session.backend.0);

    // Set up output_add
    let mut output_add_listener = Box::new(wl_listener::new(output_add));
    wl_signal_add(&mut backend.events.output_add,
                  &mut *output_add_listener);

    // Set up output_remove
    let mut output_remove_listener = Box::new(wl_listener::new(output_remove));
    wl_signal_add(&mut backend.events.output_remove,
                  &mut *output_remove_listener);

    // Leak the link in the list that points to the static function.
    ::std::mem::forget(output_add_listener);
    ::std::mem::forget(output_remove_listener);
}

unsafe extern "C" fn output_add(listener: *mut server::wl_listener,
                                data: *mut c_void) {
    let mut outputs = OUTPUTS.lock().expect("OUTPUTS mutex has been poisoned");
    let output: &mut wlr_output = &mut *(data as *mut _);
    let cur_mode = (*(*output.modes).items) as *mut _;
    wlr_output_set_mode(output, cur_mode);
    outputs.push(Output::new(output as *mut _));
    println!("Added outputs: {:?}", outputs);
}

unsafe extern "C" fn output_remove(listener: *mut server::wl_listener,
                                   data: *mut c_void) {
    let mut outputs = OUTPUTS.lock().expect("OUTPUTS mutex has been poisoned");
    let output_ptr = data as *const wlr_output;
    outputs.retain(|output| output.inner as *const _ != output_ptr);
    println!("Removed outputs: {:?}", outputs);
}
