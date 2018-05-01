//! Handler for pointers

use libc;

use {InputDevice, Pointer, PointerHandle};
use compositor::{compositor_handle, CompositorHandle};
use events::pointer_events::{AbsoluteMotionEvent, AxisEvent, ButtonEvent, MotionEvent};

use wlroots_sys::{wlr_event_pointer_axis, wlr_event_pointer_button, wlr_event_pointer_motion};

pub trait PointerHandler {
    /// Callback that is triggered when the pointer moves.
    fn on_motion(&mut self, CompositorHandle, PointerHandle, &MotionEvent) {}

    fn on_motion_absolute(&mut self, CompositorHandle, PointerHandle, &AbsoluteMotionEvent) {}

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_button(&mut self, CompositorHandle, PointerHandle, &ButtonEvent) {}

    /// Callback that is triggerde when an axis event fires
    fn on_axis(&mut self, CompositorHandle, PointerHandle, &AxisEvent) {}
}

wayland_listener!(PointerWrapper, (Pointer, Box<PointerHandler>), [
    button_listener => key_notify: |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = ButtonEvent::from_ptr(data as *mut wlr_event_pointer_button);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_button(compositor, pointer.weak_reference(), &event);
    };
    motion_listener => motion_notify:  |this: &mut PointerWrapper, data: *mut libc::c_void,|
    unsafe {
        let pointer = &mut this.data.0;
        let event = MotionEvent::from_ptr(data as *mut wlr_event_pointer_motion);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_motion(compositor, pointer.weak_reference(), &event);
    };
    motion_absolute_listener => motion_absolute_notify:
    |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = AbsoluteMotionEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_motion_absolute(compositor, pointer.weak_reference(), &event);
    };
    axis_listener => axis_notify:  |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = AxisEvent::from_ptr(data as *mut wlr_event_pointer_axis);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_axis(compositor, pointer.weak_reference(), &event);
    };
]);

impl PointerWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub(crate) fn pointer(&mut self) -> PointerHandle {
        self.data.0.weak_reference()
    }
}
