#[macro_use]
extern crate wlroots;

use wlroots::{Compositor, CompositorBuilder, Cursor, CursorBuilder, CursorHandler, CursorId,
              InputManagerHandler, Keyboard, KeyboardHandler, Output, OutputBuilder,
              OutputBuilderResult, OutputHandler, OutputLayout, OutputManagerHandler, Pointer,
              PointerHandler, XCursorTheme};
use wlroots::key_events::KeyEvent;
use wlroots::pointer_events::{AxisEvent, ButtonEvent, MotionEvent};
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::wlroots_sys::gl;
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct State {
    color: [f32; 4],
    default_color: [f32; 4],
    xcursor_theme: XCursorTheme,
    cursor_id: CursorId,
    layout: OutputLayout
}

impl State {
    fn new(xcursor_theme: XCursorTheme, layout: OutputLayout, cursor_id: CursorId) -> Self {
        State { color: [0.25, 0.25, 0.25, 1.0],
                default_color: [0.25, 0.25, 0.25, 1.0],
                xcursor_theme,
                cursor_id,
                layout }
    }
}

compositor_data!(State);

struct ExCursor;

struct OutputManager;

struct ExOutput;

struct InputManager;

struct ExPointer;

struct ExKeyboardHandler;

impl CursorHandler for ExCursor {
    fn on_pointer_motion(&mut self, _: &mut Compositor, _: &mut Cursor, event: &mut MotionEvent) {
        wlr_log!(L_DEBUG, "cursor delta: {:?}", event.delta())
    }
}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             compositor: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let result = builder.build_best_mode(ExOutput);
        let state: &mut State = compositor.into();
        let xcursor = state.xcursor_theme
                           .get_cursor("left_ptr".into())
                           .expect("Could not load left_ptr cursor");
        let image = &xcursor.images()[0];
        // TODO use output config if present instead of auto
        state.layout.add_auto(result.output);
        let mut cursor = state.layout.cursor(state.cursor_id).unwrap();
        cursor.set_cursor_image(image);
        let (x, y) = cursor.coords();
        // https://en.wikipedia.org/wiki/Mouse_warping
        cursor.warp(None, x, y);
        Some(result)
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

impl PointerHandler for ExPointer {
    fn on_motion(&mut self, compositor: &mut Compositor, _: &mut Pointer, event: &MotionEvent) {
        let state: &mut State = compositor.into();
        let (delta_x, delta_y) = event.delta();
        let mut cursor = state.layout.cursor(state.cursor_id).unwrap();
        cursor.move_to(None, delta_x, delta_y);
    }

    fn on_button(&mut self, compositor: &mut Compositor, _: &mut Pointer, event: &ButtonEvent) {
        let state: &mut State = compositor.into();
        if event.state() == WLR_BUTTON_RELEASED {
            state.color = state.default_color;
        } else {
            state.color = [0.25, 0.25, 0.25, 1.0];
            state.color[event.button() as usize % 3] = 1.0;
        }
    }

    fn on_axis(&mut self, compositor: &mut Compositor, _: &mut Pointer, event: &AxisEvent) {
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

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut Output) {
        let state: &mut State = compositor.into();
        // NOTE gl functions will probably always be unsafe.
        unsafe {
            output.make_current();
            gl::ClearColor(state.color[0], state.color[1], state.color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            output.swap_buffers(None, None);
        }
    }
}

impl InputManagerHandler for InputManager {
    fn pointer_added(&mut self,
                     compositor: &mut Compositor,
                     pointer: &mut Pointer)
                     -> Option<Box<PointerHandler>> {
        let state: &mut State = compositor.into();
        let cursor = &mut state.layout.cursor(state.cursor_id).unwrap();
        cursor.attach_input_device(pointer.input_device());
        Some(Box::new(ExPointer))
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
    let cursor = CursorBuilder::new(Box::new(ExCursor)).expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let mut layout = OutputLayout::new().expect("Could not construct an output layout");

    let cursor_id = layout.attach_cursor(cursor);
    let compositor =
        CompositorBuilder::new().input_manager(Box::new(InputManager))
                                .output_manager(Box::new(OutputManager))
                                .build_auto(State::new(xcursor_theme, layout, cursor_id));
    compositor.run();
}
