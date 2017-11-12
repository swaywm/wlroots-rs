#[macro_use]
extern crate wlroots;

use std::cell::RefCell;
use std::rc::Rc;

use wlroots::{CompositorBuilder, InputDevice, InputManagerHandler, KeyEvent, KeyboardHandler,
              OutputBuilder, OutputBuilderResult, OutputHandler, OutputManagerHandler};
use wlroots::render::{GLES2Renderer, Texture};
use wlroots::types::{KeyboardHandle, OutputHandle};
use wlroots::wlroots_sys::wl_output_transform;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

const CAT_STRIDE: i32 = 128;
const CAT_WIDTH: i32 = 128;
const CAT_HEIGHT: i32 = 128;
const CAT_BYTES_PER_PXEL: i32 = 4;
const CAT_DATA: &'static [u8] = include_bytes!("cat.data");

static mut RENDERER: *mut GLES2Renderer = 0 as *mut _;

/// Helper step by iterator, because `step_by` on `Range` is unstable.
struct StepRange(i32, i32, i32);
impl Iterator for StepRange {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.0 < self.1 {
            let v = self.0;
            self.0 = v + self.2;
            Some(v)
        } else {
            None
        }
    }
}

// TODO Basic rotation
// TODO Config reading
// TODO Arrow key velocity control

struct OutputManager {
    cat_texture: Rc<RefCell<Option<Texture>>>
}

struct Output {
    cat_texture: Rc<RefCell<Option<Texture>>>,
    x_offs: f64,
    y_offs: f64,
    x_vel: f64,
    y_vel: f64
}

struct InputManager;

struct KeyboardManager;

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let output = Output {
            cat_texture: self.cat_texture.clone(),
            x_offs: 0.0,
            x_vel: 0.0,
            y_offs: 0.0,
            y_vel: 128.0
        };
        let res = builder.build_best_mode(output);
        // TODO
        // for output in self.outputs() {
        //    if output.name() == "pre-configured-name-from-config" {
        // TODO Don't hard code, alias name, read from file
        res.output
            .transform(wl_output_transform::WL_OUTPUT_TRANSFORM_NORMAL);
        //    }
        // }
        Some(res)
    }
}

impl OutputHandler for Output {
    fn output_frame(&mut self, output: &mut OutputHandle) {
        let (width, height) = output.effective_resolution();
        output.make_current();
        // TODO Make this safe
        // let renderer = self.gles2_renderer().expect("Renderer was not loaded");
        let renderer: &mut GLES2Renderer = unsafe { &mut *RENDERER };
        // TODO the method probably takes a different type, because you nede to call
        // start
        // first. Will look into it.
        renderer.render(output, |renderer, output| {
            let cat_texture = self.cat_texture.borrow_mut().take().unwrap();
            for y in StepRange(-128 + self.y_offs as i32, height, 128) {
                for x in StepRange(-128 + self.x_offs as i32, width, 128) {
                    let matrix = cat_texture.get_matrix(&output.transform_matrix(), x, y);
                    renderer.render_with_matrix(&cat_texture, &matrix);
                }
            }
            *self.cat_texture.borrow_mut() = Some(cat_texture)
        });
        // TODO Render stuff
        output.swap_buffers();
        // TODO time stuff
    }
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self, _: &mut InputDevice) -> Option<Box<KeyboardHandler>> {
        Some(Box::new(KeyboardManager))
    }
}

impl KeyboardHandler for KeyboardManager {
    fn on_key(&mut self, keyboard: &mut KeyboardHandle, key_event: &mut KeyEvent) {
        let keys = key_event.input_keys();

        wlr_log!(L_DEBUG,
                 "Got key event. Keys: {:?}. Modifiers: {}",
                 keys,
                 keyboard.get_modifiers());

        for key in keys {
            if key == KEY_Escape {
                wlroots::terminate()
            }
        }
    }
}

fn main() {
    let input_manager = Box::new(InputManager);
    let cat_texture = Rc::new(RefCell::new(None));
    let output_manager = Box::new(OutputManager { cat_texture: cat_texture.clone() });
    let mut compositor = CompositorBuilder::new()
        .gles2_renderer(true)
        .build_auto(input_manager, output_manager);
    {
        let gles2_renderer = &mut compositor.gles2_renderer.as_mut().unwrap();
        *cat_texture.borrow_mut() = gles2_renderer
            .create_texture()
            .map(|mut cat_texture| {
                     cat_texture.upload_pixels(CAT_STRIDE, CAT_WIDTH, CAT_HEIGHT, CAT_DATA);
                     cat_texture
                 });
        unsafe {
            RENDERER = *gles2_renderer;
        }
    }
    compositor.run();
}
