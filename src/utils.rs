//! Utility functions that patch holes in wayland-rs.

use std::ptr;

use wlroots_sys::{wl_list, wl_signal};
use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_notify_func_t};

#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}


#[macro_export]
macro_rules! wl_container_of {
    ($ptr:ident, $ty:ty, $field:ident) => {
        ($ptr as usize - offset_of!($ty, $field)) as *const $ty
    }
}

/// Real definition of wl_listener, as defined in the Wayland server headers
#[repr(C)]
pub struct wl_listener {
    link: wl_list,
    notify: wl_notify_func_t
}

impl wl_listener {
    pub fn new(notify: wl_notify_func_t) -> Self {
        wl_listener {
            link: wl_list {
                prev: ptr::null_mut(),
                next: ptr::null_mut()
            },
            notify
        }
    }
}

/// This is what wl_signal_add is suppose to be but it's not defined in
/// wayland-rs for some reason, so we define it here.
pub unsafe fn wl_signal_add(signal: &mut wl_signal,
                        listener: &mut self::wl_listener) {
    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                  wl_list_insert,
                  signal.listener_list.prev as *mut _,
                  &mut listener.link as *mut _ as *mut _)
}
