#[macro_use]
extern crate wlroots;

use std::time::Instant;

use wlroots::{CompositorBuilder, CompositorHandle, InputManagerHandler, KeyboardHandle,
              KeyboardHandler, OutputBuilder, OutputBuilderResult, OutputHandle, OutputHandler,
              OutputManagerHandler};
use wlroots::key_events::KeyEvent;
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::wlroots_sys::gl;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct ExOutput {
    color: [f32; 3],
    dec: usize,
    last_frame: Instant
}

struct OutputManager;

struct InputManager;
struct ExKeyboardHandler;

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self,
              compositor: CompositorHandle,
              keyboard: KeyboardHandle,
              key_event: &KeyEvent) {
        let keys = key_event.pressed_keys();
        with_handles!([(keyboard: {keyboard}),
                      (compositor: {compositor})] => {
            wlr_log!(L_DEBUG,
                     "Got key event. Keys: {:?}. Modifiers: {}",
                     keys,
                     keyboard.get_modifiers());
            for key in keys {
                if key == KEY_Escape {
                    compositor.terminate()
                }
            }
        }).unwrap();
    }
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self,
                      _: CompositorHandle,
                      _: KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _: CompositorHandle,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        Some(builder.build_best_mode(ExOutput { color: [0.0, 0.0, 0.0],
                                                dec: 0,
                                                last_frame: Instant::now() }))
    }
}

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, _: CompositorHandle, output: OutputHandle) {
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
        with_handles!([(output: {output})] => {
            unsafe {
                output.make_current();
                gl::ClearColor(self.color[0], self.color[1], self.color[2], 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                output.swap_buffers(None, None);
            }
        }).unwrap();
    }
}

fn main() {
    init_logging(L_DEBUG, None);
    CompositorBuilder::new().input_manager(Box::new(InputManager))
                            .output_manager(Box::new(OutputManager))
                            .build_auto(())
                            .run()
}
