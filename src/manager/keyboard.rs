//! Handler for keyboards

use events::key_events::KeyEvent;
use libc;
use types::input_device::InputDevice;
use types::keyboard::KeyboardHandle;
use wlroots_sys::wlr_event_keyboard_key;

pub trait KeyboardHandler {
    /// Callback that is triggered when a key is pressed.
    fn on_key(&mut self, &mut KeyEvent) {}
}

wayland_listener!(Keyboard, (InputDevice, Box<KeyboardHandler>), [
    key_listener => key_notify: |this: &mut Keyboard, data: *mut libc::c_void,| unsafe {
        let (ref input_device, ref mut keyboard_handler) = this.data;
        let mut key = KeyEvent::new(data as *mut wlr_event_keyboard_key,
                                    KeyboardHandle::new(input_device.dev_union().keyboard));

        keyboard_handler.on_key(&mut key)
    };
]);

impl Keyboard {
    pub fn input_device(&self) -> &InputDevice {
        &self.data.0
    }
}
