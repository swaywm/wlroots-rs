//! TODO Documentation
use std::{cell::Cell, fmt, ptr::NonNull, rc::Rc};

pub use wlroots_sys::wlr_key_state;
use wlroots_sys::{
    wlr_input_device, wlr_keyboard, wlr_keyboard_get_modifiers, wlr_keyboard_led, wlr_keyboard_led_update,
    wlr_keyboard_modifier, wlr_keyboard_modifiers, wlr_keyboard_set_keymap, xkb_keysym_t
};
use xkbcommon::xkb::ffi::{xkb_keymap, xkb_state};
use xkbcommon::xkb::{self, Keycode, Keymap, LedIndex, ModIndex};

pub use events::key_events as event;
pub use manager::keyboard_handler::*;
use {
    input::{self, InputState},
    utils::{self, HandleErr, HandleResult, Handleable}
};

pub type Key = xkb_keysym_t;
pub type Handle = utils::Handle<NonNull<wlr_input_device>, wlr_keyboard, Keyboard>;

/// Information about repeated keypresses for a particular Keyboard.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RepeatInfo {
    /// The rate at which extended keypresses will fire more events.
    pub rate: i32,
    /// How long it takes for a keypress to register on this device.
    pub delay: i32
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Modifiers {
    pub depressed: Modifier,
    pub latched: Modifier,
    pub locked: Modifier,
    pub group: Modifier
}

impl Default for Modifiers {
    fn default() -> Self {
        Modifiers {
            depressed: Modifier::empty(),
            latched: Modifier::empty(),
            locked: Modifier::empty(),
            group: Modifier::empty()
        }
    }
}

impl From<wlr_keyboard_modifiers> for Modifiers {
    fn from(mods: wlr_keyboard_modifiers) -> Self {
        Modifiers {
            depressed: Modifier::from_bits_truncate(mods.depressed),
            latched: Modifier::from_bits_truncate(mods.latched),
            locked: Modifier::from_bits_truncate(mods.locked),
            group: Modifier::from_bits_truncate(mods.group)
        }
    }
}
impl Into<wlr_keyboard_modifiers> for Modifiers {
    fn into(self) -> wlr_keyboard_modifiers {
        wlr_keyboard_modifiers {
            depressed: self.depressed.bits(),
            latched: self.latched.bits(),
            locked: self.locked.bits(),
            group: self.group.bits()
        }
    }
}

#[derive(Debug)]
pub struct Keyboard {
    /// The structure that ensures weak handles to this structure are still
    /// alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `keyboard::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `keyboard::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this keyboard.
    device: input::Device,
    /// The underlying keyboard data.
    keyboard: NonNull<wlr_keyboard>
}

impl Keyboard {
    /// Tries to convert an input device to a Keyboard
    ///
    /// Returns None if it is of a different type of input variant.
    ///
    /// # Safety
    /// This creates a totally new Keyboard (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_KEYBOARD => {
                let keyboard = NonNull::new((*device).__bindgen_anon_1.keyboard).expect(
                    "Keyboard pointer \
                     was null"
                );
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState {
                    handle,
                    device: input::Device::from_ptr(device)
                });
                (*keyboard.as_ptr()).data = Box::into_raw(state) as *mut _;
                Some(Keyboard {
                    liveliness,
                    device: input::Device::from_ptr(device),
                    keyboard
                })
            },
            _ => None
        }
    }

    /// Gets the wlr_keyboard associated with this keyboard::Handle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_keyboard {
        self.keyboard.as_ptr()
    }

    /// Gets the wlr_input_device associated with this keyboard::Handle
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }

    /// Set the keymap for this Keyboard.
    pub fn set_keymap(&mut self, keymap: &Keymap) {
        unsafe {
            // NOTE wlr_keyboard_set_keymap updates the reference count,
            // so we don't need to mem::forget the key map here
            // or take it by value.
            wlr_keyboard_set_keymap(self.keyboard.as_ptr(), keymap.get_raw_ptr() as _);
        }
    }

    /// Get the XKB keymap associated with this Keyboard.
    pub fn get_keymap(&mut self) -> Option<Keymap> {
        unsafe {
            let keymap_ptr = (*self.keyboard.as_ptr()).keymap as *mut xkb_keymap;
            if keymap_ptr.is_null() {
                None
            } else {
                Some(Keymap::from_raw_ptr(keymap_ptr))
            }
        }
    }

    /// Get the keycodes for this keyboard as reported by XKB.
    ///
    /// # Limitations
    /// wlroots limits this list to `WLR_KEYBOARD_KEYS_CAP` elements,
    /// which at the time of writing is `32`.
    pub fn keycodes(&self) -> Vec<Keycode> {
        unsafe {
            let mut result = (*self.keyboard.as_ptr()).keycodes.to_vec();
            result.truncate((*self.keyboard.as_ptr()).num_keycodes);
            result
        }
    }

    /// Get the list of LEDs for this keyboard as reported by XKB.
    pub fn led_list(&self) -> &[LedIndex] {
        unsafe { &(*self.keyboard.as_ptr()).led_indexes }
    }

    /// Get the list of modifiers for this keyboard as reported by XKB.
    pub fn modifier_list(&self) -> &[ModIndex] {
        unsafe { &(*self.keyboard.as_ptr()).mod_indexes }
    }

    /// Get the size of the keymap.
    pub fn keymap_size(&self) -> usize {
        unsafe { (*self.keyboard.as_ptr()).keymap_size }
    }

    /// Get the XKB state associated with this `Keyboard`.
    pub fn get_xkb_state(&mut self) -> Option<xkb::State> {
        unsafe {
            let xkb_state_ptr = (*self.keyboard.as_ptr()).xkb_state as *mut xkb_state;
            if xkb_state_ptr.is_null() {
                None
            } else {
                Some(xkb::State::from_raw_ptr(xkb_state_ptr))
            }
        }
    }

    /// Get the repeat info for this keyboard.
    pub fn repeat_info(&self) -> RepeatInfo {
        unsafe {
            RepeatInfo {
                rate: (*self.keyboard.as_ptr()).repeat_info.rate,
                delay: (*self.keyboard.as_ptr()).repeat_info.delay
            }
        }
    }

    /// Update the LED lights using the provided bitmap.
    ///
    /// 1 means one, 0 means off.
    pub fn update_led(&mut self, leds: Led) {
        unsafe {
            wlr_keyboard_led_update(self.keyboard.as_ptr(), leds.bits() as u32);
        }
    }

    /// Get the modifiers that are currently pressed on the keyboard.
    pub fn get_modifiers(&self) -> Modifier {
        unsafe { Modifier::from_bits_truncate(wlr_keyboard_get_modifiers(self.keyboard.as_ptr())) }
    }

    /// Get the modifier masks for each group.
    pub fn get_modifier_masks(&self) -> Modifiers {
        From::from(unsafe { (*self.keyboard.as_ptr()).modifiers })
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped Keyboard {:p}", self.keyboard.as_ptr());
            unsafe {
                let _ = Box::from_raw((*self.keyboard.as_ptr()).data as *mut InputState);
            }
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(
                    WLR_DEBUG,
                    "Still {} weak pointers to Keyboard {:p}",
                    weak_count,
                    self.keyboard.as_ptr()
                );
            }
        }
    }
}

