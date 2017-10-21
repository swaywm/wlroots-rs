#[macro_use]
extern crate wlroots;

use std::env;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use wlroots::compositor::Compositor;
use wlroots::manager::{InputManagerHandler, OutputManagerHandler};
use wlroots::output::Output;
use wlroots::device::Device;
use wlroots::key_event::KeyEvent;
use wlroots::wlroots_sys::{gl, wlr_input_device, wlr_output, wlr_output_make_current,
                           wlr_output_swap_buffers, xkb_keysym_t};
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct OutputHandler {
    color: [f32; 3],
    dec: usize,
    last_frame: Instant
}

struct InputManager {
    //compositor: Rc<Compositor>,
    dev: Option<Device>
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self, dev: Device) {
        self.dev = Some(dev)
    }

    fn key(&mut self, key_event: KeyEvent) {
        let keys = key_event.get_input_keys(self.dev.clone().unwrap());
        for key in keys {
            if key == KEY_Escape {
                panic!()
                //compositor.terminate()
            }
        }
    }
}

impl OutputManagerHandler for OutputHandler {
    fn output_frame(&mut self, output: Output) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        let seconds_delta = delta.as_secs();
        let nano_delta = delta.subsec_nanos() as u64;
        let ms = (seconds_delta * 1000) + nano_delta / 1000000;
        let inc = (self.dec + 1) % 3;
        self.color[inc as usize] += ms as f32 / 2000.0;
        self.color[self.dec as usize] -= ms as f32 / 2000.0;

        if self.color[self.dec as usize] < 0.0 {
            self.color[inc as usize] = 1.0;
            self.color[self.dec as usize] = 0.0;
            self.dec = inc;
        }
        self.last_frame = now;
        // NOTE gl functions will probably always be unsafe.
        // NOTE wlr_output_* functions will be wrapped eventually.
        unsafe {
            wlr_output_make_current(output.to_ptr());
            gl::ClearColor(self.color[0], self.color[1], self.color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            wlr_output_swap_buffers(output.to_ptr());
        }
    }
}

fn main() {
    let dummy = 0;
    let input_manager = InputManager { dev: None };
    let output_manager = OutputHandler {
        color: [0.0, 0.0, 0.0],
        dec: 0,
        last_frame: Instant::now()
    };
    let mut compositor = Compositor::new(Box::new(input_manager), Box::new(output_manager));
    compositor.run();
}
