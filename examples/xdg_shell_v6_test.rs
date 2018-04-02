#[macro_use]
extern crate wlroots;

use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use wlroots::{project_box, Area, Capability, Compositor, CompositorBuilder, CursorBuilder,
              CursorHandler, CursorId, InputManagerHandler, Keyboard, KeyboardGrab,
              KeyboardHandle, KeyboardHandler, Origin, Output, OutputBuilder, OutputBuilderResult,
              OutputHandler, OutputLayout, OutputManagerHandler, Pointer, PointerHandler,
              Renderer, Seat, SeatHandler, SeatId, Size, Surface, XCursorTheme, XdgV6ShellHandler,
              XdgV6ShellManagerHandler, XdgV6ShellState, XdgV6ShellSurface,
              XdgV6ShellSurfaceHandle};
use wlroots::key_events::KeyEvent;
use wlroots::pointer_events::{ButtonEvent, MotionEvent};
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_PRESSED;
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::keysyms::{KEY_Escape, KEY_F1};

struct State {
    xcursor_theme: XCursorTheme,
    keyboard: Option<KeyboardHandle>,
    layout: OutputLayout,
    cursor_id: CursorId,
    shells: Vec<XdgV6ShellSurfaceHandle>,
    seat_id: Option<SeatId>
}

impl State {
    fn new(xcursor_theme: XCursorTheme, layout: OutputLayout, cursor_id: CursorId) -> Self {
        State { xcursor_theme,
                layout,
                cursor_id,
                keyboard: None,
                seat_id: None,
                shells: vec![] }
    }
}

compositor_data!(State);

struct SeatHandlerEx;

struct CursorEx;

impl CursorHandler for CursorEx {}

impl SeatHandler for SeatHandlerEx {}

struct XdgV6ShellHandlerEx;
struct XdgV6ShellManager;

impl XdgV6ShellHandler for XdgV6ShellHandlerEx {
    fn on_commit(&mut self,
                 compositor: &mut Compositor,
                 surface: &mut Surface,
                 shell: &mut XdgV6ShellSurface) {
    }
}
impl XdgV6ShellManagerHandler for XdgV6ShellManager {
    fn new_surface(&mut self,
                   compositor: &mut Compositor,
                   shell: &mut XdgV6ShellSurface,
                   surface: &mut Surface)
                   -> Option<Box<XdgV6ShellHandler>> {
        shell.ping();
        match shell.state() {
            Some(&mut XdgV6ShellState::TopLevel(ref mut toplevel)) => {
                toplevel.set_activated(true);
            }
            _ => {}
        }
        let seat_id = {
            let state: &mut State = compositor.into();
            state.shells.push(shell.weak_reference());
            for (mut output, _) in state.layout.outputs() {
                output.run(|output| output.schedule_frame()).unwrap();
            }
            state.seat_id.unwrap()
        };
        let seat = compositor.seats.get(seat_id).expect("invalid seat id");
        let mut keyboard = seat.get_keyboard().expect("Seat did not have a keyboard set");
        keyboard.run(|keyboard| {
                         seat.keyboard_notify_enter(surface,
                                                    &mut keyboard.keycodes(),
                                                    &mut keyboard.get_modifier_masks())
                     })
                .unwrap();
        Some(Box::new(XdgV6ShellHandlerEx))
    }

    fn surface_destroyed(&mut self,
                         compositor: &mut Compositor,
                         shell: &mut XdgV6ShellSurface,
                         _: &mut Surface) {
        let state: &mut State = compositor.into();
        let weak = shell.weak_reference();
        if let Some(index) = state.shells.iter().position(|s| *s == weak) {
            state.shells.remove(index);
        }
    }
}

struct OutputManager;

struct ExOutput;

struct InputManager;

struct ExPointer;

