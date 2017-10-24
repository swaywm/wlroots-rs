use std::fmt;
use wlroots_sys::{wlr_keyboard, wlr_keyboard_get_modifiers, wlr_keyboard_led,
                  wlr_keyboard_led_update, wlr_keyboard_modifier, wlr_keyboard_set_keymap,
                  xkb_keymap};

#[derive(Debug)]
pub struct Keyboard {
    keyboard: *mut wlr_keyboard,
}

impl Keyboard {
    pub(crate) fn new(kb_pointer: *mut wlr_keyboard) -> Self {
        Keyboard { keyboard: kb_pointer }
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

    pub fn led_update(&mut self, leds: KeyboardLed) {
        unsafe {
            wlr_keyboard_led_update(self.keyboard, leds.state());
        }
    }

    pub fn get_modifiers(&self) -> KeyboardModifier {
        unsafe { KeyboardModifier::new(wlr_keyboard_get_modifiers(self.keyboard)) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyboardLed {
    state: u32,
}

impl KeyboardLed {
    pub fn new(num_lock: bool, caps_lock: bool, scroll_lock: bool) -> Self {
        let mut state = 0;

        if num_lock {
            state |= wlr_keyboard_led::WLR_LED_NUM_LOCK as u32
        };

        if caps_lock {
            state |= wlr_keyboard_led::WLR_LED_CAPS_LOCK as u32
        }

        if scroll_lock {
            state |= wlr_keyboard_led::WLR_LED_SCROLL_LOCK as u32
        }

        KeyboardLed { state: state }
    }

    fn state(&self) -> u32 {
        self.state
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyboardModifier {
    modifiers: u32,
}

macro_rules! keyboard_modifier_accessor {
    ($func_name: ident, $c_enum: expr) => [
        pub fn $func_name(&self) -> bool {
            (self.modifiers | $c_enum as u32) != 0
        }
    ]
}

impl KeyboardModifier {
    fn new(modifiers: u32) -> Self {
        KeyboardModifier { modifiers: modifiers }
    }

    keyboard_modifier_accessor!(shift, wlr_keyboard_modifier::WLR_MODIFIER_SHIFT);
    keyboard_modifier_accessor!(caps, wlr_keyboard_modifier::WLR_MODIFIER_CAPS);
    keyboard_modifier_accessor!(ctrl, wlr_keyboard_modifier::WLR_MODIFIER_CTRL);
    keyboard_modifier_accessor!(alt, wlr_keyboard_modifier::WLR_MODIFIER_ALT);
    keyboard_modifier_accessor!(mod2, wlr_keyboard_modifier::WLR_MODIFIER_MOD2);
    keyboard_modifier_accessor!(mod3, wlr_keyboard_modifier::WLR_MODIFIER_MOD3);
    keyboard_modifier_accessor!(logo, wlr_keyboard_modifier::WLR_MODIFIER_LOGO);
    keyboard_modifier_accessor!(mod5, wlr_keyboard_modifier::WLR_MODIFIER_MOD5);
}

impl fmt::Display for KeyboardModifier {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mod_vec: Vec<(&str, &Fn(&KeyboardModifier) -> bool)> =
            vec![("Shift", &KeyboardModifier::shift),
                 ("Caps lock", &KeyboardModifier::caps),
                 ("Ctrl", &KeyboardModifier::ctrl),
                 ("Alt", &KeyboardModifier::alt),
                 ("Mod2", &KeyboardModifier::mod2),
                 ("Mod3", &KeyboardModifier::mod3),
                 ("Logo", &KeyboardModifier::logo),
                 ("Mod5", &KeyboardModifier::mod5)];

        let mods: Vec<&str> = mod_vec.into_iter()
            .filter(|&(_, func)| func(self))
            .map(|(st, _)| st)
            .collect();

        write!(formatter, "Modifiers: {:?}", mods)
    }
}