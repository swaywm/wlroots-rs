#[macro_use]
extern crate wlroots;

use std::{env, time::Instant};

use wlroots::{compositor,
              input::{self, keyboard},
              output,
              render::{Texture, TextureFormat},
              utils::log::{init_logging, WLR_DEBUG}};
use wlroots::wlroots_sys::wl_output_transform;
use wlroots::xkbcommon::xkb::keysyms;

const CAT_TEXTURE_WIDTH: u32 = 128;
const CAT_TEXTURE_HEIGHT: u32 = 128;
const CAT_TEXTURE_DATA: &'static [u8] = include_bytes!("cat.data");
const VELOCITY_STEP_DIFF: f32 = 16.0;

struct Vector2 {
    x: f32,
    y: f32,
}
impl Vector2 {
    pub fn increment(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
}

struct CompositorState {
    cat_texture: Option<Texture<'static>>,
    rotation_transform: wl_output_transform,
    last_frame: Instant,
    offset: Vector2,
    velocity: Vector2,
}
impl CompositorState {
    fn new(rotation_transform: wl_output_transform) -> Self {
        CompositorState { cat_texture: None,
                          rotation_transform,
                          last_frame: Instant::now(),
                          offset: Vector2 {
                              x: 0.0,
                              y: 0.0
                          },
                          velocity: Vector2 {
                              x: 128.0,
                              y: 128.0
                          }}
    }

    /// Registers `now` as the last frame and returns the calculated delta time since the previous last frame in seconds.
    pub fn register_frame(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        let seconds_delta = delta.as_secs() as f32;
        let nano_delta = delta.subsec_nanos() as u64;
        let ms = (seconds_delta * 1000.0) + nano_delta as f32 / 1000000.0;
        ms / 1000.0
    }
}

compositor_data!(CompositorState);

fn output_added<'output>(compositor_handle: compositor::Handle,
                         output_builder: output::Builder<'output>)
                         -> Option<output::BuilderResult<'output>> {
    let ex_output = ExOutput;
    let mut result = output_builder.build_best_mode(ex_output);
    with_handles!([(compositor: {compositor_handle}), (output: {&mut result.output})] => {
        let compositor_state: &mut CompositorState = compositor.into();
        output.transform(compositor_state.rotation_transform);
    }).unwrap();
    Some(result)
}

struct ExOutput;
impl output::Handler for ExOutput {
    fn on_frame(&mut self,
                mut compositor_handle: compositor::Handle,
                mut output_handle: output::Handle) {
        with_handles!([(compositor: {&mut compositor_handle}), (output: {&mut output_handle})] => {
            let (output_width, output_height) = output.effective_resolution();
            let renderer = compositor.renderer
                                    .as_mut()
                                    .expect("Compositor was not loaded with gles2 renderer");
            let compositor_state: &mut CompositorState = (&mut compositor.data).downcast_mut()
                .unwrap();
            let delta_time_in_seconds = compositor_state.register_frame();
            let transform_matrix = output.transform_matrix();
            let mut renderer = renderer.render(output, None);
            let cat_texture = compositor_state.cat_texture.as_ref().unwrap();
            let (max_width, max_height) = (CAT_TEXTURE_WIDTH as i32, CAT_TEXTURE_HEIGHT as i32);
            for y in (-max_height + compositor_state.offset.y as i32..output_height).step_by(max_height as usize) {
                for x in (-max_width + compositor_state.offset.x as i32..output_width).step_by(max_width as usize) {
                    renderer.render_texture(&cat_texture, transform_matrix, x, y, 1.0);
                }
            }
            compositor_state.offset.increment(
                compositor_state.velocity.x * delta_time_in_seconds,
                compositor_state.velocity.y * delta_time_in_seconds
            );
            if compositor_state.offset.x > max_width as f32 {
                compositor_state.offset.x = 0.0
            }
            if compositor_state.offset.y > max_height as f32 {
                compositor_state.offset.y = 0.0
            }
        }).unwrap();
    }
}

struct InputManager;
impl input::ManagerHandler for InputManager {
    fn keyboard_added(&mut self,
                      _compositor_handle: compositor::Handle,
                      _keyboard_handle: keyboard::Handle)
                      -> Option<Box<keyboard::Handler>> {
        Some(Box::new(KeyboardManager))
    }
}

struct KeyboardManager;
impl keyboard::Handler for KeyboardManager {
    fn on_key(&mut self,
              compositor_handle: compositor::Handle,
              _keyboard_handle: keyboard::Handle,
              key_event: &keyboard::event::Key) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
            for key in key_event.pressed_keys() {
                match key {
                    keysyms::KEY_Escape => compositor::terminate(),
                    keysyms::KEY_Left => compositor_state.velocity.increment(-VELOCITY_STEP_DIFF, 0.0),
                    keysyms::KEY_Right => compositor_state.velocity.increment(VELOCITY_STEP_DIFF, 0.0),
                    keysyms::KEY_Up => compositor_state.velocity.increment(0.0, -VELOCITY_STEP_DIFF),
                    keysyms::KEY_Down => compositor_state.velocity.increment(0.0, VELOCITY_STEP_DIFF),
                    _ => {}
                }
            }
        }).unwrap();
    }
}

fn rotation_transform_from_str(rotation_str: &str) -> wl_output_transform {
    use wl_output_transform::*;
    match rotation_str {
        "90" => WL_OUTPUT_TRANSFORM_90,
        "180" => WL_OUTPUT_TRANSFORM_180,
        "270" => WL_OUTPUT_TRANSFORM_270,
        "flipped" => WL_OUTPUT_TRANSFORM_FLIPPED,
        "flipped_90" => WL_OUTPUT_TRANSFORM_FLIPPED_90,
        "flipped_180" => WL_OUTPUT_TRANSFORM_FLIPPED_180,
        "flipped_270" => WL_OUTPUT_TRANSFORM_FLIPPED_270,
        _ => WL_OUTPUT_TRANSFORM_NORMAL
    }
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let mut args = env::args();
    let rotation_argument_string = args.nth(1).unwrap_or_else(|| "".to_string());
    let rotation_transform = rotation_transform_from_str(&rotation_argument_string);
    let compositor_state = CompositorState::new(rotation_transform);
    let output_builder = output::ManagerBuilder::default().output_added(output_added);
    let mut compositor = compositor::Builder::new().gles2(true)
                                                   .input_manager(Box::new(InputManager))
                                                   .output_manager(output_builder)
                                                   .build_auto(compositor_state);
    {
        let gles2 = &mut compositor.renderer.as_mut().unwrap();
        let compositor_state: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
        compositor_state.cat_texture =
            gles2.create_texture_from_pixels(TextureFormat::ABGR8888.into(),
                                             CAT_TEXTURE_WIDTH * 4,
                                             CAT_TEXTURE_WIDTH,
                                             CAT_TEXTURE_HEIGHT,
                                             CAT_TEXTURE_DATA);
    }
    compositor.run();
}
