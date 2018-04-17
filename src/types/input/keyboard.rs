//! TODO Documentation
use std::{fmt, panic, ptr, rc::{Rc, Weak}, sync::atomic::{AtomicBool, Ordering}};

use errors::{HandleErr, HandleResult};
use wlroots_sys::{wlr_input_device, wlr_keyboard, wlr_keyboard_get_modifiers, wlr_keyboard_led,
                  wlr_keyboard_led_update, wlr_keyboard_modifier, wlr_keyboard_set_keymap};
pub use wlroots_sys::{wlr_key_state, wlr_keyboard_modifiers};

use xkbcommon::xkb::{self, Keycode, Keymap, LedIndex, ModIndex};
use xkbcommon::xkb::ffi::{xkb_keymap, xkb_state};

use InputDevice;

struct KeyboardState {
    handle: Weak<AtomicBool>,
    device: InputDevice
}

/// Information about repeated keypresses for a particular Keyboard.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RepeatInfo {
    /// The rate at which extended keypresses will fire more events.
    pub rate: i32,
    /// How long it takes for a keypress to register on this device.
    pub delay: i32
}

#[derive(Debug)]
pub struct Keyboard {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `KeyboardHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `KeyboardHandle`.
    liveliness: Option<Rc<AtomicBool>>,
    /// The device that refers to this keyboard.
    device: InputDevice,
    /// The underlying keyboard data.
    keyboard: *mut wlr_keyboard
}

#[derive(Debug)]
pub struct KeyboardHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the keyboard associated with this handle,
    handle: Weak<AtomicBool>,
    /// The device that refers to this keyboard.
    device: InputDevice,
    /// The underlying keyboard data.
    keyboard: *mut wlr_keyboard
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
                let keyboard = (*device).__bindgen_anon_1.keyboard;
                let liveliness = Rc::new(AtomicBool::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(KeyboardState { handle,
                                                     device: InputDevice::from_ptr(device) });
                (*keyboard).data = Box::into_raw(state) as *mut _;
                Some(Keyboard { liveliness: Some(liveliness),
                                device: InputDevice::from_ptr(device),
                                keyboard })
            }
            _ => None
        }
    }

    unsafe fn from_handle(handle: &KeyboardHandle) -> HandleResult<Self> {
        Ok(Keyboard { liveliness: None,
                      device: handle.input_device()?.clone(),
                      keyboard: handle.as_ptr() })
    }

    /// Gets the wlr_keyboard associated with this KeyboardHandle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_keyboard {
        self.keyboard
    }

    /// Gets the wlr_input_device associated with this KeyboardHandle
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    /// Set the keymap for this Keyboard.
    pub fn set_keymap(&mut self, keymap: &Keymap) {
        unsafe {
            // NOTE wlr_keyboard_set_keymap updates the reference count,
            // so we don't need to mem::forget the key map here
            // or take it by value.
            wlr_keyboard_set_keymap(self.keyboard, keymap.get_raw_ptr() as _);
        }
    }

    /// Get the XKB keymap associated with this Keyboard.
    pub fn get_keymap(&mut self) -> Option<Keymap> {
        unsafe {
            let keymap_ptr = (*self.keyboard).keymap as *mut xkb_keymap;
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
            let mut result = (*self.keyboard).keycodes.to_vec();
            result.truncate((*self.keyboard).num_keycodes);
            result
        }
    }

    /// Get the list of LEDs for this keyboard as reported by XKB.
    pub fn led_list(&self) -> &[LedIndex] {
        unsafe { &(*self.keyboard).led_indexes }
    }

    /// Get the list of modifiers for this keyboard as reported by XKB.
    pub fn modifier_list(&self) -> &[ModIndex] {
        unsafe { &(*self.keyboard).mod_indexes }
    }

    /// Get the size of the keymap.
    pub fn keymap_size(&self) -> usize {
        unsafe { (*self.keyboard).keymap_size }
    }

    /// Get the XKB state associated with this `Keyboard`.
    pub fn get_xkb_state(&mut self) -> Option<xkb::State> {
        unsafe {
            let xkb_state_ptr = (*self.keyboard).xkb_state as *mut xkb_state;
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
            RepeatInfo { rate: (*self.keyboard).repeat_info.rate,
                         delay: (*self.keyboard).repeat_info.delay }
        }
    }

    /// Update the LED lights using the provided bitmap.
    ///
    /// 1 means one, 0 means off.
    pub fn update_led(&mut self, leds: KeyboardLed) {
        unsafe {
            wlr_keyboard_led_update(self.keyboard, leds.bits() as u32);
        }
    }

    /// Get the modifiers that are currently pressed on the keyboard.
    pub fn get_modifiers(&self) -> KeyboardModifier {
        unsafe { KeyboardModifier::from_bits_truncate(wlr_keyboard_get_modifiers(self.keyboard)) }
    }

    /// Get the modifier masks for each group.
    pub fn get_modifier_masks(&self) -> wlr_keyboard_modifiers {
        unsafe { (*self.keyboard).modifiers }
    }

    /// Creates a weak reference to a `Keyboard`.
    ///
    /// # Panics
    /// If this `Keyboard` is a previously upgraded `KeyboardHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> KeyboardHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade previously upgraded KeyboardHandle!");
        KeyboardHandle { handle: Rc::downgrade(arc),
                         // NOTE Rationale for cloning:
                         // We can't use the keyboard handle unless the keyboard is alive,
                         // which means the device pointer is still alive.
                         device: unsafe { self.device.clone() },
                         keyboard: self.keyboard }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed Keyboard")
            .store(val, Ordering::Release);
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        match self.liveliness {
            None => {}
            Some(ref liveliness) => {
                if Rc::strong_count(liveliness) == 1 {
                    wlr_log!(L_DEBUG, "Dropped Keyboard {:p}", self.keyboard);
                    unsafe {
                        let _ = Box::from_raw((*self.keyboard).data as *mut KeyboardState);
                    }
                    let weak_count = Rc::weak_count(liveliness);
                    if weak_count > 0 {
                        wlr_log!(L_DEBUG,
                                 "Still {} weak pointers to Keyboard {:p}",
                                 weak_count,
                                 self.keyboard);
                    }
                }
            }
        }
    }
}

