//! Handler for touch input

use libc;

use compositor::{compositor_handle, CompositorHandle};
use events::touch_events::{CancelEvent, DownEvent, MotionEvent, UpEvent};
use types::input::{InputDevice, Touch, TouchHandle};

pub trait TouchHandler {
    /// Callback that is triggered when the user starts touching the
    /// screen/input device.
    fn on_down(&mut self, CompositorHandle, TouchHandle, &DownEvent) {}

    /// Callback that is triggered when the user stops touching the
    /// screen/input device.
    fn on_up(&mut self, CompositorHandle, TouchHandle, &UpEvent) {}

    /// Callback that is triggered when the user moves his fingers along the
    /// screen/input device.
    fn on_motion(&mut self, CompositorHandle, TouchHandle, &MotionEvent) {}

    /// Callback triggered when the touch is canceled.
    fn on_cancel(&mut self, CompositorHandle, TouchHandle, &CancelEvent) {}
}

wayland_listener!(TouchWrapper, (Touch, Box<TouchHandler>), [
    down_listener => down_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = DownEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_down(compositor,
                        touch.weak_reference(),
                        &event);
    };
    up_listener => up_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = UpEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_up(compositor,
                      touch.weak_reference(),
                      &event);
    };
    motion_listener => motion_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = MotionEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_motion(compositor,
                          touch.weak_reference(),
                          &event);
    };
    cancel_listener => cancel_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = CancelEvent::from_ptr(data as *mut _);
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_cancel(compositor,
                          touch.weak_reference(),
                          &event);
    };
]);

impl TouchWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub(crate) fn touch(&mut self) -> TouchHandle {
        self.data.0.weak_reference()
    }
}
