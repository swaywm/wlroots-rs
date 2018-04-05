//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use libc;

use std::{env, panic};
use std::process::abort;

use super::{KeyboardHandler, KeyboardWrapper, PointerHandler, PointerWrapper, TabletPadHandler,
            TabletPadWrapper, TabletToolHandler, TabletToolWrapper, TouchHandler, TouchWrapper};
use compositor::{Compositor, COMPOSITOR_PTR};
use types::input::{InputDevice, Keyboard, Pointer, TabletPad, TabletTool, Touch};
use utils::safe_as_cstring;

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_input_device, wlr_input_device_type, wlr_keyboard_set_keymap,
                  wlr_keyboard_set_repeat_info, xkb_context_new, xkb_context_unref,
                  xkb_keymap_new_from_names, xkb_keymap_unref, xkb_rule_names};
use wlroots_sys::xkb_context_flags::*;
use wlroots_sys::xkb_keymap_compile_flags::*;

/// Different type of inputs that can be acquired.
pub enum Input {
    Keyboard(Box<KeyboardWrapper>),
    Pointer(Box<PointerWrapper>),
    Touch(Box<TouchWrapper>),
    TabletTool(Box<TabletToolWrapper>),
    TabletPad(Box<TabletPadWrapper>)
}

impl Input {
    pub(crate) unsafe fn input_device(&self) -> *mut wlr_input_device {
        use self::Input::*;
        match *self {
            Keyboard(ref keyboard) => keyboard.input_device().as_ptr(),
            Pointer(ref pointer) => pointer.input_device().as_ptr(),
            Touch(ref touch) => touch.input_device().as_ptr(),
            TabletTool(ref tool) => tool.input_device().as_ptr(),
            TabletPad(ref pad) => pad.input_device().as_ptr()
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
    fn input_removed(&mut self, &mut Compositor, &mut InputDevice) {}

    /// Callback triggered when a keyboard device is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn keyboard_added(&mut self, &mut Compositor, &mut Keyboard) -> Option<Box<KeyboardHandler>> {
        None
    }

    /// Callback triggered when a keyboard device is removed.
    fn keyboard_removed(&mut self, &mut Compositor, &mut Keyboard) {}

    /// Callback triggered when a pointer device is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn pointer_added(&mut self, &mut Compositor, &mut Pointer) -> Option<Box<PointerHandler>> {
        None
    }

    /// Callback triggered when a pointer device is removed.
    fn pointer_removed(&mut self, &mut Compositor, &mut Pointer) {}

    /// Callback triggered when a touch device is added.
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn touch_added(&mut self, &mut Compositor, &mut Touch) -> Option<Box<TouchHandler>> {
        None
    }

    /// Callback triggered when a touch device is removed.
    fn touch_removed(&mut self, &mut Compositor, &mut Touch) {}

    /// Callback triggered when a tablet tool is added.
    ///
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn tablet_tool_added(&mut self,
                         &mut Compositor,
                         &mut TabletTool)
                         -> Option<Box<TabletToolHandler>> {
        None
    }

    /// Callback triggered when a touch device is removed.
    fn tablet_tool_removed(&mut self, &mut Compositor, &mut TabletTool) {}

    /// Callback triggered when a tablet pad is added.
    ///
    ///
    /// # Panics
    /// Any panic in this function will cause the process to abort.
    fn tablet_pad_added(&mut self,
                        &mut Compositor,
                        &mut TabletPad)
                        -> Option<Box<TabletPadHandler>> {
        None
    }

    /// Callback triggered when a touch device is removed.
    fn tablet_pad_removed(&mut self, &mut Compositor, &mut TabletPad) {}
}

