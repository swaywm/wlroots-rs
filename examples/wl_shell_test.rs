extern crate byteorder;
extern crate tempfile;
#[macro_use]
extern crate wayland_client;
#[macro_use]
extern crate wlroots;

use std::thread;
use std::time::Duration;

use wlroots::{matrix_mul, matrix_rotate, matrix_scale, matrix_translate, Area, Compositor,
              CompositorBuilder, CursorBuilder, InputManagerHandler, Keyboard, KeyboardHandler,
              Origin, Output, OutputBuilder, OutputBuilderResult, OutputHandler, OutputLayout,
              OutputManagerHandler, Pointer, PointerHandler, Renderer, Seat, SeatHandler, Size,
              Surface, WlShellHandler, WlShellManagerHandler, WlShellSurface,
              WlShellSurfaceHandle, XCursorTheme};
use wlroots::key_events::KeyEvent;
use wlroots::pointer_events::{AxisEvent, ButtonEvent, MotionEvent};
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct State {
    color: [f32; 4],
    default_color: [f32; 4],
    xcursor_theme: XCursorTheme,
    layout: OutputLayout,
    shells: Vec<WlShellSurfaceHandle>,
    seat: Option<Box<Seat>>
}

impl State {
    fn new(xcursor_theme: XCursorTheme, layout: OutputLayout) -> Self {
        State { color: [0.25, 0.25, 0.25, 1.0],
                default_color: [0.25, 0.25, 0.25, 1.0],
                xcursor_theme,
                layout,
                shells: vec![],
                seat: None }
    }
}

compositor_data!(State);

struct SeatHandlerEx;

impl SeatHandler for SeatHandlerEx {
    // TODO
}

struct WlShellHandlerEx;
struct WlShellManager;

impl WlShellHandler for WlShellHandlerEx {}
impl WlShellManagerHandler for WlShellManager {
    fn new_surface(&mut self,
                   compositor: &mut Compositor,
                   shell: &mut WlShellSurface,
                   _: &mut Surface)
                   -> Option<Box<WlShellHandler>> {
        let state: &mut State = compositor.into();
        state.shells.push(shell.weak_reference());
        Some(Box::new(WlShellHandlerEx))
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
        let cursor = &mut state.layout.cursors()[0];
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
            // TODO This is a dumb way to compare these values
            else if key_event.key_state() as u32 == 1 {
                thread::spawn(move || {
                    use byteorder::{NativeEndian, WriteBytesExt};
                    use std::cmp::min;
                    use std::io::Write;
                    use std::os::unix::io::AsRawFd;
                    use wayland_client::EnvHandler;
                    use wayland_client::protocol::{wl_compositor, wl_pointer, wl_seat, wl_shell,
                                                   wl_shell_surface, wl_shm};

                    wayland_env!(WaylandEnv,
                                 compositor: wl_compositor::WlCompositor,
                                 seat: wl_seat::WlSeat,
                                 shell: wl_shell::WlShell,
                                 shm: wl_shm::WlShm);

                    fn pointer_impl() -> wl_pointer::Implementation<()> {
                        wl_pointer::Implementation {
                            enter: |_, _, _pointer, _serial, _surface, x, y| {
                                println!("Pointer entered surface at ({},{}).", x, y);
                            },
                            leave: |_, _, _pointer, _serial, _surface| {
                                println!("Pointer left surface.");
                            },
                            motion: |_, _, _pointer, _time, x, y| {
                                println!("Pointer moved to ({},{}).", x, y);
                            },
                            button: |_, _, _pointer, _serial, _time, button, state| {
                                println!(
                                    "Button {} ({}) was {:?}.",
                                    match button {
                                        272 => "Left",
                                        273 => "Right",
                                        274 => "Middle",
                                        _ => "Unknown",
                                    },
                                    button,
                                    state
                                );
                            },
                            axis: |_, _, _, _, _, _| { /* not used in this example */ },
                            frame: |_, _, _| { /* not used in this example */ },
                            axis_source: |_, _, _, _| { /* not used in this example */ },
                            axis_discrete: |_, _, _, _, _| { /* not used in this example */ },
                            axis_stop: |_, _, _, _, _| { /* not used in this example */ },
                        }
                    }

                    fn shell_surface_impl() -> wl_shell_surface::Implementation<()> {
                        wl_shell_surface::Implementation { ping: |_, _, shell_surface, serial| {
                                                               shell_surface.pong(serial);
                                                           },
                                                           configure: |_, _, _, _, _, _| {
                                                               /* not used in this example */
                                                           },
                                                           popup_done: |_, _, _| {
                                                               /* not used in this example */
                                                           } }
                    }

                    let (display, mut event_queue) = match wayland_client::default_connect() {
                        Ok(ret) => ret,
                        Err(e) => panic!("Cannot connect to wayland server: {:?}", e)
                    };

                    let registry = display.get_registry();

                    let env_token = EnvHandler::<WaylandEnv>::init(&mut event_queue, &registry);

                    event_queue.sync_roundtrip().unwrap();

                    // buffer (and window) width and height
                    let buf_x: u32 = 320;
                    let buf_y: u32 = 240;

                    // create a tempfile to write the conents of the window on
                    let mut tmp = tempfile::tempfile().ok()
                                                      .expect("Unable to create a tempfile.");
                    // write the contents to it, lets put a nice color gradient
                    for i in 0..(buf_x * buf_y) {
                        let x = (i % buf_x) as u32;
                        let y = (i / buf_x) as u32;
                        let r: u32 =
                            min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
                        let g: u32 = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
                        let b: u32 = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
                        let _ =
                            tmp.write_u32::<NativeEndian>((0xFF << 24) + (r << 16) + (g << 8) + b);
                    }
                    let _ = tmp.flush();

                    // retrieve the env
                    let env = event_queue.state().get(&env_token).clone_inner().unwrap();

                    // prepare the wayland surface
                    let surface = env.compositor.create_surface();
                    let shell_surface = env.shell.get_shell_surface(&surface);

                    let pool = env.shm.create_pool(tmp.as_raw_fd(), (buf_x * buf_y * 4) as i32);
                    // match a buffer on the part we wrote on
                    let buffer = pool.create_buffer(
                        0,
                        buf_x as i32,
                        buf_y as i32,
                        (buf_x * 4) as i32,
                        wl_shm::Format::Argb8888,
                    ).expect("The pool cannot be already dead");

                    // make our surface as a toplevel one
                    shell_surface.set_toplevel();
                    // attach the buffer to it
                    surface.attach(Some(&buffer), 0, 0);
                    // commit
                    surface.commit();

                    let pointer = env.seat.get_pointer()
                                     .expect("Seat cannot be already destroyed.");

                    event_queue.register(&shell_surface, shell_surface_impl(), ());
                    event_queue.register(&pointer, pointer_impl(), ());

                    loop {
                        display.flush().unwrap();
                        event_queue.dispatch().unwrap();
                    }
                });
            }
        }
    }
}