impl Handleable<NonNull<wlr_input_device>, wlr_keyboard> for Keyboard {
    #[doc(hidden)]
    unsafe fn from_ptr(keyboard: *mut wlr_keyboard) -> Option<Self> {
        let keyboard = NonNull::new(keyboard)?;
        let data = Box::from_raw((*keyboard.as_ptr()).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*keyboard.as_ptr()).data = Box::into_raw(data) as *mut _;
        Some(Keyboard {
            liveliness: handle.upgrade().unwrap(),
            device,
            keyboard
        })
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_keyboard {
        self.keyboard.as_ptr()
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle.upgrade().ok_or(HandleErr::AlreadyDropped)?;
        let device = handle.data.ok_or(HandleErr::AlreadyDropped)?;
        Ok(Keyboard {
            liveliness,
            // NOTE Rationale for cloning:
            // If we already dropped we don't reach this point.
            device: input::Device { device },
            keyboard: handle.as_non_null()
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle {
            ptr: self.keyboard,
            handle: Rc::downgrade(&self.liveliness),
            // NOTE Rationale for cloning:
            // Since we have a strong reference already,
            // the input must still be alive.
            data: unsafe { Some(self.device.as_non_null()) },
            _marker: std::marker::PhantomData
        }
    }
}

bitflags! {
    pub struct Led: u32 {
        const WLR_LED_NUM_LOCK = wlr_keyboard_led::WLR_LED_NUM_LOCK as u32;
        const WLR_LED_CAPS_LOCK = wlr_keyboard_led::WLR_LED_CAPS_LOCK as u32;
        const WLR_LED_SCROLL_LOCK = wlr_keyboard_led::WLR_LED_SCROLL_LOCK as u32;
    }
}

bitflags! {
    pub struct Modifier: u32 {
        const WLR_MODIFIER_SHIFT = wlr_keyboard_modifier::WLR_MODIFIER_SHIFT as u32;
        const WLR_MODIFIER_CAPS = wlr_keyboard_modifier::WLR_MODIFIER_CAPS as u32;
        const WLR_MODIFIER_CTRL = wlr_keyboard_modifier::WLR_MODIFIER_CTRL as u32;
        const WLR_MODIFIER_ALT = wlr_keyboard_modifier::WLR_MODIFIER_ALT as u32;
        const WLR_MODIFIER_MOD2 = wlr_keyboard_modifier::WLR_MODIFIER_MOD2 as u32;
        const WLR_MODIFIER_MOD3 = wlr_keyboard_modifier::WLR_MODIFIER_MOD3 as u32;
        const WLR_MODIFIER_LOGO = wlr_keyboard_modifier::WLR_MODIFIER_LOGO as u32;
        const WLR_MODIFIER_MOD5 = wlr_keyboard_modifier::WLR_MODIFIER_MOD5 as u32;
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mod_vec = vec![
            ("Shift", Modifier::WLR_MODIFIER_SHIFT),
            ("Caps lock", Modifier::WLR_MODIFIER_CAPS),
            ("Ctrl", Modifier::WLR_MODIFIER_CTRL),
            ("Alt", Modifier::WLR_MODIFIER_ALT),
            ("Mod2", Modifier::WLR_MODIFIER_MOD2),
            ("Mod3", Modifier::WLR_MODIFIER_MOD3),
            ("Logo", Modifier::WLR_MODIFIER_LOGO),
            ("Mod5", Modifier::WLR_MODIFIER_MOD5),
        ];

        let mods: Vec<&str> = mod_vec
            .into_iter()
            .filter(|&(_, flag)| self.contains(flag))
            .map(|(st, _)| st)
            .collect();

        write!(formatter, "{:?}", mods)
    }
}
