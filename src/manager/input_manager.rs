//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use super::{Keyboard, KeyboardHandler, Pointer, PointerHandler};
use device::Device;
use libc;
use std::env;
use utils::safe_as_cstring;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_input_device, wlr_input_device_type, wlr_keyboard_set_keymap,
                  xkb_context_new, xkb_context_unref, xkb_keymap_new_from_names, xkb_rule_names};
use wlroots_sys::xkb_context_flags::*;
use wlroots_sys::xkb_keymap_compile_flags::*;

/// Handles input addition and removal.
pub trait InputManagerHandler {
    /// Callback triggered when an input device is added.
    fn input_added(&mut self, &mut Device) {}

    /// Callback triggered when an input device is removed.
    fn input_removed(&mut self, &mut Device) {
        // TODO
    }

    fn keyboard_added(&mut self, &mut Device) -> Option<Box<KeyboardHandler>> {
        None
    }

    fn pointer_added(&mut self, &mut Device) -> Option<Box<PointerHandler>> {
        None
    }
}

wayland_listener!(InputManager, Box<InputManagerHandler>, [
    add_listener => add_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        use self::wlr_input_device_type::*;
        let mut dev = Device::from_ptr(data as *mut wlr_input_device);
        unsafe {
            match dev.dev_type() {
                WLR_INPUT_DEVICE_KEYBOARD => {
                    // Boring setup that we won't make the user do
                    add_keyboard(&mut dev);
                    // Get the optional user keyboard struct, add the on_key signal
                    if let Some(keyboard) = this.data.keyboard_added(&mut dev) {
                        let dev_ = Device::from_ptr(data as *mut wlr_input_device);
                        let mut keyboard = Keyboard::new((dev_, keyboard));
                        wl_signal_add(&mut (*dev.dev_union().keyboard).events.key as *mut _ as _,
                                    keyboard.key_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        ::std::mem::forget(keyboard);
                    }
                },
                WLR_INPUT_DEVICE_POINTER => {
                    // Get the optional user pointer struct, add the signals
                    if let Some(pointer) = this.data.pointer_added(&mut dev) {
                        let dev_ = Device::from_ptr(data as *mut wlr_input_device);
                        let mut pointer = Pointer::new((dev_, pointer));
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.motion as *mut _ as _,
                                    pointer.motion_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.motion_absolute as *mut _ as _,
                                    pointer.motion_absolute_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.button as *mut _ as _,
                                    pointer.button_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.axis as *mut _ as _,
                                    pointer.axis_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        ::std::mem::forget(pointer)
                    }
                },
                _ => unimplemented!(), // TODO FIXME We _really_ shouldn't panic here
            }
        }
        this.data.input_added(&mut dev)
    };
    remove_listener => remove_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        this.data.input_removed(&mut Device::from_ptr(data as *mut wlr_input_device))
    };
]);

pub unsafe fn add_keyboard(dev: &mut Device) {
    // Set the XKB settings
    let rules = safe_as_cstring(env::var("XKB_DEFAULT_RULES").unwrap_or("".into()));
    let model = safe_as_cstring(env::var("XKB_DEFAULT_MODEL").unwrap_or("".into()));
    let layout = safe_as_cstring(env::var("XKB_DEFAULT_LAYOUT").unwrap_or("".into()));
    let variant = safe_as_cstring(env::var("XKB_DEFAULT_VARIANT").unwrap_or("".into()));
    let options = safe_as_cstring(env::var("XKB_DEFAULT_OPTIONS").unwrap_or("".into()));
    let rules = xkb_rule_names {
        rules: rules.into_raw(),
        model: model.into_raw(),
        layout: layout.into_raw(),
        variant: variant.into_raw(),
        options: options.into_raw()
    };
    let context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
    if context.is_null() {
        wlr_log!(L_ERROR, "Failed to create XKB context");
        // NOTE We don't panic here, because we have a C call stack above us
        ::std::process::exit(1)
    }
    let xkb_map = xkb_keymap_new_from_names(context, &rules, XKB_KEYMAP_COMPILE_NO_FLAGS);
    wlr_keyboard_set_keymap(dev.dev_union().keyboard, xkb_map);
    xkb_context_unref(context);
}
