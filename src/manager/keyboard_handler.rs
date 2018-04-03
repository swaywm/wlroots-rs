//! Handler for keyboards

use libc;

use {InputDevice, Keyboard};
use compositor::{Compositor, COMPOSITOR_PTR};
use events::key_events::KeyEvent;

use wlroots_sys::wlr_event_keyboard_key;

pub trait KeyboardHandler {
    /// Callback that is triggered when a key is pressed.
    fn on_key(&mut self, &mut Compositor, &mut Keyboard, &mut KeyEvent) {}

    /// Callback that is triggered when modifiers are pressed.
    fn modifiers(&mut self, &mut Compositor, &mut Keyboard) {}

    /// Callback that is triggered when the keymap is updated.
    fn keymap(&mut self, &mut Compositor, &mut Keyboard) {}

    /// Callback that is triggered when repeat info is updated.
    fn repeat_info(&mut self, &mut Compositor, &mut Keyboard) {}
}

wayland_listener!(KeyboardWrapper, (Keyboard, Box<KeyboardHandler>), [
    key_listener => key_notify: |this: &mut KeyboardWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        let xkb_state = (*keyboard.as_ptr()).xkb_state;
        let mut key = KeyEvent::new(data as *mut wlr_event_keyboard_key, xkb_state);

        keyboard.set_lock(true);
        keyboard_handler.on_key(compositor, keyboard, &mut key);
        keyboard.set_lock(false);
    };
    modifiers_listener => modifiers_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;

        keyboard.set_lock(true);
        keyboard_handler.modifiers(compositor, keyboard);
        keyboard.set_lock(false);
    };
    keymap_listener => keymap_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;

        keyboard.set_lock(true);
        keyboard_handler.keymap(compositor, keyboard);
        keyboard.set_lock(false);
    };
   repeat_listener => repeat_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;

        keyboard.set_lock(true);
        keyboard_handler.repeat_info(compositor, keyboard);
        keyboard.set_lock(false);
    };
]);

impl KeyboardWrapper {
    pub fn input_device(&self) -> &InputDevice {
        self.data.0.input_device()
    }
}
