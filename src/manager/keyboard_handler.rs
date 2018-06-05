//! Handler for keyboards

use libc;
use wlroots_sys::wlr_input_device;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;

use {Keyboard, KeyboardHandle};
use compositor::{compositor_handle, CompositorHandle};
use events::key_events::KeyEvent;

use wlroots_sys::wlr_event_keyboard_key;

pub trait KeyboardHandler {
    /// Callback that is triggered when a key is pressed.
    fn on_key(&mut self, CompositorHandle, KeyboardHandle, &KeyEvent) {}

    /// Callback that is triggered when modifiers are pressed.
    fn modifiers(&mut self, CompositorHandle, KeyboardHandle) {}

    /// Callback that is triggered when the keymap is updated.
    fn keymap(&mut self, CompositorHandle, KeyboardHandle) {}

    /// Callback that is triggered when repeat info is updated.
    fn repeat_info(&mut self, CompositorHandle, KeyboardHandle) {}

    /// Callback that is triggered when the keyboard is destroyed.
    fn destroyed(&mut self, CompositorHandle, KeyboardHandle) {}
}

wayland_listener!(KeyboardWrapper, (Keyboard, Box<KeyboardHandler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut KeyboardWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut keyboard, ref mut keyboard_handler) = this.data;
            let compositor = match compositor_handle() {
                Some(handle) => handle,
                None => return
            };
            keyboard_handler.destroyed(compositor, keyboard.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.key_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.modifiers_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.keymap_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.repeat_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut KeyboardWrapper);
    };
    key_listener => key_notify: |this: &mut KeyboardWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let xkb_state = (*keyboard.as_ptr()).xkb_state;
        let key = KeyEvent::new(data as *mut wlr_event_keyboard_key, xkb_state);

        keyboard_handler.on_key(compositor, keyboard.weak_reference(), &key);
    };
    modifiers_listener => modifiers_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        keyboard_handler.modifiers(compositor, keyboard.weak_reference());
    };
    keymap_listener => keymap_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        keyboard_handler.keymap(compositor, keyboard.weak_reference());
    };
   repeat_listener => repeat_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        keyboard_handler.repeat_info(compositor, keyboard.weak_reference());
    };
]);
