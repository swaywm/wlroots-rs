#[macro_use]
extern crate wlroots;

use std::f64::consts::PI;

use wlroots::{area::{Area, Size, Origin}, compositor,
              input::{self, keyboard, tablet_tool, tablet_pad},
              output, render::matrix,
              wlroots_sys::wl_output_transform::WL_OUTPUT_TRANSFORM_NORMAL,
              wlr_tablet_tool_proximity_state::*,
              wlr_button_state::*,
              utils::log,
              xkbcommon::xkb::KEY_Escape};

#[derive(Debug, Default)]
struct State {
    proximity: bool,
    tap: bool,
    button: bool,
    distance: f64,
    pressure: f64,
    pos: (f64, f64),
    tilt: (f64, f64),
    size_mm: (f64, f64),
    ring: f64,
    tool_color: [f32; 4],
    pad_color: [f32; 4]
}

impl State {
    fn new() -> Self {
        State { tool_color: [1.0, 1.0, 1.0, 1.0],
                pad_color: [0.5, 0.5, 0.5, 1.0],
                ..State::default() }
    }
}

struct OutputEx;
struct KeyboardEx;
struct TabletEx;

fn keyboard_added(_: compositor::Handle,
                  _: keyboard::Handle)
                  -> Option<Box<keyboard::Handler>> {
    Some(Box::new(KeyboardEx))
}

fn tablet_tool_added(compositor: compositor::Handle,
                     tool: tablet_tool::Handle)
                     -> Option<Box<tablet_tool::Handler>> {
    with_handles!([(compositor: {compositor}), (tool: {tool})] => {
        let state: &mut State = compositor.into();
        state.size_mm = tool.input_device().size();
        if state.size_mm.0 == 0.0 {
            state.size_mm.0 = 20.0;
        }
        if state.size_mm.1 == 0.0 {
            state.size_mm.1 = 10.0;
        }
    }).unwrap();
    Some(Box::new(TabletEx))
}

fn tablet_pad_added(_: compositor::Handle,
                    _: tablet_pad::Handle)
                    -> Option<Box<tablet_pad::Handler>> {
    Some(Box::new(TabletEx))
}

fn output_added<'output>(_: compositor::Handle,
                         builder: output::Builder<'output>)
                         -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(OutputEx);
    Some(result)
}

impl keyboard::Handler for KeyboardEx {
    fn on_key(&mut self,
              _: compositor::Handle,
              _: keyboard::Handle,
              key_event: &keyboard::event::Key) {
        for key in key_event.pressed_keys() {
            if key == KEY_Escape {
                compositor::terminate()
            }
        }
    }
}

impl tablet_pad::Handler for TabletEx {
    fn on_button(&mut self,
                 compositor: compositor::Handle,
                 _: tablet_pad::Handle,
                 event: &tablet_pad::event::Button) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            if event.state() == WLR_BUTTON_RELEASED {
                state.pad_color = [0.5, 0.5, 0.5, 1.0];
            } else {
                for i in 0..3 {
                    state.pad_color[i] = if event.button() % 3 == i as u32 {
                        0.0
                    } else {
                        1.0
                    }
                }
            }
        }).unwrap();
    }

    fn on_ring(&mut self,
               compositor: compositor::Handle,
               _: tablet_pad::Handle,
               event: &tablet_pad::event::Ring) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let position = event.position();
            if position != -1.0 {
                state.ring = -(position * PI / 180.0)
            }
        }).unwrap();
    }
}

