#[macro_use]
extern crate wlroots;

use std::{thread, process::Command};

use wlroots::{area::{Area, Origin, Size},
              compositor,
              cursor::{self, Cursor, xcursor},
              input::{self, keyboard, pointer},
              output::{self, layout::Layout},
              render::{matrix, Renderer},
              seat::{self, Seat},
              shell::xdg_shell_v6,
              surface,
              utils::{Handleable, log::{init_logging, WLR_DEBUG}, current_time}};
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::keysyms::{KEY_Escape, KEY_F1};
use wlroots::wlroots_dehandle;

struct State {
    xcursor_manager: xcursor::Manager,
    keyboard: Option<keyboard::Handle>,
    layout: output::layout::Handle,
    cursor: cursor::Handle,
    shells: Vec<xdg_shell_v6::Handle>,
    seat_handle: Option<seat::Handle>
}

impl State {
    fn new(xcursor_manager: xcursor::Manager,
           layout: output::layout::Handle,
           cursor: cursor::Handle)
           -> Self {
        State { xcursor_manager,
                layout,
                cursor,
                keyboard: None,
                seat_handle: None,
                shells: vec![] }
    }
}

compositor_data!(State);

struct SeatHandlerEx;

struct CursorEx;

impl cursor::Handler for CursorEx {}

impl seat::Handler for SeatHandlerEx {}

struct XdgV6ShellHandlerEx;
struct XdgV6ShellManager;
struct OutputLayoutEx;

impl output::layout::Handler for OutputLayoutEx {}

impl xdg_shell_v6::Handler for XdgV6ShellHandlerEx {
    fn destroyed(&mut self, compositor: compositor::Handle, shell: xdg_shell_v6::Handle) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let weak = shell;
            if let Some(index) = state.shells.iter().position(|s| *s == weak) {
                state.shells.remove(index);
            }
        }).unwrap();
    }
}


struct SurfaceEx;

impl surface::Handler for SurfaceEx {
    fn on_commit(&mut self, _: compositor::Handle, surface: surface::Handle) {
        wlr_log!(WLR_DEBUG, "Commiting for surface {:?}", surface);
    }
}

impl xdg_shell_v6::ManagerHandler for XdgV6ShellManager {
    #[wlroots_dehandle(compositor, shell, layout, output)]
    fn new_surface(&mut self,
                   compositor: compositor::Handle,
                   shell: xdg_shell_v6::Handle)
                   -> (Option<Box<xdg_shell_v6::Handler>>, Option<Box<surface::Handler>>) {
        {
            use compositor as compositor;
            use shell as shell;
            shell.ping();
            let state: &mut State = compositor.into();
            state.shells.push(shell.weak_reference());
            let layout_handle = &state.layout;
            use layout_handle as layout;
            for (output, _) in layout.outputs() {
                use output as output;
                output.schedule_frame()
            }
        }
        (Some(Box::new(XdgV6ShellHandlerEx)), Some(Box::new(SurfaceEx)))
    }
}

struct ExOutput;

struct ExPointer;

struct ExKeyboardHandler;

#[wlroots_dehandle(compositor, output, layout, cursor)]
fn output_added<'output>(compositor: compositor::Handle,
                         builder: output::Builder<'output>)
                         -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(ExOutput);
    {
        let output_handle = &result.output;
        use compositor as compositor;
        use output_handle as output;
        let state: &mut State = compositor.data.downcast_mut().unwrap();
        let xcursor_manager = &mut state.xcursor_manager;
        // TODO use output config if present instead of auto
        let layout = &mut state.layout;
        let cursor = &mut state.cursor;
        use layout as layout;
        use cursor as cursor;
        layout.add_auto(output);
        cursor.attach_output_layout(layout);
        xcursor_manager.load(output.scale());
        xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
        let (x, y) = cursor.coords();
        // https://en.wikipedia.org/wiki/Mouse_warping
        cursor.warp(None, x, y);
    }
    Some(result)
}

impl keyboard::Handler for ExKeyboardHandler {
    #[wlroots_dehandle(compositor, seat)]
    fn on_key(&mut self,
              compositor_handle: compositor::Handle,
              _: keyboard::Handle,
              key_event: &keyboard::event::Key) {
        for key in key_event.pressed_keys() {
            if key == KEY_Escape {
                compositor::terminate();
            } else if key_event.key_state() == WLR_KEY_PRESSED {
                if key == KEY_F1 {
                    thread::spawn(move || {
                        Command::new("weston-terminal").output().unwrap();
                    });
                    return
                }
            }
        };
        use compositor_handle as compositor;
        let state: &mut State = compositor.into();
        let seat_handle = state.seat_handle.clone().unwrap();
        use seat_handle as seat;
        seat.keyboard_notify_key(key_event.time_msec(),
                                    key_event.keycode(),
                                    key_event.key_state() as u32);
    }
}

impl pointer::Handler for ExPointer {
    #[wlroots_dehandle(compositor, cursor)]
    fn on_motion_absolute(&mut self,
                          compositor: compositor::Handle,
                          _: pointer::Handle,
                          event: &pointer::event::AbsoluteMotion) {
        use compositor as compositor;
        let state: &mut State = compositor.into();
        let (x, y) = event.pos();
        let cursor_handle = &state.cursor;
        use cursor_handle as cursor;
        cursor.warp_absolute(event.device(), x, y)
    }

