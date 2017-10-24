//! Handler for keyboards

use events::key_events::KeyEvent;
use libc;
use types::input_device::InputDevice;

use types::keyboard::Keyboard;
use wlroots_sys::wlr_event_keyboard_key;

pub trait KeyboardHandler {
    /// Callback that is triggered when a key is pressed.
    fn on_key(&mut self, &mut Keyboard, &KeyEvent) {}
}

wayland_listener!(KeyboardWrapper, (InputDevice, Box<KeyboardHandler>), [
    key_listener => key_notify: |this: &mut KeyboardWrapper, data: *mut libc::c_void,| unsafe {
        let (ref input_device, ref mut keyboard_handler) = this.data;
        let key = KeyEvent::from_ptr(data as *mut wlr_event_keyboard_key);

        keyboard_handler.on_key(&mut Keyboard::new(input_device.dev_union().keyboard), &key)
    };
]);