impl tablet_tool::Handler for TabletEx {
    fn on_axis(&mut self,
               compositor: compositor::Handle,
               _: tablet_tool::Handle,
               event: &tablet_tool::event::Axis) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let axis = event.updated_axes();
            let (x, y) = event.position();
            let (tilt_x, tilt_y) = event.tilt();
            if axis.contains(tablet_tool::Axis::WLR_TABLET_TOOL_AXIS_X) {
                state.pos.0 = x
            }
            if axis.contains(tablet_tool::Axis::WLR_TABLET_TOOL_AXIS_Y) {
                state.pos.1 = y
            }
            if axis.contains(tablet_tool::Axis::WLR_TABLET_TOOL_AXIS_DISTANCE) {
                state.distance = event.distance()
            }
            if axis.contains(tablet_tool::Axis::WLR_TABLET_TOOL_AXIS_PRESSURE) {
                state.pressure = event.pressure()
            }
            if axis.contains(tablet_tool::Axis::WLR_TABLET_TOOL_AXIS_TILT_X) {
                state.tilt.0 = tilt_x
            }
            if axis.contains(tablet_tool::Axis::WLR_TABLET_TOOL_AXIS_TILT_Y) {
                state.tilt.1 = tilt_y
            }
        }).unwrap();
    }

    fn on_proximity(&mut self,
                    compositor: compositor::Handle,
                    _: tablet_tool::Handle,
                    event: &tablet_tool::event::Proximity) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            state.proximity = event.state() == WLR_TABLET_TOOL_PROXIMITY_IN
        }).unwrap();
    }

    fn on_button(&mut self,
                 compositor: compositor::Handle,
                 _: tablet_tool::Handle,
                 event: &tablet_tool::event::Button) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            if event.state() == WLR_BUTTON_RELEASED {
                state.button = false;
            } else {
                state.button = true;
                for i in 0..3 {
                    state.tool_color[i] = if event.button() % 3 == i as u32 {
                        0.0
                    } else {
                        1.0
                    };
                }
            }
        }).unwrap();
    }
}

impl output::Handler for OutputEx {
    fn on_frame(&mut self, compositor: compositor::Handle, output: output::Handle) {
        with_handles!([(compositor: {compositor}), (output: {output})] => {
            let state: &mut State = compositor.data.downcast_mut().unwrap();
            let (width, height) = output.effective_resolution();
            let renderer = compositor.renderer
                .as_mut()
                .expect("Compositor was not loaded with a renderer");
            let mut renderer = renderer.render(output, None);
            renderer.clear([0.25, 0.25, 0.25, 1.0]);
            let tool_color: [f32; 4] = if state.button {
                state.tool_color.clone()
            } else {
                let distance: f64 = 0.8 * (1.0 - state.distance);
                [distance as f32, distance as f32, distance as f32, 1.0]
            };
            let scale = 4.0;
            let pad_width = (state.size_mm.0 * scale) as f32;
            let pad_height = (state.size_mm.1 * scale) as f32;
            let left: f32 = (width as f64 / 2.0 - pad_width as f64 / 2.0) as f32;
            let top: f32 = (height as f64 / 2.0 - pad_height as f64 / 2.0) as f32;
            let area = Area::new(Origin::new(left as i32, top as i32),
                                 Size::new(pad_width as i32, pad_height as i32));
            let transform_matrix = renderer.output.transform_matrix();
            renderer.render_colored_rect(area, state.pad_color, transform_matrix.clone());
            if state.proximity {
                let origin =
                    Origin { x: ((state.pos.0 * pad_width as f64) - 8.0 * (state.pressure + 1.0)
                                 + left as f64) as i32,
                             y: ((state.pos.1 * pad_height as f64) - 8.0 * (state.pressure + 1.0)
                                 + top as f64) as i32 };
                let size = Size { width: (16.0 * (state.pressure + 1.0)) as i32,

                                  height: (16.0 * (state.pressure + 1.0)) as i32 };
                let mut area = Area { origin, size };
                let matrix = matrix::project_box(area,
                                                 WL_OUTPUT_TRANSFORM_NORMAL,
                                                 state.ring as _,
                                                 transform_matrix.clone());
                renderer.render_colored_quad(tool_color, matrix);

                area.origin.x += state.tilt.0 as i32;
                area.origin.y += state.tilt.1 as i32;
                area.size.width /= 2;
                area.size.width /= 2;
                renderer.render_colored_rect(area, tool_color, transform_matrix);
            }
        }).unwrap();
    }
}

fn main() {
    log::init_logging(log::WLR_DEBUG, None);
    let output_builder = output::manager::Builder::default().output_added(output_added);
    let input_builder = input::manager::Builder::default()
        .keyboard_added(keyboard_added)
        .tablet_tool_added(tablet_tool_added)
        .tablet_pad_added(tablet_pad_added);
    compositor::Builder::new().gles2(true)
                            .input_manager(input_builder)
                            .output_manager(output_builder)
                            .build_auto(State::new())
                            .run()
}
