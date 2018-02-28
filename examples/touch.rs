#[macro_use]
extern crate wlroots;

use wlroots::{Compositor, CompositorBuilder, InputManagerHandler, Keyboard, KeyboardHandler,
              Output, OutputBuilder, OutputBuilderResult, OutputHandler, OutputLayout,
              OutputManagerHandler, PointerHandler, Texture, TextureFormat, Touch, TouchHandler};
use wlroots::key_events::KeyEvent;
use wlroots::touch_events::{DownEvent, MotionEvent, UpEvent};
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

const CAT_STRIDE: i32 = 128;
const CAT_WIDTH: i32 = 128;
const CAT_HEIGHT: i32 = 128;
const CAT_DATA: &'static [u8] = include_bytes!("cat.data");

#[derive(Debug, Clone)]
struct TouchPoint {
    touch_id: i32,
    x: f64,
    y: f64
}

struct State {
    cat_texture: Option<Texture>,
    touch_points: Vec<TouchPoint>
}

impl State {
    fn new() -> Self {
        State { cat_texture: None,
                touch_points: Vec::new() }
    }
}

compositor_data!(State);

struct TouchHandlerEx;

struct OutputManager;

struct ExOutput;

struct InputManager;

struct ExPointer;

struct ExKeyboardHandler;

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _compositor: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        Some(builder.build_best_mode(ExOutput))
    }
}

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self, compositor: &mut Compositor, _: &mut Keyboard, key_event: &mut KeyEvent) {
        for key in key_event.pressed_keys() {
            if key == KEY_Escape {
                compositor.terminate()
            }
        }
    }
}

impl PointerHandler for ExPointer {}

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut Output) {
        let renderer = compositor.renderer.as_mut().unwrap();
        let state: &mut State = (&mut compositor.data).downcast_mut().unwrap();
        // NOTE gl functions will probably always be unsafe.
        let (width, height) = output.effective_resolution();
        let transform_matrix = output.transform_matrix();
        let mut renderer = renderer.render(output);
        renderer.clear([0.25, 0.25, 0.25, 1.0]);
        let cat_texture = state.cat_texture.as_mut().unwrap();
        let (cat_width, cat_height) = cat_texture.size();
        for touch_point in &mut state.touch_points {
            let matrix =
                cat_texture.get_matrix(&transform_matrix,
                                       (touch_point.x * width as f64) as i32 - (cat_width / 2),
                                       (touch_point.y * height as f64) as i32 - (cat_height / 2));
            renderer.render_with_matrix(cat_texture, &matrix);
        }
    }
}

impl TouchHandler for TouchHandlerEx {
    fn on_down(&mut self, compositor: &mut Compositor, _touch: &mut Touch, event: &DownEvent) {
        let state: &mut State = compositor.into();
        let (width, height) = event.size();
        let (x, y) = event.location();
        let point = TouchPoint { touch_id: event.touch_id(),
                                 x: x / width,
                                 y: y / height };
        wlr_log!(L_ERROR, "New touch point at {:?}", point);
        state.touch_points.push(point)
    }

    fn on_up(&mut self, compositor: &mut Compositor, _touch: &mut Touch, event: &UpEvent) {
        let state: &mut State = compositor.into();
        wlr_log!(L_ERROR,
                 "Removing {:?} from {:#?}",
                 event.touch_id(),
                 state.touch_points);
        if let Some(index) = state.touch_points
                                  .iter()
                                  .position(|touch_point| touch_point.touch_id == event.touch_id())
        {
            state.touch_points.remove(index);
        }
    }

    fn on_motion(&mut self, compositor: &mut Compositor, _touch: &mut Touch, event: &MotionEvent) {
        let state: &mut State = compositor.into();
        let (width, height) = event.size();
        let (x, y) = event.location();
        wlr_log!(L_ERROR, "New location: {:?}", (x, y));
        for touch_point in &mut state.touch_points {
            if touch_point.touch_id == event.touch_id() {
                touch_point.x = x / width;
                touch_point.y = y / height;
            }
        }
    }
}

impl InputManagerHandler for InputManager {
    fn touch_added(&mut self, _: &mut Compositor, _: &mut Touch) -> Option<Box<TouchHandler>> {
        Some(Box::new(TouchHandlerEx))
    }
    fn keyboard_added(&mut self,
                      _: &mut Compositor,
                      _: &mut Keyboard)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

fn main() {
    init_logging(L_DEBUG, None);
    let mut layout = OutputLayout::new().expect("Could not construct an output layout");
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .build_auto(State::new(),
                                                             Some(Box::new(InputManager)),
                                                             Some(Box::new(OutputManager)),
                                                             None);
    {
        let gles2 = &mut compositor.renderer.as_mut().unwrap();
        let compositor_data: &mut State = (&mut compositor.data).downcast_mut().unwrap();
        compositor_data.cat_texture = gles2.create_texture().map(|mut cat_texture| {
            cat_texture.upload_pixels(TextureFormat::ABGR8888,
                                      CAT_STRIDE,
                                      CAT_WIDTH,
                                      CAT_HEIGHT,
                                      CAT_DATA);
            cat_texture
        });
    }
    compositor.run();
}
