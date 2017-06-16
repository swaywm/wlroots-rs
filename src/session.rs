//! The session that the compositor controls. Each compositor has one session.

use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_event_loop,
                          wl_event_loop_timer_func_t};
use wlroots_sys::{self, wl_display, wlr_session, wlr_backend,
                  wlr_session_start, wlr_backend_autocreate, wlr_backend_init};
use std::os::raw::{c_void, c_int};
use ::Backend;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SessionErr {
    SessionFailed,
    DisplayFailed,
    EventLoopFailed,
    BackendFailed
}

/// A session that controls the wl_display, it's event loop, and the
/// [wlr_session](../../wlroots_sys/struct.wlr_session.html)
pub struct Session {
    /// The backend that this session uses to display content.
    pub backend: Backend,
    /// The session for the compositor. Usually this is via logind.
    pub session: *mut wlr_session,
    /// The pointer to the wayland display proxy object.
    pub display: *mut wl_display,
    /// The pointer to the wayland event loop.
    pub event_loop: *mut wl_event_loop,
}

impl Session {
    /// Creates a new Wayland session.
    ///
    /// Automatically creates the `wl_display` and `wl_event_loop` objects
    /// using the standard Wayland library.
    pub fn new() -> Result<Self, SessionErr> {
        unsafe {
            let display = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                        wl_display_create,) as *mut _;
            if display.is_null() {
                return Err(SessionErr::DisplayFailed)
            }
            let event_loop = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                           wl_display_get_event_loop,
                                           display as *mut _);
            if event_loop.is_null() {
                return Err(SessionErr::EventLoopFailed)
            }
            let session = wlr_session_start(display);
            if session.is_null() {
                return Err(SessionErr::SessionFailed)
            }
            let backend = Backend::autocreate_backend(display, session)?;
            Ok(Session {backend, session, display, event_loop})
        }
    }

    /// Dispatches queued events and fetches any new signals/events/requests.
    ///
    /// Returns whatever numerical code `wl_event_loop_dispatch` returns.
    pub fn dispatch_event_loop(&mut self) -> i32 {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                        wl_event_loop_dispatch,
                        self.event_loop,
                        0)
        }
    }


    pub fn set_timeout<T: Send + Sync, F>(&mut self,
                                    data: &mut T,
                                    callback: F,
                                    delay: u32) where F: Fn(&mut T) {
        let boxed_func = Box::new(callback);
        let callback_ptr = Box::into_raw(boxed_func);
        let actual_data = Box::new(TimeoutData {
            callback_ptr,
            data
        });
        let actual_data_ptr = Box::into_raw(actual_data);
        unsafe {
            let timer = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                    wl_event_loop_add_timer,
                                    self.event_loop,
                                    timer_done::<T, F>,
                                    actual_data_ptr as *mut _);
            if timer.is_null() {
                panic!("Timer was null");
            }
            let res = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                    wl_event_source_timer_update,
                                    timer,
                                    delay as i32);
            // TODO Make sure handling this right, return an Err
            if res != 0 {
                panic!("wl_event_loop_add_timer returned an non-zero value!")
            }
        }
    }
}

#[repr(C)]
pub struct TimeoutData<T, F> where F: Fn(&mut T) {
    callback_ptr: *mut F,
    data: *mut T
}

unsafe extern "C" fn timer_done<T, F>(data: *mut c_void) -> c_int
    where F: Fn(&mut T) {
    let data = data as *mut TimeoutData<T, F>;
    let callback = Box::from_raw((*data).callback_ptr);
    let mut real_data = Box::from_raw((*data).data);
    (*callback)(&mut *real_data);
    1
}
