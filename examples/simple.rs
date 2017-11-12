#[macro_use]
extern crate wlroots;

use std::time::Instant;

use wlroots::{Compositor, CompositorBuilder, InputManagerHandler, KeyEvent, KeyboardHandler,
              OutputBuilder, OutputBuilderResult, OutputHandler, OutputManagerHandler};
use wlroots::types::{KeyboardHandle, OutputHandle};
use wlroots::wlroots_sys::gl;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct Output {
    color: [f32; 3],
    dec: usize,
    last_frame: Instant
}

struct OutputManager;

struct InputManager;
struct ExKeyboardHandler;

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self,
              compositor: &mut Compositor,
              keyboard: &mut KeyboardHandle,
              key_event: &mut KeyEvent) {
        let keys = key_event.input_keys();

        wlr_log!(L_DEBUG,
                 "Got key event. Keys: {:?}. Modifiers: {}",
                 keys,
                 keyboard.get_modifiers());

        for key in keys {
            if key == KEY_Escape {
                compositor.terminate()
            }
        }
    }
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self,
                      _: &mut Compositor,
                      _: &mut KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        Some(builder.build_best_mode(Output {
                                         color: [0.0, 0.0, 0.0],
                                         dec: 0,
                                         last_frame: Instant::now()
                                     }))
    }
}

impl OutputHandler for Output {
    fn output_frame(&mut self, _: &mut Compositor, output: &mut OutputHandle) {
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
        output.make_current();
        unsafe {
            gl::ClearColor(self.color[0], self.color[1], self.color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        output.swap_buffers()
    }
}

fn main() {
    CompositorBuilder::new()
        .build_auto(Box::new(InputManager), Box::new(OutputManager))
        .run()
}
