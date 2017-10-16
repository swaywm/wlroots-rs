//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during initialization.

use ::utils::safe_as_cstring;
use super::io_manager::IOManager;
use ::device::Device;
use wlroots_sys::{wlr_input_device, wlr_input_device_type, wl_listener,
                  wl_list, xkb_rule_names, xkb_context_new, xkb_context_unref,
                  xkb_keymap_new_from_names, wlr_keyboard_set_keymap};
use wlroots_sys::xkb_context_flags::*;
use wlroots_sys::xkb_keymap_compile_flags::*;
use wayland_sys::server::{WAYLAND_SERVER_HANDLE};
use wayland_sys::server::signal::wl_signal_add;
use libc;
use std::ops::{Deref, DerefMut};
use std::{env, ptr, mem};

/// Handles input addition and removal.
pub trait InputManagerHandler {
    /// Callback triggered when an input device is added.
    fn input_added(&mut self, Device);
    /// Callback triggered when an input device is removed.
    fn input_removed(&mut self, Device);
}

#[repr(C)]
/// Holds the user-defined input manager.
/// Pass this to the `Compositor` during initialization.
pub struct InputManager(Box<IOManager<Box<InputManagerHandler>>>);

impl InputManager {
    pub fn new(input_manager: Box<InputManagerHandler>) -> Self {
        InputManager(Box::new(IOManager::new(input_manager,
                                    InputManager::input_add_notify,
                                    InputManager::input_remove_notify)))
    }

    unsafe extern "C" fn input_add_notify(listener: *mut wl_listener,
                                          data: *mut libc::c_void) {
        let device = data as *mut wlr_input_device;
        let input_wrapper = container_of!(listener,
                                          IOManager<Box<InputManagerHandler>>,
                                          add_listener);
        let input_manager = &mut (*input_wrapper).manager;
        // TODO FIXME
        // Ensure this is safe
        input_manager.input_added(Device::from_ptr(device))
    }

    unsafe extern "C" fn input_remove_notify(listener: *mut wl_listener,
                                             data: *mut libc::c_void) {
        let device = data as *mut wlr_input_device;
        let input_wrapper = container_of!(listener, InputManager, remove_listener);
        let input_manager = &mut (*input_wrapper).manager;
        // TODO FIXME
        // Ensure this is safe
        input_manager.input_removed(Device::from_ptr(device))
    }
}

impl Deref for InputManager {
    type Target = IOManager<Box<InputManagerHandler>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InputManager {
    fn deref_mut(&mut self) -> &mut IOManager<Box<InputManagerHandler>> {
        &mut self.0
    }
}

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
                WLR_INPUT_DEVICE_KEYBOARD => {
                    self.add_keyboard(dev)
                },
                WLR_INPUT_DEVICE_POINTER => {
                    self.add_pointer(dev)
                }
                _ => unimplemented!() // TODO FIXME We _really_ shouldn't panic here
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
        let rules = safe_as_cstring(env::var("XKB_DEFAULT_RULES")
                                    .unwrap_or("".into()));
        let model = safe_as_cstring(env::var("XKB_DEFAULT_MODEL")
                                 .unwrap_or("".into()));
        let layout = safe_as_cstring(env::var("XKB_DEFAULT_LAYOUT")
                                  .unwrap_or("".into()));
        let variant = safe_as_cstring(env::var("XKB_DEFAULT_VARIANT")
                                   .unwrap_or("".into()));
        let options = safe_as_cstring(env::var("XKB_DEFAULT_OPTIONS")
                                   .unwrap_or("".into()));
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

    pub unsafe extern "C" fn motion_notify(listener: *mut wl_listener,
                                           data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn motion_absolute_notify(listener: *mut wl_listener,
                                           data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn button_notify(listener: *mut wl_listener,
                                           data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn axis_notify(listener: *mut wl_listener,
                                           data: *mut libc::c_void) {
        unimplemented!()
    }

    pub unsafe extern "C" fn key_notify(listener: *mut wl_listener,
                                         data: *mut libc::c_void) {
        // TODO implement
    }
}
