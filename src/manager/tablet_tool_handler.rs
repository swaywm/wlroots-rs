//! Handler for tablet tools

use {Compositor, InputDevice, TabletTool, compositor::COMPOSITOR_PTR};
use events::tablet_tool_events::{AxisEvent, ButtonEvent, ProximityEvent, TipEvent};
use libc;

pub trait TabletToolHandler {
    /// Callback that is triggered when an axis event fires
    fn on_axis(&mut self, &mut Compositor, &mut TabletTool, &AxisEvent) {}

    /// Callback that is triggered when a table tool is brought close to the
    /// input source.
    fn on_proximity(&mut self, &mut Compositor, &mut TabletTool, &ProximityEvent) {}

    /// Callback that is triggered when a table tool's tip touches the input
    /// source.
    fn on_tip(&mut self, &mut Compositor, &mut TabletTool, &TipEvent) {}

    /// Callback that is triggered when a button is pressed on the tablet tool.
    fn on_button(&mut self, &mut Compositor, &mut TabletTool, &ButtonEvent) {}
}

wayland_listener!(TabletToolWrapper, (TabletTool, Box<TabletToolHandler>), [
    axis_listener => axis_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut tool, ref mut handler) = this.data;
        let event = AxisEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        tool.set_lock(true);
        handler.on_axis(compositor, tool, &event);
        tool.set_lock(false);
        compositor.lock.set(false);
    };
    proximity_listener => proximity_notify: |this: &mut TabletToolWrapper,
    data: *mut libc::c_void,|
    unsafe {
        let (ref mut tool, ref mut handler) = this.data;
        let event = ProximityEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        tool.set_lock(true);
        handler.on_proximity(compositor, tool, &event);
        tool.set_lock(false);
        compositor.lock.set(false);
    };
    tip_listener => tip_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut tool, ref mut handler) = this.data;
        let event = TipEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        tool.set_lock(true);
        handler.on_tip(compositor, tool, &event);
        tool.set_lock(false);
        compositor.lock.set(false);
    };
    button_listener => button_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref mut tool, ref mut handler) = this.data;
        let event = ButtonEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        tool.set_lock(true);
        handler.on_button(compositor, tool, &event);
        tool.set_lock(false);
        compositor.lock.set(false);
    };
]);

impl TabletToolWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub fn tablet_tool(&mut self) -> &mut TabletTool {
        &mut self.data.0
    }
}
