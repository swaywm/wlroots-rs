use std::fmt;
use wlroots_sys::{wlr_input_device, wlr_keyboard, wlr_keyboard_get_modifiers, wlr_keyboard_led,
                  wlr_keyboard_led_update, wlr_keyboard_modifier, wlr_keyboard_set_keymap,
                  xkb_keymap};

#[derive(Debug)]
pub struct KeyboardHandle {
    device: *mut wlr_input_device,
    keyboard: *mut wlr_keyboard
}

impl KeyboardHandle {
    pub(crate) unsafe fn from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_KEYBOARD => {
                let keyboard = (*device).__bindgen_anon_1.keyboard;
                Some(KeyboardHandle { device, keyboard })
            }
            _ => None,
        }
    }

    pub(crate) unsafe fn to_ptr(&self) -> *mut wlr_keyboard {
        self.keyboard
    }

    // TODO: Implement keymap wrapper?
    pub fn set_keymap(&mut self, keymap: *mut xkb_keymap) {
        unsafe {
            wlr_keyboard_set_keymap(self.keyboard, keymap);
        }
    }

    pub fn update_led(&mut self, leds: KeyboardLed) {
        unsafe {
            wlr_keyboard_led_update(self.keyboard, leds.bits() as u32);
        }
    }

    pub fn get_modifiers(&self) -> KeyboardModifier {
        unsafe { KeyboardModifier::from_bits_truncate(wlr_keyboard_get_modifiers(self.keyboard)) }
    }

    pub unsafe fn input_device(&self) -> *mut wlr_input_device {
        self.device
    }
}

bitflags! {
    pub struct KeyboardLed: u32 {
        const WLR_LED_NUM_LOCK = wlr_keyboard_led::WLR_LED_NUM_LOCK as u32;
        const WLR_LED_CAPS_LOCK = wlr_keyboard_led::WLR_LED_CAPS_LOCK as u32;
        const WLR_LED_SCROLL_LOCK = wlr_keyboard_led::WLR_LED_SCROLL_LOCK as u32;
    }
}

bitflags! {
    pub struct KeyboardModifier: u32 {
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

impl fmt::Display for KeyboardModifier {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mod_vec = vec![
            ("Shift", KeyboardModifier::WLR_MODIFIER_SHIFT),
            ("Caps lock", KeyboardModifier::WLR_MODIFIER_CAPS),
            ("Ctrl", KeyboardModifier::WLR_MODIFIER_CTRL),
            ("Alt", KeyboardModifier::WLR_MODIFIER_ALT),
            ("Mod2", KeyboardModifier::WLR_MODIFIER_MOD2),
            ("Mod3", KeyboardModifier::WLR_MODIFIER_MOD3),
            ("Logo", KeyboardModifier::WLR_MODIFIER_LOGO),
            ("Mod5", KeyboardModifier::WLR_MODIFIER_MOD5),
        ];

        let mods: Vec<&str> = mod_vec
            .into_iter()
            .filter(|&(_, flag)| self.contains(flag))
            .map(|(st, _)| st)
            .collect();

        write!(formatter, "{:?}", mods)
    }
}
