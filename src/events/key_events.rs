use std::time::Duration;

use wlroots_sys::{wlr_event_keyboard_key, xkb_keysym_t, xkb_state, xkb_state_key_get_syms};

pub type Key = xkb_keysym_t;

use types::keyboard::KeyState;

#[derive(Debug)]
pub struct KeyEvent {
    key: *mut wlr_event_keyboard_key,
    xkb_state: *mut xkb_state
}

impl KeyEvent {
    /// Constructs a KeyEvent from the raw key event pointer information.
    pub(crate) unsafe fn new(key: *mut wlr_event_keyboard_key, xkb_state: *mut xkb_state) -> Self {
        KeyEvent { key, xkb_state }
    }

    /// Gets the raw keycode from the device.
    ///
    /// Usually you want to use `KeyEvent::input_keys` since you care about what
    /// value XKB says this is.
    pub fn keycode(&self) -> u32 {
        unsafe { (*self.key).keycode + 8 }
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
    pub fn key_state(&self) -> KeyState {
        unsafe { (*self.key).state }
    }

    /// Gets the keys that are pressed using XKB to convert them to a more
    /// programmer friendly form.
    pub fn input_keys(&self) -> Vec<Key> {
        unsafe {
            let mut syms = 0 as *const xkb_keysym_t;
            let key_length = xkb_state_key_get_syms(self.xkb_state, self.keycode(), &mut syms);
            (0..key_length).map(|index| *syms.offset(index as isize))
                           .collect()
        }
    }
}
