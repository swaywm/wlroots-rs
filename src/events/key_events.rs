use types::input_device::InputDevice;

use wlroots_sys::{wlr_event_keyboard_key, xkb_keysym_t, xkb_state_key_get_syms};

#[derive(Debug)]
pub struct KeyEvent {
    key: *mut wlr_event_keyboard_key,
}

impl KeyEvent {
    pub unsafe fn from_ptr(key: *mut wlr_event_keyboard_key) -> Self {
        KeyEvent { key: key }
    }

    pub fn keycode(&self) -> u32 {
        unsafe { (*self.key).keycode + 8 }
    }

    pub unsafe fn get_input_keys(&self, dev: &InputDevice) -> Vec<xkb_keysym_t> {
        let mut syms = 0 as *const xkb_keysym_t;
        unsafe {
            // TODO check union (or better yet, wrap it!)
            let key_length = xkb_state_key_get_syms((*dev.dev_union().keyboard).xkb_state,
                                                    self.keycode(),
                                                    &mut syms);
            (0..key_length)
                .map(|index| *syms.offset(index as isize))
                .collect()
        }
    }
}