struct ExKeyboardHandler;

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
            } else if key_event.key_state() == WLR_KEY_PRESSED {
                if key == KEY_F1 {
                    thread::spawn(move || {
                                      Command::new("weston-terminal").output().unwrap();
                                  });
                    return
                }
            }
        }
        let state: &mut State = compositor.data.downcast_mut().unwrap();
        let seat_id = state.seat_id.unwrap();
        let seat = compositor.seats.get(seat_id).expect("invalid seat id");
        seat.keyboard_notify_key(key_event.time_msec(),
                                 key_event.keycode(),
                                 key_event.key_state() as u32);
    }
}

impl PointerHandler for ExPointer {
    fn on_motion(&mut self, compositor: &mut Compositor, _: &mut Pointer, event: &MotionEvent) {
        let state: &mut State = compositor.into();
        let (delta_x, delta_y) = event.delta();
        state.layout
             .cursor(state.cursor_id)
             .unwrap()
             .move_to(event.device(), delta_x, delta_y);
    }
}

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut Output) {
        let state: &mut State = compositor.data.downcast_mut().unwrap();
        if state.shells.len() < 1 {
            return
        }
        let renderer = compositor.renderer
                                 .as_mut()
                                 .expect("Compositor was not loaded with a renderer");
        render_shells(state, &mut renderer.render(output));
    }
}

impl InputManagerHandler for InputManager {
    fn pointer_added(&mut self,
                     _: &mut Compositor,
                     _: &mut Pointer)
                     -> Option<Box<PointerHandler>> {
        Some(Box::new(ExPointer))
    }

    fn keyboard_added(&mut self,
                      compositor: &mut Compositor,
                      keyboard: &mut Keyboard)
                      -> Option<Box<KeyboardHandler>> {
        let seat_id = {
            let state: &mut State = compositor.into();
            state.keyboard = Some(keyboard.weak_reference());
            state.seat_id.unwrap()
        };
        let seat = compositor.seats.get(seat_id).expect("Invalid seat id");
        seat.set_keyboard(keyboard.input_device());
        Some(Box::new(ExKeyboardHandler))
    }
}

fn main() {
    init_logging(L_DEBUG, None);
    let cursor = CursorBuilder::new(Box::new(CursorEx)).expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let mut layout = OutputLayout::new().expect("Could not construct an output layout");

    let cursor_id = layout.attach_cursor(cursor);
    let mut compositor =
        CompositorBuilder::new().gles2(true)
                                .input_manager(Box::new(InputManager))
                                .output_manager(Box::new(OutputManager))
                                .xdg_shell_v6_manager(Box::new(XdgV6ShellManager))
                                .build_auto(State::new(xcursor_theme, layout, cursor_id));

    {
        let seat_id = {
            let seat = Seat::create(&mut compositor, "Main Seat".into(), Box::new(SeatHandlerEx))
                .expect("Could not allocate the global seat");
            seat.set_capabilities(Capability::all());
            seat.id()
        };
        let state: &mut State = (&mut compositor).into();
        state.seat_id = Some(seat_id);
    }
    compositor.run();
}

/// Render the shells in the current compositor state on the given output.
fn render_shells(state: &mut State, renderer: &mut Renderer) {
    let shells = state.shells.clone();
    for mut shell in shells {
        shell.run(|shell| {
                      shell.surface()
                           .run(|surface| {
                                    let (width, height) = surface.current_state().size();
                                    let (render_width, render_height) =
                                        (width * renderer.output.scale() as i32,
                                        height * renderer.output.scale() as i32);
                                    let (lx, ly) = (0.0, 0.0);
                                    let render_box = Area::new(Origin::new(lx as i32, ly as i32),
                                                               Size::new(render_width,
                                                                         render_height));
                                    if state.layout.intersects(renderer.output, render_box) {
                                        let transform = renderer.output.get_transform().invert();
                                        let matrix = project_box(render_box,
                                                                 transform,
                                                                 0.0,
                                                                 renderer.output
                                                                         .transform_matrix());
                                        renderer.render_texture_with_matrix(&surface.texture(),
                                                                            matrix);
                                        let start = SystemTime::now();
                                        let now = start.duration_since(UNIX_EPOCH)
                                            .expect("Time went backwards");
                                        surface.send_frame_done(now);
                                    }
                                })
                           .unwrap()
                  })
             .unwrap();
    }
}
