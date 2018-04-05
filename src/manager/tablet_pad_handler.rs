//! Handler for tablet pads

use {Compositor, InputDevice, TabletPad, compositor::COMPOSITOR_PTR};
use events::tablet_pad_events::{ButtonEvent, RingEvent, StripEvent};
use libc;

pub trait TabletPadHandler {
    /// Callback that is triggered when a button is pressed on the tablet pad.
    fn on_button(&mut self, &mut Compositor, &mut TabletPad, &ButtonEvent) {}

    /// Callback that is triggered when the touch strip is used.
    fn on_strip(&mut self, &mut Compositor, &mut TabletPad, &StripEvent) {}

    /// Callback that is triggered when the ring is touched.
    fn on_ring(&mut self, &mut Compositor, &mut TabletPad, &RingEvent) {}
}

wayland_listener!(TabletPadWrapper, (TabletPad, Box<TabletPadHandler>), [
    button_listener => button_notify: |this: &mut TabletPadWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref mut pad, ref mut handler) = this.data;
        let event = ButtonEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;
        pad.set_lock(true);
        handler.on_button(compositor, pad, &event);
        pad.set_lock(false);
    };
    strip_listener => strip_notify: |this: &mut TabletPadWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref mut pad, ref mut handler) = this.data;
        let event = StripEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;
        pad.set_lock(true);
        handler.on_strip(compositor, pad, &event);
        pad.set_lock(false);
    };
    ring_listener => ring_notify: |this: &mut TabletPadWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref mut pad, ref mut handler) = this.data;
        let event = RingEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;
        pad.set_lock(true);
        handler.on_ring(compositor, pad, &event);
        pad.set_lock(false);
    };
]);

impl TabletPadWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub(crate) fn tablet_pad(&mut self) -> &mut TabletPad {
        &mut self.data.0
    }
}
