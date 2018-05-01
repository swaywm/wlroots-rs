#[macro_use]
extern crate wlroots;
use std::f64::consts::PI;
use wlroots::{tablet_pad_events, tablet_tool_events, *, key_events::*, utils::*,
              wlroots_sys::wl_output_transform::WL_OUTPUT_TRANSFORM_NORMAL,
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

struct OutputManagerEx;
struct InputManagerEx;
struct OutputEx;
struct KeyboardEx;
struct TabletEx;

impl InputManagerHandler for InputManagerEx {
    fn keyboard_added(&mut self,
                      _: CompositorHandle,
                      _: KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(KeyboardEx))
    }

    fn tablet_tool_added(&mut self,
                         compositor: CompositorHandle,
                         tool: TabletToolHandle)
                         -> Option<Box<TabletToolHandler>> {
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

    fn tablet_pad_added(&mut self,
                        _: CompositorHandle,
                        _: TabletPadHandle)
                        -> Option<Box<TabletPadHandler>> {
        Some(Box::new(TabletEx))
    }
}

impl OutputManagerHandler for OutputManagerEx {
    fn output_added<'output>(&mut self,
                             _: CompositorHandle,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let result = builder.build_best_mode(OutputEx);
        Some(result)
    }
}

impl KeyboardHandler for KeyboardEx {
    fn on_key(&mut self,
              mut compositor: CompositorHandle,
              _: KeyboardHandle,
              key_event: &KeyEvent) {
        for key in key_event.pressed_keys() {
            if key == KEY_Escape {
                with_handles!([(compositor: {&mut compositor})] => {
                    compositor.terminate()
                }).unwrap();
            }
        }
    }
}

impl TabletPadHandler for TabletEx {
    fn on_button(&mut self,
                 compositor: CompositorHandle,
                 _: TabletPadHandle,
                 event: &tablet_pad_events::ButtonEvent) {
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
               compositor: CompositorHandle,
               _: TabletPadHandle,
               event: &tablet_pad_events::RingEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let position = event.position();
            if position != -1.0 {
                state.ring = -(position * PI / 180.0)
            }
        }).unwrap();
    }
}

impl TabletToolHandler for TabletEx {
    fn on_axis(&mut self,
               compositor: CompositorHandle,
               _: TabletToolHandle,
               event: &tablet_tool_events::AxisEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let axis = event.updated_axes();
            let (x, y) = event.position();
            let (tilt_x, tilt_y) = event.tilt();
            use tablet_tool_events::TabletToolAxis;
            if axis.contains(TabletToolAxis::WLR_TABLET_TOOL_AXIS_X) {
                state.pos.0 = x
            }
            if axis.contains(TabletToolAxis::WLR_TABLET_TOOL_AXIS_Y) {
                state.pos.1 = y
            }
            if axis.contains(TabletToolAxis::WLR_TABLET_TOOL_AXIS_DISTANCE) {
                state.distance = event.distance()
            }
            if axis.contains(TabletToolAxis::WLR_TABLET_TOOL_AXIS_PRESSURE) {
                state.pressure = event.pressure()
            }
            if axis.contains(TabletToolAxis::WLR_TABLET_TOOL_AXIS_TILT_X) {
                state.tilt.0 = tilt_x
            }
            if axis.contains(TabletToolAxis::WLR_TABLET_TOOL_AXIS_TILT_Y) {
                state.tilt.1 = tilt_y
            }
        }).unwrap();
    }

    fn on_proximity(&mut self,
                    compositor: CompositorHandle,
                    _: TabletToolHandle,
                    event: &tablet_tool_events::ProximityEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            state.proximity = event.state() == WLR_TABLET_TOOL_PROXIMITY_IN
        }).unwrap();
    }

    fn on_button(&mut self,
                 compositor: CompositorHandle,
                 _: TabletToolHandle,
                 event: &tablet_tool_events::ButtonEvent) {
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

impl OutputHandler for OutputEx {
    fn on_frame(&mut self, compositor: CompositorHandle, output: OutputHandle) {
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
                let matrix = project_box(area,
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

compositor_data!(State);

fn main() {
    init_logging(L_DEBUG, None);
    CompositorBuilder::new().gles2(true)
                            .input_manager(Box::new(InputManagerEx))
                            .output_manager(Box::new(OutputManagerEx))
                            .build_auto(State::new())
                            .run()
}
