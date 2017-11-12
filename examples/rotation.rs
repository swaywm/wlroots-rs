#[macro_use]
extern crate wlroots;

use std::time::Instant;

use wlroots::{Compositor, CompositorBuilder, InputManagerHandler, KeyEvent, KeyboardHandler,
              OutputBuilder, OutputBuilderResult, OutputHandler, OutputManagerHandler};
use wlroots::render::Texture;
use wlroots::types::{KeyboardHandle, OutputHandle};
use wlroots::wlroots_sys::wl_output_transform;
use wlroots::xkbcommon::xkb::keysyms;

const CAT_STRIDE: i32 = 128;
const CAT_WIDTH: i32 = 128;
const CAT_HEIGHT: i32 = 128;
const CAT_DATA: &'static [u8] = include_bytes!("cat.data");

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

struct CompositorState {
    cat_texture: Option<Texture>,
    last_frame: Instant,
    x_offs: f32,
    y_offs: f32,
    x_vel: f32,
    y_vel: f32
}

compositor_data!(CompositorState);

impl CompositorState {
    fn new() -> Self {
        CompositorState {
            cat_texture: None,
            last_frame: Instant::now(),
            x_offs: 0.0,
            y_offs: 0.0,
            x_vel: 128.0,
            y_vel: 128.0
        }
    }
}

// TODO Basic rotation
// TODO Config reading
// TODO Arrow key velocity control

struct OutputManager;

struct Output;

struct InputManager;

struct KeyboardManager;

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let output = Output;
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
    fn output_frame(&mut self, compositor: &mut Compositor, output: &mut OutputHandle) {
        let (width, height) = output.effective_resolution();
        let renderer = compositor
            .gles2_renderer
            .as_mut()
            .expect("Compositor was not loaded with gles2 renderer");
        let compositor_data: &mut CompositorState = (&mut compositor.data)
            .downcast_mut()
            .unwrap();
        let now = Instant::now();
        let delta = now.duration_since(compositor_data.last_frame);
        let seconds_delta = delta.as_secs() as f32;
        let nano_delta = delta.subsec_nanos() as u64;
        let ms = (seconds_delta * 1000.0) + nano_delta as f32 / 1000000.0;
        let seconds = ms / 1000.0;
        // TODO the method probably takes a different type, because you nede to call
        // start first. Will look into it.
        renderer.render(output, |renderer, output| {
            let cat_texture = compositor_data.cat_texture.as_ref().unwrap();
            for y in StepRange(-128 + compositor_data.y_offs as i32, height, 128) {
                for x in StepRange(-128 + compositor_data.x_offs as i32, width, 128) {
                    let matrix = cat_texture.get_matrix(&output.transform_matrix(), x, y);
                    // wlr_log!(L_ERROR, "x: {}, y: {}", x, y);
                    renderer.render_with_matrix(&cat_texture, &matrix);
                }
            }
        });
        compositor_data.x_offs += compositor_data.x_vel * seconds;
        compositor_data.y_offs += compositor_data.y_vel * seconds;
        if compositor_data.x_offs > 128.0 {
            compositor_data.x_offs = 0.0
        }
        if compositor_data.y_offs > 128.0 {
            compositor_data.y_offs = 0.0
        }
        compositor_data.last_frame = now;
    }
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self,
                      _: &mut Compositor,
                      _: &mut KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(KeyboardManager))
    }
}

impl KeyboardHandler for KeyboardManager {
    fn on_key(&mut self,
              compositor: &mut Compositor,
              _: &mut KeyboardHandle,
              key_event: &mut KeyEvent) {
        let keys = key_event.input_keys();

        for key in keys {
            match key {
                keysyms::KEY_Escape => compositor.terminate(),
                keysyms::KEY_Left => update_velocities(compositor.into(), -16.0, 0.0),
                keysyms::KEY_Right => update_velocities(compositor.into(), 16.0, 0.0),
                keysyms::KEY_Up => update_velocities(compositor.into(), 0.0, -16.0),
                keysyms::KEY_Down => update_velocities(compositor.into(), 0.0, 16.0),
                _ => {}
            }
        }
    }
}

fn update_velocities(compositor: &mut CompositorState, x_diff: f32, y_diff: f32) {
    compositor.x_vel += x_diff;
    compositor.y_vel += y_diff;
}

fn main() {
    let compositor_state = CompositorState::new();
    let input_manager = Box::new(InputManager);
    let output_manager = Box::new(OutputManager);
    let mut compositor = CompositorBuilder::new()
        .gles2_renderer(true)
        .build_auto(compositor_state, input_manager, output_manager);
    {
        let gles2_renderer = &mut compositor.gles2_renderer.as_mut().unwrap();
        let compositor_data: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
        compositor_data.cat_texture = gles2_renderer
            .create_texture()
            .map(|mut cat_texture| {
                     cat_texture.upload_pixels(CAT_STRIDE, CAT_WIDTH, CAT_HEIGHT, CAT_DATA);
                     cat_texture
                 })
    };
    compositor.run();
}