    #[wlroots_dehandle(compositor, cursor)]
    fn on_motion(&mut self,
                 compositor: compositor::Handle,
                 _: pointer::Handle,
                 event: &pointer::event::Motion) {
        use compositor as compositor;
        let state: &mut State = compositor.into();
        let (delta_x, delta_y) = event.delta();
        let cursor_handle = &state.cursor;
        use cursor_handle as cursor;
        cursor.move_to(event.device(), delta_x, delta_y)
    }

    #[wlroots_dehandle(compositor, shell, seat, keyboard, surface)]
    fn on_button(&mut self,
                 compositor: compositor::Handle, _:
                 pointer::Handle,
                 _: &pointer::event::Button) {
        use compositor as compositor;
        let state: &mut State = compositor.into();
        let seat = state.seat_handle.clone().unwrap();
        let keyboard = state.keyboard.clone().unwrap();
        let shell_handle = &state.shells[0];
        use shell_handle as shell;
        match shell.state() {
            Some(&mut xdg_shell_v6::ShellState::TopLevel(ref mut toplevel)) => {
                toplevel.set_activated(true);
            }
            _ => {}
        };
        let surface = shell.surface();
        use seat as seat;
        use keyboard as keyboard;
        use surface as surface;
        seat.set_keyboard(keyboard.input_device());
        seat.keyboard_notify_enter(surface,
                                   &mut keyboard.keycodes(),
                                   &mut keyboard.get_modifier_masks())
    }
}

impl output::Handler for ExOutput {
    #[wlroots_dehandle(compositor, output)]
    fn on_frame(&mut self, compositor: compositor::Handle, output: output::Handle) {
        use compositor as compositor;
        use output as output;
        let state: &mut State = compositor.data.downcast_mut().unwrap();
        let renderer = compositor.renderer
            .as_mut()
            .expect("Compositor was not loaded with a renderer");
        let mut render_context = renderer.render(output, None);
        render_context.clear([0.25, 0.25, 0.25, 1.0]);
        render_shells(state, &mut render_context)
    }
}

fn pointer_added(_: compositor::Handle,
                 _: pointer::Handle)
                 -> Option<Box<pointer::Handler>> {
    Some(Box::new(ExPointer))
}

#[wlroots_dehandle(compositor, keyboard, seat)]
fn keyboard_added(compositor: compositor::Handle,
                  keyboard: keyboard::Handle)
                  -> Option<Box<keyboard::Handler>> {
    {
        use compositor as compositor;
        use keyboard as keyboard;
        let state: &mut State = compositor.into();
        state.keyboard = Some(keyboard.weak_reference());
        let seat_handle = state.seat_handle.as_ref().unwrap();
        use seat_handle as seat;
        seat.set_keyboard(keyboard.input_device());
    }
    Some(Box::new(ExKeyboardHandler))
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let cursor = Cursor::create(Box::new(CursorEx));
    let mut xcursor_manager =
        xcursor::Manager::create("default".to_string(), 24).expect("Could not create xcursor \
                                                                    manager");
    xcursor_manager.load(1.0);
    cursor.run(|c| xcursor_manager.set_cursor_image("left_ptr".to_string(), c))
        .unwrap();
    let layout = Layout::create(Box::new(OutputLayoutEx));

    let output_builder = output::ManagerBuilder::default().output_added(output_added);
    let input_builder = input::ManagerBuilder::default()
        .keyboard_added(keyboard_added)
        .pointer_added(pointer_added);
    let mut compositor =
        compositor::Builder::new().gles2(true)
        .wl_shm(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .xdg_shell_v6_manager(Box::new(XdgV6ShellManager))
        .build_auto(State::new(xcursor_manager, layout, cursor));

    {
        let seat_handle =
            Seat::create(&mut compositor, "seat0".into(), Box::new(SeatHandlerEx));
        seat_handle.run(|seat| {
                            seat.set_capabilities(seat::Capability::all());
                        })
                   .unwrap();
        let state: &mut State = (&mut compositor).into();
        state.seat_handle = Some(seat_handle);
    }
    compositor.run();
}

/// Render the shells in the current compositor state on the given output.
#[wlroots_dehandle(shell, surface, layout)]
fn render_shells(state: &mut State, renderer: &mut Renderer) {
    let shells = state.shells.clone();
    let layout_handle = &state.layout;
    for mut shell in shells {
        use shell as shell;
        let surface_handle = shell.surface();
        use surface_handle as surface;
        use layout_handle as layout;
        let (width, height) = surface.current_state().size();
        let (render_width, render_height) =
            (width * renderer.output.scale() as i32,
             height * renderer.output.scale() as i32);
        let (lx, ly) = (0.0, 0.0);
        let render_box = Area::new(Origin::new(lx as i32, ly as i32),
                                   Size::new(render_width,
                                             render_height));
        if layout.intersects(renderer.output, render_box) {
            let transform = renderer.output.get_transform().invert();
            let matrix = matrix::project_box(render_box,
                                             transform,
                                             0.0,
                                             renderer.output
                                             .transform_matrix());
            if let Some(texture) = surface.texture().as_ref() {
                renderer.render_texture_with_matrix(texture, matrix);
            }
            surface.send_frame_done(current_time());
        }
    }
}
