use wlroots_sys::{wlr_event_keyboard_key, xkb_keysym_t, xkb_state, xkb_state_key_get_syms};

pub type Key = xkb_keysym_t;

#[derive(Debug)]
pub struct KeyEvent {
    key: *mut wlr_event_keyboard_key,
    xkb_state: *mut xkb_state
}

impl KeyEvent {
    pub(crate) unsafe fn new(key: *mut wlr_event_keyboard_key, xkb_state: *mut xkb_state) -> Self {
        KeyEvent { key, xkb_state }
    }

    pub fn keycode(&self) -> u32 {
        unsafe { (*self.key).keycode + 8 }
    }

    pub fn input_keys(&self) -> Vec<Key> {
        unsafe {
            let mut syms = 0 as *const xkb_keysym_t;
            let key_length = xkb_state_key_get_syms(self.xkb_state, self.keycode(), &mut syms);
            (0..key_length)
                .map(|index| *syms.offset(index as isize))
                .collect()
        }
    }
}
