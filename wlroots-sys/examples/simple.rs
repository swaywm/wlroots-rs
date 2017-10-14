extern crate wlroots_sys;
extern crate wayland_sys;
extern crate wayland_server;

use wlroots_sys::wlr_backend_destroy;

#[allow(warnings)]
mod shared;
use shared::*;

// For graphical functions
// TODO Move into real library
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// TODO Necessary? Probably
#[repr(C)]
struct SampleState {
    color: [f32; 3],
    dec: usize
}

unsafe extern "C" fn handle_output_frame(output: *mut output_state, ts: *mut timespec) {
    let output = &mut *output;
    let ts = &mut *ts;
    let state = &mut *output.compositor;
    let sample = &mut *(state.data as *mut SampleState);

    let ms = (ts.tv_sec - state.last_frame.tv_sec) * 1000 +
        (ts.tv_nsec - state.last_frame.tv_nsec) / 1000000;
    let inc = (sample.dec + 1) % 3;

    sample.color[inc] += ms as f32 / 2000.0;
    sample.color[sample.dec] -= ms as f32 / 2000.0;
    if sample.color[sample.dec] < 0.0 {
        sample.color[inc] = 1.0;
        sample.color[sample.dec] = 0.0;
        sample.dec = inc;
    }
    wlr_output_make_current(output.output);
    gl::ClearColor(sample.color[0], sample.color[1], sample.color[2], 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    wlr_output_swap_buffers(output.output);
}

fn main() {
    unsafe {
        let mut state = SampleState {
            color: [1.0, 0.0, 0.0],
            dec: 0
        };
        let mut compositor = compositor_state::default();
        compositor.data = &mut state as *mut _ as *mut _;
        compositor.output_frame_cb = Some(handle_output_frame);
        compositor_init(&mut compositor);
        if !wlr_backend_start(compositor.backend) {
            wlr_backend_destroy(compositor.backend as _);
            panic!("Failed to start backend");
        }
        wl_display_run(compositor.display);
        compositor_fini(&mut compositor);
    }
}
