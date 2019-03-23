use std::time::Duration;

use wlroots_sys::{wlr_event_keyboard_key, wlr_key_state, xkb_keysym_t, xkb_state, xkb_state_key_get_syms};

use input::keyboard;

#[derive(Debug)]
pub struct Key {
    key: *mut wlr_event_keyboard_key,
    xkb_state: *mut xkb_state
}

impl Key {
    /// Constructs a Key from the raw key event pointer information.
    pub(crate) unsafe fn new(key: *mut wlr_event_keyboard_key, xkb_state: *mut xkb_state) -> Self {
        Key { key, xkb_state }
    }

    /// Gets the raw keycode from the device.
    ///
    /// Usually you want to use `Key::input_keys` since you care about what
    /// value XKB says this is.
    pub fn keycode(&self) -> u32 {
        unsafe { (*self.key).keycode }
    }

    /// Get how long the key has been pressed down, in milliseconds.
    pub fn time_msec(&self) -> Duration {
        Duration::from_millis(unsafe { (*self.key).time_msec } as u64)
    }

    /// TODO What is this?
    pub fn update_state(&self) -> bool {
        unsafe { (*self.key).update_state }
    }

    /// Get the pressed/released state of the key.
    pub fn key_state(&self) -> wlr_key_state {
        unsafe { (*self.key).state }
    }

    /// Gets the keys that are pressed using XKB to convert them to a more
    /// programmer friendly form.
    pub fn pressed_keys(&self) -> Vec<keyboard::Key> {
        unsafe {
            let mut syms = 0 as *const xkb_keysym_t;
            let key_length = xkb_state_key_get_syms(self.xkb_state, self.keycode() + 8, &mut syms);
            (0..key_length)
                .map(|index| *syms.offset(index as isize))
                .collect()
        }
    }
}
