//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use libc;

use std::{env, panic};
use std::process::abort;

use super::{KeyboardHandler, KeyboardWrapper, PointerHandler, PointerWrapper};
use compositor::{Compositor, COMPOSITOR_PTR};
use types::{InputDevice, Keyboard, Pointer};
use utils::safe_as_cstring;

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_input_device, wlr_input_device_type, wlr_keyboard_set_keymap,
                  xkb_context_new, xkb_context_unref, xkb_keymap_new_from_names, xkb_rule_names};
use wlroots_sys::xkb_context_flags::*;
use wlroots_sys::xkb_keymap_compile_flags::*;

/// Different type of inputs that can be acquired.
pub enum Input {
    Keyboard(Box<KeyboardWrapper>),
    Pointer(Box<PointerWrapper>)
}

impl Input {
    pub(crate) unsafe fn input_device(&self) -> *mut wlr_input_device {
        use self::Input::*;
        match *self {
            Keyboard(ref keyboard) => keyboard.input_device().as_ptr(),
            Pointer(ref pointer) => pointer.input_device().as_ptr()
        }
    }
}

/// Handles input addition and removal.
pub trait InputManagerHandler {
    /// Callback triggered when an input device is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn input_added(&mut self, &mut Compositor, &mut InputDevice) {}

    /// Callback triggered when an input device is removed.
    fn input_removed(&mut self, &mut Compositor, &mut InputDevice) {
        // TODO
    }

    /// Callback triggered when a keyboard device is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn keyboard_added(&mut self, &mut Compositor, &mut Keyboard) -> Option<Box<KeyboardHandler>> {
        None
    }

    /// Callback triggered when a pointer device is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn pointer_added(&mut self, &mut Compositor, &mut Pointer) -> Option<Box<PointerHandler>> {
        None
    }
}

wayland_listener!(InputManager, (Vec<Input>, Box<InputManagerHandler>), [
    add_listener => add_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        let data = data as *mut wlr_input_device;
        let (ref mut inputs, ref mut manager) = this.data;
        use self::wlr_input_device_type::*;
        let mut dev = InputDevice::from_ptr(data);
        let compositor = &mut *COMPOSITOR_PTR;
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            match dev.dev_type() {
                WLR_INPUT_DEVICE_KEYBOARD => {
                    // Boring setup that we won't make the user do
                    add_keyboard(&mut dev);
                    // Get the optional user keyboard struct, add the on_key signal
                    let mut keyboard_handle = match Keyboard::new_from_input_device(data) {
                        Some(dev) => dev,
                        None => {
                            wlr_log!(L_ERROR, "Device {:#?} was not a keyboard!", dev);
                            abort()
                        }
                    };
                    keyboard_handle.set_lock(true);
                    if let Some(keyboard_handler) = manager.keyboard_added(compositor,
                                                                           &mut keyboard_handle) {
                        keyboard_handle.set_lock(false);
                        let mut keyboard = KeyboardWrapper::new((keyboard_handle,
                                                                 keyboard_handler));
                        wl_signal_add(&mut (*dev.dev_union().keyboard).events.key as *mut _ as _,
                                    keyboard.key_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        inputs.push(Input::Keyboard(keyboard));
                    }
                },
                WLR_INPUT_DEVICE_POINTER => {
                    // Get the optional user pointer struct, add the signals
                    let mut pointer_handle = match Pointer::new_from_input_device(data) {
                        Some(dev) => dev,
                        None => {
                            wlr_log!(L_ERROR, "Device {:#?} was not a pointer!", dev);
                            abort()
                        }
                    };
                    pointer_handle.set_lock(true);
                    if let Some(pointer) = manager.pointer_added(compositor, &mut pointer_handle) {
                        pointer_handle.set_lock(false);
                        let mut pointer = PointerWrapper::new((pointer_handle, pointer));
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.motion as *mut _ as _,
                                    pointer.motion_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().pointer)
                                      .events.motion_absolute as *mut _ as _,
                                    pointer.motion_absolute_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.button as *mut _ as _,
                                    pointer.button_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().pointer).events.axis as *mut _ as _,
                                    pointer.axis_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        inputs.push(Input::Pointer(pointer))
                    }
                },
                _ => unimplemented!(),
            }
            manager.input_added(compositor, &mut dev)
        }));
        match res {
            Ok(_) => {},
            // NOTE
            // Either Wayland or wlroots does not handle failure to set up input correctly.
            // Calling wl_display_terminate does not work if input is incorrectly set up.
            //
            // Instead, execution keeps going with an eventual segfault (if lucky).
            //
            // To fix this, we abort the process if there was a panic in input setup.
            Err(_) => abort()
        }
    };
    remove_listener => remove_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        let data = data as *mut wlr_input_device;
        let (ref mut inputs, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        manager.input_removed(compositor, &mut InputDevice::from_ptr(data));
        // Remove user output data
        let find_index = inputs.iter()
            .position(|input| input.input_device() == data);
        if let Some(index) = find_index {
            let removed_input = inputs.remove(index);
            match removed_input {
                Input::Keyboard(mut keyboard) => {
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*keyboard.key_listener()).link as *mut _ as _);
                },
                Input::Pointer(mut pointer) => {
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pointer.button_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pointer.motion_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pointer.motion_absolute_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pointer.axis_listener()).link as *mut _ as _);
                }
            }
        }
    };
]);

pub(crate) unsafe fn add_keyboard(dev: &mut InputDevice) {
    // Set the XKB settings
    let rules = safe_as_cstring(env::var("XKB_DEFAULT_RULES").unwrap_or("".into()));
    let model = safe_as_cstring(env::var("XKB_DEFAULT_MODEL").unwrap_or("".into()));
    let layout = safe_as_cstring(env::var("XKB_DEFAULT_LAYOUT").unwrap_or("".into()));
    let variant = safe_as_cstring(env::var("XKB_DEFAULT_VARIANT").unwrap_or("".into()));
    let options = safe_as_cstring(env::var("XKB_DEFAULT_OPTIONS").unwrap_or("".into()));
    let rules = xkb_rule_names { rules: rules.into_raw(),
                                 model: model.into_raw(),
                                 layout: layout.into_raw(),
                                 variant: variant.into_raw(),
                                 options: options.into_raw() };
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
