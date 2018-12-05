//! Handler for touch input

use libc;
use wlroots_sys::wlr_input_device;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;

use {compositor::{compositor_handle, CompositorHandle},
     events::touch_events::{CancelEvent, DownEvent, MotionEvent, UpEvent},
     input::touch::{Touch, TouchHandle}};

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

    /// Callback that is triggered when the touch is destroyed.
    fn destroyed(&mut self, CompositorHandle, TouchHandle) {}
}

wayland_listener!(pub(crate) TouchWrapper, (Touch, Box<TouchHandler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut touch, ref mut touch_handler) = this.data;
            let compositor = match compositor_handle() {
                Some(handle) => handle,
                None => return
            };
            touch_handler.destroyed(compositor, touch.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.down_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.up_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.motion_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.cancel_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut TouchWrapper);
    };
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
