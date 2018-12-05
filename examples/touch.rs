#[macro_use]
extern crate wlroots;
extern crate libc;

use wlroots::{CompositorBuilder, CompositorHandle, InputManagerHandler, KeyboardHandle,
              KeyboardHandler, OutputBuilder, OutputBuilderResult, OutputHandle, OutputHandler,
              OutputManagerHandler, Texture, TextureFormat, TouchHandle, TouchHandler};
use wlroots::key_events::KeyEvent;
use wlroots::touch_events::{DownEvent, MotionEvent, UpEvent};
use wlroots::utils::log::{init_logging, WLR_DEBUG};
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

const CAT_WIDTH: u32 = 128;
const CAT_HEIGHT: u32 = 128;
const CAT_DATA: &'static [u8] = include_bytes!("cat.data");

#[derive(Debug, Clone)]
struct TouchPoint {
    touch_id: i32,
    x: f64,
    y: f64
}

struct State {
    cat_texture: Option<Texture<'static>>,
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

struct ExKeyboardHandler;

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _: CompositorHandle,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        Some(builder.build_best_mode(ExOutput))
    }
}

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self, _: CompositorHandle, _: KeyboardHandle, key_event: &KeyEvent) {
        for key in key_event.pressed_keys() {
            if key == KEY_Escape {
                wlroots::terminate()
            }
        }
    }
}

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor: CompositorHandle, output: OutputHandle) {
        with_handles!([(compositor: {compositor}), (output: {output})] => {
            let renderer = compositor.renderer.as_mut().unwrap();
            let state: &mut State = (&mut compositor.data).downcast_mut().unwrap();
            // NOTE gl functions will probably always be unsafe.
            let (width, height) = output.effective_resolution();
            let transform_matrix = output.transform_matrix();
            let mut renderer = renderer.render(output, None);
            renderer.clear([0.25, 0.25, 0.25, 1.0]);
            let cat_texture = state.cat_texture.as_mut().unwrap();
            let (cat_width, cat_height) = cat_texture.size();
            for touch_point in &mut state.touch_points {
                let x = (touch_point.x * width as f64) as i32 - (cat_width / 2);
                let y = (touch_point.y * height as f64) as i32 - (cat_height / 2);
                renderer.render_texture(cat_texture, transform_matrix, x, y, 1.0);
            }
        }).unwrap();
    }
}

impl TouchHandler for TouchHandlerEx {
    fn on_down(&mut self, compositor: CompositorHandle, _: TouchHandle, event: &DownEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let (x, y) = event.location();
            let point = TouchPoint { touch_id: event.touch_id(),
                                    x: x,
                                    y: y };
            wlr_log!(WLR_ERROR, "New touch point at {:?}", point);
            state.touch_points.push(point)
        }).unwrap();
    }

    fn on_up(&mut self, compositor: CompositorHandle, _: TouchHandle, event: &UpEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            wlr_log!(WLR_ERROR,
                    "Removing {:?} from {:#?}",
                    event.touch_id(),
                    state.touch_points);
            let touch_id = event.touch_id();
            if let Some(index) = state.touch_points
                                    .iter()
                                    .position(|touch_point| touch_point.touch_id == touch_id)
            {
                state.touch_points.remove(index);
            }
        }).unwrap();
    }

    fn on_motion(&mut self, compositor: CompositorHandle, _: TouchHandle, event: &MotionEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let (x, y) = event.location();
            wlr_log!(WLR_ERROR, "New location: {:?}", (x, y));
            for touch_point in &mut state.touch_points {
                if touch_point.touch_id == event.touch_id() {
                    touch_point.x = x;
                    touch_point.y = y;
                }
            }
        }).unwrap();
    }
}

impl InputManagerHandler for InputManager {
    fn touch_added(&mut self, _: CompositorHandle, _: TouchHandle) -> Option<Box<TouchHandler>> {
        Some(Box::new(TouchHandlerEx))
    }

    fn keyboard_added(&mut self,
                      _: CompositorHandle,
                      _: KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .input_manager(Box::new(InputManager))
                                                 .output_manager(Box::new(OutputManager))
                                                 .build_auto(State::new());
    {
        let gles2 = &mut compositor.renderer.as_mut().unwrap();
        let compositor_data: &mut State = (&mut compositor.data).downcast_mut().unwrap();
        compositor_data.cat_texture =
            gles2.create_texture_from_pixels(TextureFormat::ABGR8888.into(),
                                             CAT_WIDTH * 4,
                                             CAT_WIDTH,
                                             CAT_HEIGHT,
                                             CAT_DATA);
    }
    compositor.run();
}
