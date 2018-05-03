//! Handler for tablet pads

use {InputDevice, TabletPad, TabletPadHandle};
use compositor::{compositor_handle, CompositorHandle};
use events::tablet_pad_events::{ButtonEvent, RingEvent, StripEvent};
use libc;

pub trait TabletPadHandler {
    /// Callback that is triggered when a button is pressed on the tablet pad.
    fn on_button(&mut self, CompositorHandle, TabletPadHandle, &ButtonEvent) {}

    /// Callback that is triggered when the touch strip is used.
    fn on_strip(&mut self, CompositorHandle, TabletPadHandle, &StripEvent) {}

    /// Callback that is triggered when the ring is touched.
    fn on_ring(&mut self, CompositorHandle, TabletPadHandle, &RingEvent) {}
}

wayland_listener!(TabletPadWrapper, (TabletPad, Box<TabletPadHandler>), [
    button_listener => button_notify: |this: &mut TabletPadWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref pad, ref mut handler) = this.data;
        let event = ButtonEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_button(compositor,
                          pad.weak_reference(),
                          &event);
    };
    strip_listener => strip_notify: |this: &mut TabletPadWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref pad, ref mut handler) = this.data;
        let event = StripEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_strip(compositor,
                         pad.weak_reference(),
                         &event);
    };
    ring_listener => ring_notify: |this: &mut TabletPadWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref pad, ref mut handler) = this.data;
        let event = RingEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_ring(compositor,
                        pad.weak_reference(),
                        &event);
    };
]);

impl TabletPadWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub(crate) fn tablet_pad(&mut self) -> TabletPadHandle {
        self.data.0.weak_reference()
    }
}
