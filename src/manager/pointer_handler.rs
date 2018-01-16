//! Handler for pointers

use libc;

use {InputDevice, Pointer};
use compositor::{Compositor, COMPOSITOR_PTR};
use events::pointer_events::{AbsoluteMotionEvent, AxisEvent, ButtonEvent, MotionEvent};

use wlroots_sys::{wlr_event_pointer_axis, wlr_event_pointer_button, wlr_event_pointer_motion};

pub trait PointerHandler {
    /// Callback that is triggered when the pointer moves.
    fn on_motion(&mut self, &mut Compositor, &mut Pointer, &MotionEvent) {}

    fn on_motion_absolute(&mut self, &mut Compositor, &mut Pointer, &AbsoluteMotionEvent) {}

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_button(&mut self, &mut Compositor, &mut Pointer, &ButtonEvent) {}

    fn on_axis(&mut self, &mut Compositor, &mut Pointer, &AxisEvent) {}
}

wayland_listener!(PointerWrapper, (Pointer, Box<PointerHandler>), [
    button_listener => key_notify: |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = ButtonEvent::from_ptr(data as *mut wlr_event_pointer_button);
        let compositor = &mut *COMPOSITOR_PTR;
        pointer.set_lock(true);
        this.data.1.on_button(compositor, pointer, &event);
        pointer.set_lock(false);
    };
    motion_listener => motion_notify:  |this: &mut PointerWrapper, data: *mut libc::c_void,|
    unsafe {
        let pointer = &mut this.data.0;
        let event = MotionEvent::from_ptr(data as *mut wlr_event_pointer_motion);
        let compositor = &mut *COMPOSITOR_PTR;
        pointer.set_lock(true);
        this.data.1.on_motion(compositor, pointer, &event);
        pointer.set_lock(false);
    };
    motion_absolute_listener => motion_absolute_notify:
    |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = AbsoluteMotionEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;
        pointer.set_lock(true);
        this.data.1.on_motion_absolute(compositor, pointer, &event);
        pointer.set_lock(false);
    };
    axis_listener => axis_notify:  |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = AxisEvent::from_ptr(data as *mut wlr_event_pointer_axis);
        let compositor = &mut *COMPOSITOR_PTR;
        pointer.set_lock(true);
        this.data.1.on_axis(compositor, pointer, &event);
        pointer.set_lock(false);
    };
]);

impl PointerWrapper {
    pub(crate) unsafe fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }
}