impl KeyboardHandle {
    /// Constructs a new KeyboardHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            KeyboardHandle { handle: Weak::new(),
                             // NOTE Rationale for null pointer here:
                             // It's never used, because you can never upgrade it,
                             // so no way to dereference it and trigger UB.
                             device: InputDevice::from_ptr(ptr::null_mut()),
                             keyboard: ptr::null_mut() }
        }
    }

    /// Creates an KeyboardHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(keyboard: *mut wlr_keyboard) -> Self {
        let data = Box::from_raw((*keyboard).data as *mut KeyboardState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*keyboard).data = Box::into_raw(data) as *mut _;
        KeyboardHandle { handle,
                         keyboard,
                         device }
    }

    /// Upgrades the keyboard handle to a reference to the backing `Keyboard`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbounded `Keyboard`
    /// which may live forever..
    /// But no keyboard lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Keyboard> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let keyboard = Keyboard::from_handle(self)?;
                if check.load(Ordering::Acquire) {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(keyboard)
            })
    }

    /// Run a function on the referenced Keyboard, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Keyboard does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut Keyboard) -> R
    {
        let mut keyboard = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut keyboard)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running keyboard callback, mutable \
                                                    lock was false for: {:?}",
                                                   keyboard);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this KeyboardHandle
    pub fn input_device(&self) -> HandleResult<&InputDevice> {
        match self.handle.upgrade() {
            Some(_) => Ok(&self.device),
            None => Err(HandleErr::AlreadyDropped)
        }
    }

    /// Gets the wlr_keyboard associated with this KeyboardHandle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_keyboard {
        self.keyboard
    }
}

impl Default for KeyboardHandle {
    fn default() -> Self {
        KeyboardHandle::new()
    }
}

impl Clone for KeyboardHandle {
    fn clone(&self) -> Self {
        KeyboardHandle { keyboard: self.keyboard,
                         handle: self.handle.clone(),
                         /// NOTE Rationale for unsafe clone:
                         ///
                         /// You can only access it after a call to `upgrade`,
                         /// and that implicitly checks that it is valid.
                         device: unsafe { self.device.clone() } }
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
        let mod_vec = vec![("Shift", KeyboardModifier::WLR_MODIFIER_SHIFT),
                           ("Caps lock", KeyboardModifier::WLR_MODIFIER_CAPS),
                           ("Ctrl", KeyboardModifier::WLR_MODIFIER_CTRL),
                           ("Alt", KeyboardModifier::WLR_MODIFIER_ALT),
                           ("Mod2", KeyboardModifier::WLR_MODIFIER_MOD2),
                           ("Mod3", KeyboardModifier::WLR_MODIFIER_MOD3),
                           ("Logo", KeyboardModifier::WLR_MODIFIER_LOGO),
                           ("Mod5", KeyboardModifier::WLR_MODIFIER_MOD5)];

        let mods: Vec<&str> = mod_vec.into_iter()
                                     .filter(|&(_, flag)| self.contains(flag))
                                     .map(|(st, _)| st)
                                     .collect();

        write!(formatter, "{:?}", mods)
    }
}

impl PartialEq for KeyboardHandle {
    fn eq(&self, other: &KeyboardHandle) -> bool {
        self.keyboard == other.keyboard
    }
}

impl Eq for KeyboardHandle {}
