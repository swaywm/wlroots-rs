#[macro_use]
extern crate wlroots;

use std::env;
use std::time::Instant;

use wlroots::{Compositor, CompositorBuilder, InputManagerHandler, Keyboard, KeyboardHandler,
              Output, OutputBuilder, OutputBuilderResult, OutputHandler, OutputManagerHandler};
use wlroots::key_events::KeyEvent;
use wlroots::render::{Texture, TextureFormat};
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::wlroots_sys::wl_output_transform;
use wlroots::xkbcommon::xkb::keysyms;

const CAT_WIDTH: u32 = 128;
const CAT_HEIGHT: u32 = 128;
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
    rotation: wl_output_transform,
    last_frame: Instant,
    x_offs: f32,
    y_offs: f32,
    x_vel: f32,
    y_vel: f32
}

compositor_data!(CompositorState);

impl CompositorState {
    fn new(rotation: wl_output_transform) -> Self {
        CompositorState { cat_texture: None,
                          rotation,
                          last_frame: Instant::now(),
                          x_offs: 0.0,
                          y_offs: 0.0,
                          x_vel: 128.0,
                          y_vel: 128.0 }
    }
}

struct OutputManager;

struct ExOutput;

struct InputManager;

struct KeyboardManager;

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             compositor: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let compositor_data: &mut CompositorState = compositor.into();
        let output = ExOutput;
        let res = builder.build_best_mode(output);
        res.output.transform(compositor_data.rotation);
        Some(res)
    }
}

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut Output) {
        let (width, height) = output.effective_resolution();
        let renderer = compositor.renderer
                                 .as_mut()
                                 .expect("Compositor was not loaded with gles2 renderer");
        let compositor_data: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
        let now = Instant::now();
        let delta = now.duration_since(compositor_data.last_frame);
        let seconds_delta = delta.as_secs() as f32;
        let nano_delta = delta.subsec_nanos() as u64;
        let ms = (seconds_delta * 1000.0) + nano_delta as f32 / 1000000.0;
        let seconds = ms / 1000.0;
        let transform_matrix = output.transform_matrix();
        let mut renderer = renderer.render(output, None);
        let cat_texture = compositor_data.cat_texture.as_ref().unwrap();
        for y in StepRange(-128 + compositor_data.y_offs as i32, height, 128) {
            for x in StepRange(-128 + compositor_data.x_offs as i32, width, 128) {
                renderer.render_texture(&cat_texture, transform_matrix, x, y, 1.0);
            }
        }
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
                      _: &mut Keyboard)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(KeyboardManager))
    }
}

impl KeyboardHandler for KeyboardManager {
    fn on_key(&mut self, compositor: &mut Compositor, _: &mut Keyboard, key_event: &mut KeyEvent) {
        let keys = key_event.pressed_keys();

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
    init_logging(L_DEBUG, None);
    use wl_output_transform::*;
    let mut args = env::args();
    args.next();
    let rotation = if let Some(arg) = args.next() {
        match arg.as_str() {
            "90" => WL_OUTPUT_TRANSFORM_90,
            "180" => WL_OUTPUT_TRANSFORM_180,
            "270" => WL_OUTPUT_TRANSFORM_270,
            "flipped" => WL_OUTPUT_TRANSFORM_FLIPPED,
            "flipped_90" => WL_OUTPUT_TRANSFORM_FLIPPED_90,
            "flipped_180" => WL_OUTPUT_TRANSFORM_FLIPPED_180,
            "flipped_270" => WL_OUTPUT_TRANSFORM_FLIPPED_270,
            _ => WL_OUTPUT_TRANSFORM_NORMAL
        }
    } else {
        WL_OUTPUT_TRANSFORM_NORMAL
    };
    let compositor_state = CompositorState::new(rotation);
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .input_manager(Box::new(InputManager))
                                                 .output_manager(Box::new(OutputManager))
                                                 .build_auto(compositor_state);
    {
        let gles2 = &mut compositor.renderer.as_mut().unwrap();
        let compositor_data: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
        compositor_data.cat_texture =
            gles2.create_texture_from_pixels(TextureFormat::ABGR8888.into(),
                                             CAT_WIDTH * 4,
                                             CAT_WIDTH,
                                             CAT_HEIGHT,
                                             CAT_DATA);
    }
    compositor.run();
}
