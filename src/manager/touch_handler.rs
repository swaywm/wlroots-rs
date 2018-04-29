//! Handler for touch input

use libc;

use compositor::{Compositor, COMPOSITOR_PTR};
use events::touch_events::{CancelEvent, DownEvent, MotionEvent, UpEvent};
use types::input::{InputDevice, Touch};

pub trait TouchHandler {
    /// Callback that is triggered when the user starts touching the
    /// screen/input device.
    fn on_down(&mut self, &mut Compositor, &mut Touch, &DownEvent) {}

    /// Callback that is triggered when the user stops touching the
    /// screen/input device.
    fn on_up(&mut self, &mut Compositor, &mut Touch, &UpEvent) {}

    /// Callback that is triggered when the user moves his fingers along the
    /// screen/input device.
    fn on_motion(&mut self, &mut Compositor, &mut Touch, &MotionEvent) {}

    /// Callback triggered when the touch is canceled.
    fn on_cancel(&mut self, &mut Compositor, &mut Touch, &CancelEvent) {}
}

wayland_listener!(TouchWrapper, (Touch, Box<TouchHandler>), [
    down_listener => down_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut touch, ref mut handler) = this.data;
        let event = DownEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        touch.set_lock(true);
        handler.on_down(compositor, touch, &event);
        touch.set_lock(false);
        compositor.lock.set(false);
    };
    up_listener => up_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut touch, ref mut handler) = this.data;
        let event = UpEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        touch.set_lock(true);
        handler.on_up(compositor, touch, &event);
        touch.set_lock(false);
        compositor.lock.set(false);
    };
    motion_listener => motion_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut touch, ref mut handler) = this.data;
        let event = MotionEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        touch.set_lock(true);
        handler.on_motion(compositor, touch, &event);
        touch.set_lock(false);
        compositor.lock.set(false);
    };
    cancel_listener => cancel_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut touch, ref mut handler) = this.data;
        let event = CancelEvent::from_ptr(data as *mut _);
        let compositor = &mut *COMPOSITOR_PTR;

        compositor.lock.set(true);
        touch.set_lock(true);
        handler.on_cancel(compositor, touch, &event);
        touch.set_lock(false);
        compositor.lock.set(false);
    };
]);

impl TouchWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub(crate) fn touch(&mut self) -> &mut Touch {
        &mut self.data.0
    }
}
