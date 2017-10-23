//! Handler for pointers

use device::Device;
use libc;
use pointer;
use wlroots_sys::{wlr_event_pointer_axis, wlr_event_pointer_button, wlr_event_pointer_motion,
                  wlr_event_pointer_motion_absolute};

pub trait PointerHandler {
    /// Callback that is triggered when the pointer moves.
    fn on_motion(&mut self, &mut Device, &pointer::MotionEvent) {
        // TODO
    }

    fn on_motion_absolute(&mut self, &mut Device, &pointer::AbsoluteMotionEvent) {
        // TODO
    }

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_button(&mut self, &mut Device, &pointer::ButtonEvent) {
        // TODO
    }

    fn on_axis(&mut self, &mut Device, &pointer::AxisEvent) {
        // TODO
    }
}

wayland_listener!(Pointer, (Device, Box<PointerHandler>), [
    button_listener => key_notify: |this: &mut Pointer, data: *mut libc::c_void,| unsafe {
        let event = pointer::ButtonEvent::from_ptr(data as *mut wlr_event_pointer_button);
        this.data.1.on_button(&mut this.data.0, &event)
    };
    motion_listener => motion_notify:  |this: &mut Pointer, data: *mut libc::c_void,| unsafe {
        let event = pointer::MotionEvent::from_ptr(data as *mut wlr_event_pointer_motion);
        this.data.1.on_motion(&mut this.data.0, &event)
    };
    motion_absolute_listener => motion_absolute_notify:  |this: &mut Pointer, data: *mut libc::c_void,| unsafe {
        let event = pointer::AbsoluteMotionEvent::from_ptr(data as *mut wlr_event_pointer_motion_absolute);
        this.data.1.on_motion_absolute(&mut this.data.0, &event)
    };
    axis_listener => axis_notify:  |this: &mut Pointer, data: *mut libc::c_void,| unsafe {
        let event = pointer::AxisEvent::from_ptr(data as *mut wlr_event_pointer_axis);
        this.data.1.on_axis(&mut this.data.0, &event)
    };
]);
