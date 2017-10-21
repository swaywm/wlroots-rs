//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use device::Device;
use libc;
use std::{env, mem, ptr};
use utils::safe_as_cstring;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wl_list, wl_listener, wlr_input_device, wlr_input_device_type,
                  wlr_keyboard_set_keymap, xkb_context_new, xkb_context_unref,
                  xkb_keymap_new_from_names, xkb_rule_names};
use wlroots_sys::xkb_context_flags::*;
use wlroots_sys::xkb_keymap_compile_flags::*;

/// Handles input addition and removal.
pub trait InputManagerHandler {
    /// Callback triggered when an input device is added.
    fn input_added(&mut self, Device);
    /// Callback triggered when an input device is removed.
    fn input_removed(&mut self, Device);
}

wayland_listener!(InputManager, Box<InputManagerHandler>, [
    add_listener => add_notify: |input_manager: &mut Box<InputManagerHandler>, data: *mut libc::c_void,| unsafe {
        input_manager.input_added(Device::from_ptr(data as *mut wlr_input_device))
    };
    remove_listener => remove_notify: |input_manager: &mut Box<InputManagerHandler>, data: *mut libc::c_void,| unsafe {
        input_manager.input_removed(Device::from_ptr(data as *mut wlr_input_device))
    };
]);

pub struct DefaultInputHandler {
    dev: Device,
    // TODO This should be in a nested struct, shouldn't it?
    // TODO In fact there should be a couple of these, i.e in a Vec
    // or whatever.
    motion: wl_listener,
    motion_absolute: wl_listener,
    button: wl_listener,
    axis: wl_listener,
    key: wl_listener,
    link: wl_list,
    data: *mut libc::c_void
}

impl InputManagerHandler for DefaultInputHandler {
    fn input_added(&mut self, dev: Device) {
        use self::wlr_input_device_type::*;
        unsafe {
            match dev.dev_type() {
                WLR_INPUT_DEVICE_KEYBOARD => self.add_keyboard(dev),
                WLR_INPUT_DEVICE_POINTER => self.add_pointer(dev),
                _ => unimplemented!(), // TODO FIXME We _really_ shouldn't panic here
            }
        }
    }

    fn input_removed(&mut self, dev: Device) {
        unimplemented!()
    }
}

impl DefaultInputHandler {
    pub fn new() -> Self {
        unsafe {
            // TODO FIXME Very stupid
            mem::zeroed()
        }
    }

    pub unsafe fn add_keyboard(&mut self, dev: Device) {
        ptr::write(&mut self.dev, dev);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_init,
                      &mut self.key.link as *mut _ as _);
        ptr::write(&mut self.key.notify, Some(Self::key_notify));
        wl_signal_add(&mut (*self.dev.dev_union().keyboard).events.key as *mut _ as _,
                      &mut self.key as *mut _ as _);
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
        wlr_keyboard_set_keymap(self.dev.dev_union().keyboard, xkb_map);
        xkb_context_unref(context);
    }

    pub unsafe fn add_pointer(&mut self, dev: Device) {
        ptr::write(&mut self.dev, dev);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_init,
                      &mut self.motion.link as *mut _ as _);
        ptr::write(&mut self.motion.notify, Some(Self::motion_notify));
        wl_signal_add(&mut (*self.dev.dev_union().pointer).events.motion as *mut _ as _,
                      &mut self.motion as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_init,
                      &mut self.motion_absolute.link as *mut _ as _);
        wl_signal_add(&mut (*self.dev.dev_union().pointer).events.motion_absolute as *mut _ as _,
                      &mut self.motion_absolute as *mut _ as _);
        ptr::write(&mut self.motion.notify, Some(Self::motion_absolute_notify));
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_init,
                      &mut self.button.link as *mut _ as _);
        ptr::write(&mut self.button.notify, Some(Self::button_notify));
        wl_signal_add(&mut (*self.dev.dev_union().pointer).events.button as *mut _ as _,
                      &mut self.button as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_init,
                      &mut self.axis.link as *mut _ as _);
        ptr::write(&mut self.axis.notify, Some(Self::axis_notify));
        wl_signal_add(&mut (*self.dev.dev_union().pointer).events.axis as *mut _ as _,
                      &mut self.axis as *mut _ as _);
        // TODO add to global list
    }

    // TODO implement, wrap properly in InputManagerHandler

    pub unsafe extern "C" fn motion_notify(listener: *mut wl_listener, data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn motion_absolute_notify(listener: *mut wl_listener,
                                                    data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn button_notify(listener: *mut wl_listener, data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn axis_notify(listener: *mut wl_listener, data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn key_notify(listener: *mut wl_listener, data: *mut libc::c_void) {
        // TODO implement
    }
}
