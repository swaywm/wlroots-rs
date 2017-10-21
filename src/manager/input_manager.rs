//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use device::Device;
use key_event::KeyEvent;
use libc;
use std::env;
use utils::safe_as_cstring;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_event_keyboard_key, wlr_input_device, wlr_input_device_type,
                  wlr_keyboard_set_keymap, xkb_context_new, xkb_context_unref,
                  xkb_keymap_new_from_names, xkb_rule_names};
use wlroots_sys::xkb_context_flags::*;
use wlroots_sys::xkb_keymap_compile_flags::*;

/// Handles input addition and removal.
pub trait InputManagerHandler {
    /// Callback triggered when an input device is added.
    fn input_added(&mut self, Device) {
        // TODO?
    }

    /// Callback triggered when an input device is removed.
    fn input_removed(&mut self, Device) {
        // TODO
    }

    fn keyboard_added(&mut self, Device) {
        // TODO
    }

    fn pointer_added(&mut self, Device) {
        // TODO
    }

    fn key(&mut self, KeyEvent) {
        // TODO
    }

    fn motion(&mut self, Device) {
        // TODO
    }

    fn motion_absolute(&mut self, Device) {
        // TODO
    }

    fn button(&mut self, Device) {
        // TODO
    }

    fn axis(&mut self, Device) {
        // TODO
    }
}

wayland_listener!(InputManager, Box<InputManagerHandler>, [
    add_listener => add_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        use self::wlr_input_device_type::*;
        // TODO Ensure safety
        let mut dev = Device::from_ptr(data as *mut wlr_input_device);
        unsafe {
            match dev.dev_type() {
                WLR_INPUT_DEVICE_KEYBOARD => {
                    // Add the keyboard events to this manager
                    wl_signal_add(&mut (*dev.dev_union().keyboard).events.key as *mut _ as _,
                                  this.key_listener() as *mut _ as _);
                    add_keyboard(&mut dev);
                    this.data.keyboard_added(dev.clone())
                },
                WLR_INPUT_DEVICE_POINTER => {
                    // Add the pointer events to this manager
                    wl_signal_add(&mut (*dev.dev_union().pointer).events.motion as *mut _ as _,
                                  this.motion_listener() as *mut _ as _);
                    wl_signal_add(&mut (*dev.dev_union().pointer).events.motion_absolute as *mut _ as _,
                                  this.motion_absolute_listener() as *mut _ as _);
                    wl_signal_add(&mut (*dev.dev_union().pointer).events.button as *mut _ as _,
                                  this.button_listener() as *mut _ as _);
                    wl_signal_add(&mut (*dev.dev_union().pointer).events.axis as *mut _ as _,
                                  this.axis_listener() as *mut _ as _);
                    // Call user-defined callback
                    this.data.pointer_added(dev.clone())
                },
                _ => unimplemented!(), // TODO FIXME We _really_ shouldn't panic here
            }
        }
        this.data.input_added(dev)
    };
    remove_listener => remove_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        // TODO Ensure safety
        this.data.input_removed(Device::from_ptr(data as *mut wlr_input_device))
    };
    key_listener => key_notify:  |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        let key = KeyEvent::from_ptr(data as *mut wlr_event_keyboard_key);
        this.data.key(key)
    };
    motion_listener => motion_notify:  |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        // Ensure safety
        this.data.motion(Device::from_ptr(data as *mut wlr_input_device))
    };
    motion_absolute_listener => motion_absolute_notify:  |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        // Ensure safety
        this.data.motion_absolute(Device::from_ptr(data as *mut wlr_input_device))
    };
    button_listener => button_notify:  |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        // Ensure safety
        this.data.button(Device::from_ptr(data as *mut wlr_input_device))
    };
    axis_listener => axis_notify:  |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        // Ensure safety
        this.data.axis(Device::from_ptr(data as *mut wlr_input_device))
    };
]);

pub unsafe fn add_keyboard(dev: &mut Device) {
    // TODO add to global list

    // Set the XKB settings
    // TODO Unwrapping here is a little bad
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