impl PointerHandler for ExPointer {
    fn on_motion(&mut self, compositor: &mut Compositor, _: &mut Pointer, event: &MotionEvent) {
        let state: &mut State = compositor.into();
        let (delta_x, delta_y) = event.delta();
        state.layout.cursors()[0].move_to(event.device(), delta_x, delta_y);
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
        let renderer = compositor.renderer
                                 .as_mut()
                                 .expect("Compositor was not loaded with a renderer");
        let state: &mut State = compositor.data.downcast_mut().unwrap();
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
                      _: &mut Compositor,
                      _: &mut Keyboard)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

fn main() {
    init_logging(L_DEBUG, None);
    let cursor = CursorBuilder::new().expect("Could not create cursor");
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let mut layout = OutputLayout::new().expect("Could not construct an output layout");

    layout.attach_cursor(cursor);
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .build_auto(State::new(xcursor_theme, layout),
                                                             Some(Box::new(InputManager)),
                                                             Some(Box::new(OutputManager)),
                                                             Some(Box::new(WlShellManager)));
    let seat = Seat::create(&mut compositor, "Main Seat".into(), Box::new(SeatHandlerEx))
        .expect("Could not allocate the global seat");
    {
        let state: &mut State = (&mut compositor).into();
        state.seat = Some(seat);
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
                                    let (width, height) = {
                                        let current_state = surface.current_state();
                                        (current_state.width() as i32,
                                        current_state.height() as i32)
                                    };
                                    let (render_width, render_height) =
                                        (width * renderer.output.scale() as i32,
                                        height * renderer.output.scale() as i32);
                                    // TODO Some value from something else?
                                    let (lx, ly) = (0.0, 0.0);
                                    let (mut ox, mut oy) = (lx, ly);
                                    state.layout
                                         .output_coords(renderer.output, &mut ox, &mut oy);
                                    ox *= renderer.output.scale() as f64;
                                    oy *= renderer.output.scale() as f64;
                                    let render_box = Area::new(Origin::new(lx as i32, ly as i32),
                                                               Size::new(render_width,
                                                                         render_height));
                                    if state.layout.intersects(renderer.output, render_box) {
                                        let mut matrix = [0.0; 16];
                                        let mut translate_center = [0.0; 16];
                                        matrix_translate(&mut translate_center,
                                                         (ox as i32 + render_width / 2) as f32,
                                                         (oy as i32 + render_height / 2) as f32,
                                                         0.0);
                                        let mut rotate = [0.0; 16];
                                        // TODO what is rotation
                                        let rotation = 0.0;
                                        matrix_rotate(&mut rotate, rotation);

                                        let mut translate_origin = [0.0; 16];
                                        matrix_translate(&mut translate_origin,
                                                         (-render_width / 2) as f32,
                                                         (-render_height / 2) as f32,
                                                         0.0);

                                        let mut scale = [0.0; 16];
                                        matrix_scale(&mut scale,
                                                     render_width as f32,
                                                     render_height as f32,
                                                     1.0);

                                        let mut transform = [0.0; 16];
                                        matrix_mul(&translate_center, &mut rotate, &mut transform);
                                        matrix_mul(&transform.clone(),
                                                   &mut translate_origin,
                                                   &mut transform);
                                        matrix_mul(&transform.clone(), &mut scale, &mut transform);

                                        // TODO Handle non transform normal on the output
                                        // if ... {}
                                        matrix_mul(&renderer.output.transform_matrix(),
                                                   &mut transform,
                                                   &mut matrix);
                                        renderer.render_with_matrix(&surface.texture(), &matrix);
                                        surface.send_frame_done(Duration::from_secs(1));
                                    }
                                })
                           .unwrap()
                  })
             .unwrap();
    }
}
