//! Handler for keyboards

use libc;

use {InputDevice, Keyboard, KeyboardHandle};
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
}

wayland_listener!(KeyboardWrapper, (Keyboard, Box<KeyboardHandler>), [
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

impl KeyboardWrapper {
    pub(crate) fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }

    pub(crate) fn keyboard(&self) -> KeyboardHandle {
        self.data.0.weak_reference()
    }
}
