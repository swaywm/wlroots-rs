use types::device::Device;

use wlroots_sys::{wlr_event_keyboard_key, xkb_keysym_t, xkb_state_key_get_syms};

#[derive(Debug)]
pub struct KeyEvent {
    key: *mut wlr_event_keyboard_key
}

impl KeyEvent {
    pub unsafe fn from_ptr(key: *mut wlr_event_keyboard_key) -> Self {
        KeyEvent { key }
    }

    pub fn keycode(&self) -> u32 {
        unsafe { (*self.key).keycode + 8 }
    }

    // TODO should probably go somewhere else..like a keyboard struct or something
    // Unsafe until we can do that, because it uses a union internally and we don't
    // check it to be sure we got the right thing.
    //
    // Make a sexy Keyboard struct that holds the device, pass that to the on_key
    // callback
    // instead
    pub unsafe fn get_input_keys(&self, dev: &Device) -> Vec<xkb_keysym_t> {
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
