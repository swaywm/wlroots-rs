//! Handler for keyboards

use device::Device;
use key_event::KeyEvent;
use libc;
use wlroots_sys::wlr_event_keyboard_key;

pub trait KeyboardHandler {
    /// Callback that is triggered when a key is pressed.
    fn on_key(&mut self, &mut Device, &KeyEvent) {}
}

wayland_listener!(Keyboard, (Device, Box<KeyboardHandler>), [
    key_listener => key_notify: |this: &mut Keyboard, data: *mut libc::c_void,| unsafe {
        let key = KeyEvent::from_ptr(data as *mut wlr_event_keyboard_key);
        this.data.1.on_key(&mut this.data.0, &key)
    };
]);