wayland_listener!(InputManager, (Vec<Input>, Box<InputManagerHandler>), [
    add_listener => add_notify: |this: &mut InputManager, data: *mut libc::c_void,| unsafe {
        let data = data as *mut wlr_input_device;
        let remove_listener = this.remove_listener()  as *mut _ as _;
        let (ref mut inputs, ref mut manager) = this.data;
        use self::wlr_input_device_type::*;
        let mut dev = InputDevice::from_ptr(data);
        let compositor = &mut *COMPOSITOR_PTR;
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            match dev.dev_type() {
                WLR_INPUT_DEVICE_KEYBOARD => {
                    // Boring setup that we won't make the user do
                    add_keyboard(&mut dev);
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
                        wl_signal_add(&mut (*dev.dev_union().keyboard).events.modifiers
                                      as *mut _ as _,
                                      keyboard.modifiers_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().keyboard).events.keymap as *mut _ as _,
                                      keyboard.keymap_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().keyboard).events.repeat_info
                                      as *mut _ as _,
                                      keyboard.repeat_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        inputs.push(Input::Keyboard(keyboard));
                    }
                },
                WLR_INPUT_DEVICE_POINTER => {
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
                WLR_INPUT_DEVICE_TOUCH => {
                    let mut touch_handle = match Touch::new_from_input_device(data) {
                        Some(dev) => dev,
                        None => {
                            wlr_log!(L_ERROR, "Device {:#?} was not a touch", dev);
                            abort()
                        }
                    };
                    touch_handle.set_lock(true);
                    if let Some(touch) = manager.touch_added(compositor, &mut touch_handle) {
                        touch_handle.set_lock(false);
                        let mut touch = TouchWrapper::new((touch_handle, touch));
                        wl_signal_add(&mut (*dev.dev_union().touch).events.down as *mut _ as _,
                                      touch.down_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().touch).events.up as *mut _ as _,
                                      touch.up_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().touch).events.motion as *mut _ as _,
                                      touch.motion_listener() as *mut _ as _);
                        wl_signal_add(&mut (*dev.dev_union().touch).events.cancel as *mut _ as _,
                                      touch.cancel_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        inputs.push(Input::Touch(touch))
                    }
                },
                WLR_INPUT_DEVICE_TABLET_TOOL => {
                    let mut tablet_tool_handle = match TabletTool::new_from_input_device(data) {
                        Some(dev) => dev,
                        None => {
                            wlr_log!(L_ERROR, "Device {:#?}, was not a tablet tool", dev);
                            abort()
                        }
                    };
                    tablet_tool_handle.set_lock(true);
                    if let Some(tablet_tool) = manager.tablet_tool_added(compositor,
                                                                         &mut tablet_tool_handle) {
                        tablet_tool_handle.set_lock(false);
                        let mut tablet_tool = TabletToolWrapper::new((tablet_tool_handle,
                                                                      tablet_tool));
                        let tool_ptr = &mut (*dev.dev_union().tablet_tool);
                        wl_signal_add(&mut tool_ptr.events.axis as *mut _ as _,
                                      tablet_tool.axis_listener() as *mut _ as _);
                        wl_signal_add(&mut tool_ptr.events.proximity as *mut _ as _,
                                      tablet_tool.proximity_listener() as *mut _ as _);
                        wl_signal_add(&mut tool_ptr.events.tip as *mut _ as _,
                                      tablet_tool.tip_listener() as *mut _ as _);
                        wl_signal_add(&mut tool_ptr.events.button as *mut _ as _,
                                      tablet_tool.button_listener() as *mut _ as _);
                        // Forget until we need to drop it in the destroy callback
                        inputs.push(Input::TabletTool(tablet_tool))
                    }
                },
                WLR_INPUT_DEVICE_TABLET_PAD => {
                    let mut tablet_pad_handle = match TabletPad::new_from_input_device(data) {
                        Some(dev) => dev,
                        None => {
                            wlr_log!(L_ERROR, "Device {:#?}, was not a tablet pad", dev);
                            abort()
                        }
                    };
                    tablet_pad_handle.set_lock(true);
                    if let Some(tablet_pad) = manager.tablet_pad_added(compositor,
                                                                       &mut tablet_pad_handle) {
                        tablet_pad_handle.set_lock(false);
                        let mut tablet_pad = TabletPadWrapper::new((tablet_pad_handle, tablet_pad));
                        let pad_ptr = &mut (*dev.dev_union().tablet_pad);
                        wl_signal_add(&mut pad_ptr.events.button as *mut _ as _,
                                      tablet_pad.button_listener() as *mut _ as _);;
                        wl_signal_add(&mut pad_ptr.events.ring as *mut _ as _,
                                      tablet_pad.ring_listener() as *mut _ as _);;
                        wl_signal_add(&mut pad_ptr.events.strip as *mut _ as _,
                                      tablet_pad.strip_listener() as *mut _ as _);;
                        // Forget until we need to drop it in the destroy callback
                        inputs.push(Input::TabletPad(tablet_pad))
                    }
                }
            }
            manager.input_added(compositor, &mut dev)
        }));
        wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                      remove_listener);
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
        if COMPOSITOR_PTR.is_null() {
            // We are shutting down, do nothing.
            return;
        }
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
                    manager.keyboard_removed(compositor, keyboard.keyboard());
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
                    manager.pointer_removed(compositor, pointer.pointer());
                },
                Input::Touch(mut touch) => {
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*touch.down_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*touch.up_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*touch.motion_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*touch.cancel_listener()).link as *mut _ as _);
                    manager.touch_removed(compositor, touch.touch());
                },
                Input::TabletTool(mut tool) => {
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*tool.axis_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*tool.proximity_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*tool.tip_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*tool.button_listener()).link as *mut _ as _);
                    manager.tablet_tool_removed(compositor, tool.tablet_tool());
                },
                Input::TabletPad(mut pad) => {
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pad.button_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pad.ring_listener()).link as *mut _ as _);
                    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_list_remove,
                                  &mut (*pad.strip_listener()).link as *mut _ as _);
                    manager.tablet_pad_removed(compositor, pad.tablet_pad());
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
    wlr_log!(L_DEBUG, "Using xkb rules: {:?}", rules);
    wlr_log!(L_DEBUG, "Using xkb model: {:?}", model);
    wlr_log!(L_DEBUG, "Using xkb layout: {:?}", layout);
    wlr_log!(L_DEBUG, "Using xkb variant: {:?}", variant);
    wlr_log!(L_DEBUG, "Using xkb options: {:?}", options);
    let rules = xkb_rule_names { rules: rules.into_raw(),
                                 model: model.into_raw(),
                                 layout: layout.into_raw(),
                                 variant: variant.into_raw(),
                                 options: options.into_raw() };
    let context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
    if context.is_null() {
        panic!("Failed to create XKB context");
    }
    let xkb_map = xkb_keymap_new_from_names(context, &rules, XKB_KEYMAP_COMPILE_NO_FLAGS);
    if xkb_map.is_null() {
        panic!("Could not create xkb map");
    }
    wlr_keyboard_set_keymap(dev.dev_union().keyboard, xkb_map);
    xkb_keymap_unref(xkb_map);
    xkb_context_unref(context);
    wlr_keyboard_set_repeat_info(dev.dev_union().keyboard, 25, 600);
}
