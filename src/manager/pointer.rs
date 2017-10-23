//! Handler for pointers

use device::Device;
use libc;
use pointer;
use wlroots_sys::wlr_event_pointer_button;

pub trait PointerHandler {
    /// Callback that is triggered when the pointer moves.
    fn on_motion(&mut self, &mut Device) {
        // TODO
    }

    fn on_motion_absolute(&mut self, &mut Device) {
        // TODO
    }

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_button(&mut self, &mut Device, &pointer::ButtonEvent) {
        // TODO
    }

    fn on_axis(&mut self, &mut Device) {
        // TODO
    }
}

wayland_listener!(Pointer, (Device, Box<PointerHandler>), [
    button_listener => key_notify: |this: &mut Pointer, data: *mut libc::c_void,| unsafe {
        let event = pointer::ButtonEvent::from_ptr(data as *mut wlr_event_pointer_button);
        this.data.1.on_button(&mut this.data.0, &event)
    };
    motion_listener => motion_notify:  |this: &mut Pointer, _data: *mut libc::c_void,| unsafe {
        this.data.1.on_motion(&mut this.data.0)
    };
    motion_absolute_listener => motion_absolute_notify:  |this: &mut Pointer, _data: *mut libc::c_void,| unsafe {
        this.data.1.on_motion_absolute(&mut this.data.0)
    };
    axis_listener => axis_notify:  |this: &mut Pointer, _data: *mut libc::c_void,| unsafe {
        this.data.1.on_axis(&mut this.data.0)
    };
]);
