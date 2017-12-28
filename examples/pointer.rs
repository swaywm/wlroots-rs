#[macro_use]
extern crate wlroots;

use std::cell::RefCell;
use std::rc::Rc;
use wlroots::{AxisEvent, ButtonEvent, Compositor, CompositorBuilder, Cursor, InputManagerHandler,
              KeyEvent, KeyboardHandler, MotionEvent, OutputBuilder, OutputBuilderResult,
              OutputHandler, OutputLayout, OutputManagerHandler, PointerHandler, XCursorTheme, XCursor};
use wlroots::types::{KeyboardHandle, OutputHandle, PointerHandle};
use wlroots::wlroots_sys::gl;
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct State {
    color: [f32; 4],
    default_color: [f32; 4],
    cursor: Cursor,
    xcursor: XCursor
}

impl State {
    fn new(cursor: Cursor, xcursor: XCursor) -> Self {
        State {
            color: [0.25, 0.25, 0.25, 1.0],
            default_color: [0.25, 0.25, 0.25, 1.0],
            cursor,
            xcursor
        }
    }
}

compositor_data!(State);

struct OutputManager;

struct Output;

struct InputManager;

struct Pointer;

struct ExKeyboardHandler;

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             compositor: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let result = builder.build_best_mode(Output);
        let state: &mut State = compositor.into();
        let cursor = &mut state.cursor;
        // TODO use output config if present instead of auto
        {
            let layout = cursor
                .output_layout()
                .as_ref()
                .expect("Could not get output layout");
            result.output.add_layout_auto(layout.clone());
        }
        let image = &state.xcursor.images()[0];
        cursor.set_cursor_image(image);
        let (x, y) = cursor.coords();
        // https://en.wikipedia.org/wiki/Mouse_warping
        cursor.warp(None, x, y);
        Some(result)
    }
}

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self,
              compositor: &mut Compositor,
              _: &mut KeyboardHandle,
              key_event: &mut KeyEvent) {
        for key in key_event.input_keys() {
            if key == KEY_Escape {
                compositor.terminate()
            }
        }
    }
}

impl PointerHandler for Pointer {
    fn on_motion(&mut self,
                 compositor: &mut Compositor,
                 _: &mut PointerHandle,
                 event: &MotionEvent) {
        let state: &mut State = compositor.into();
        let (delta_x, delta_y) = event.delta();
        state.cursor.move_to(&event.device(), delta_x, delta_y);
    }

    fn on_button(&mut self,
                 compositor: &mut Compositor,
                 _: &mut PointerHandle,
                 event: &ButtonEvent) {
        let state: &mut State = compositor.into();
        if event.state() == WLR_BUTTON_RELEASED {
            state.color = state.default_color;
        } else {
            state.color = [0.25, 0.25, 0.25, 1.0];
            state.color[event.button() as usize % 3] = 1.0;
        }
    }

    fn on_axis(&mut self, compositor: &mut Compositor, _: &mut PointerHandle, event: &AxisEvent) {
        let state: &mut State = compositor.into();
        for color_byte in &mut state.default_color[..3] {
            *color_byte += if event.delta() > 0.0 { -0.05 } else { 0.05 };
            if *color_byte > 1.0 {
                *color_byte = 1.0
            }
            if *color_byte < 0.0 {
                *color_byte = 0.0
            }
        }
        state.color = state.default_color.clone()
    }
}

impl OutputHandler for Output {
    fn output_frame(&mut self, compositor: &mut Compositor, output: &mut OutputHandle) {
        let state: &mut State = compositor.into();
        output.make_current();
        unsafe {
            gl::ClearColor(state.color[0], state.color[1], state.color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        output.swap_buffers();
    }
}

impl InputManagerHandler for InputManager {
    fn pointer_added(&mut self,
                     _: &mut Compositor,
                     _: &mut PointerHandle)
                     -> Option<Box<PointerHandler>> {
        Some(Box::new(Pointer))
    }

    fn keyboard_added(&mut self,
                      _: &mut Compositor,
                      _: &mut KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

fn main() {
    let mut cursor = Cursor::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let xcursor = xcursor_theme
        .get_cursor("left_ptr".into())
        .expect("Could not load cursor from theme");
    let layout = Rc::new(RefCell::new(OutputLayout::new()));

    cursor.attach_output_layout(layout);
    let compositor = CompositorBuilder::new().build_auto(State::new(cursor, xcursor),
                                                         Box::new(InputManager),
                                                         Box::new(OutputManager));
    compositor.run();
}
