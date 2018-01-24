//! Wrapper for wlr_seat. For more information about what a seat is, please
//! consult the Wayland documentation ([libinput docs](https://wayland.freedesktop.org/libinput/doc/latest/seats.html), [wayland docs](https://wayland.freedesktop.org/docs/html/apa.html#protocol-spec-wl_seat))
//!
//! TODO This module could really use some examples, as the API surface is huge.

use std::time::Duration;

use xkbcommon::xkb::Keycode;

use wlroots_sys::{wlr_axis_orientation, wlr_seat, wlr_seat_create, wlr_seat_destroy,
                  wlr_seat_keyboard_clear_focus, wlr_seat_keyboard_end_grab,
                  wlr_seat_keyboard_enter, wlr_seat_keyboard_has_grab,
                  wlr_seat_keyboard_notify_enter, wlr_seat_keyboard_notify_key,
                  wlr_seat_keyboard_notify_modifiers, wlr_seat_keyboard_send_key,
                  wlr_seat_keyboard_send_modifiers, wlr_seat_keyboard_start_grab,
                  wlr_seat_pointer_clear_focus, wlr_seat_pointer_end_grab, wlr_seat_pointer_enter,
                  wlr_seat_pointer_has_grab, wlr_seat_pointer_notify_axis,
                  wlr_seat_pointer_notify_button, wlr_seat_pointer_notify_enter,
                  wlr_seat_pointer_notify_motion, wlr_seat_pointer_send_axis,
                  wlr_seat_pointer_send_button, wlr_seat_pointer_send_motion,
                  wlr_seat_pointer_start_grab, wlr_seat_pointer_surface_has_focus,
                  wlr_seat_set_capabilities, wlr_seat_set_keyboard, wlr_seat_set_name,
                  wlr_seat_touch_end_grab, wlr_seat_touch_get_point, wlr_seat_touch_has_grab,
                  wlr_seat_touch_notify_down, wlr_seat_touch_notify_motion,
                  wlr_seat_touch_notify_up, wlr_seat_touch_num_points,
                  wlr_seat_touch_point_clear_focus, wlr_seat_touch_point_focus,
                  wlr_seat_touch_send_down, wlr_seat_touch_send_motion, wlr_seat_touch_send_up,
                  wlr_seat_touch_start_grab};
use wlroots_sys::wayland_server::protocol::wl_seat::Capability;

use compositor::Compositor;
use utils::{c_to_rust_string, safe_as_cstring};

use super::grab::{KeyboardGrab, PointerGrab, TouchGrab};
use super::touch_point::{TouchId, TouchPoint};
use types::input_device::InputDevice;
use types::surface::Surface;
use utils::ToMS;

use KeyboardModifiers;

/// A wrapper around `wlr_seat`.
pub struct Seat {
    seat: *mut wlr_seat
}

impl Seat {
    /// Allocates a new `wlr_seat` and adds a wl_seat global to the display.
    pub fn create(compositor: &mut Compositor, name: String) -> Option<Self> {
        unsafe {
            let name = safe_as_cstring(name);
            let seat = wlr_seat_create(compositor.display() as _, name.as_ptr());
            if seat.is_null() {
                None
            } else {
                Some(Seat { seat })
            }
        }
    }

    /// Get the name of the seat.
    pub fn name(&self) -> Option<String> {
        unsafe {
            let name_ptr = (*self.seat).name;
            if name_ptr.is_null() {
                return None
            }
            c_to_rust_string(name_ptr)
        }
    }

    /// Updates the name of this seat.
    /// Will automatically send it to all clients.
    pub fn set_name(&mut self, name: String) {
        let name = safe_as_cstring(name);
        unsafe {
            wlr_seat_set_name(self.seat, name.as_ptr());
        }
    }

    /// Gets the capabilities of this seat.
    pub fn capabilities(&self) -> Capability {
        unsafe {
            Capability::from_raw((*self.seat).capabilities).expect("Invalid capabilities")
        }
    }

    /// Updates the capabilities available on this seat.
    /// Will automatically send it to all clients.
    pub fn set_capabilities(&mut self, capabilities: Capability) {
        unsafe { wlr_seat_set_capabilities(self.seat, capabilities.bits()) }
    }

    /// Determines if the surface has pointer focus.
    pub fn pointer_surface_has_focus(&mut self, surface: Surface) -> bool {
        unsafe { wlr_seat_pointer_surface_has_focus(self.seat, surface.as_ptr()) }
    }

    // Sends a pointer enter event to the given surface and considers it to be
    // the focused surface for the pointer.
    //
    // This will send a leave event to the last surface that was entered.
    //
    // Coordinates for the enter event are surface-local.
    //
    // Compositor should use `Seat::pointer_notify_enter` to
    // change pointer focus to respect pointer grabs.
    pub fn pointer_enter(&mut self, surface: Surface, sx: f64, sy: f64) {
        unsafe {
            wlr_seat_pointer_enter(self.seat, surface.as_ptr(), sx, sy);
        }
    }

    /// Clears the focused surface for the pointer and leaves all entered
    /// surfaces.
    pub fn clear_focus(&mut self) {
        unsafe { wlr_seat_pointer_clear_focus(self.seat) }
    }

    /// Sends a motion event to the surface with pointer focus.
    ///
    /// Coordinates for the motion event are surface-local.
    ///
    /// Compositors should use `Seat::notify_motion` to
    /// send motion events to the respect pointer grabs.
    pub fn send_motion(&mut self, time: Duration, sx: f64, sy: f64) {
        unsafe { wlr_seat_pointer_send_motion(self.seat, time.to_ms(), sx, sy) }
    }

    // TODO Button and State should probably be wrapped in some sort of type...

    /// Send a button event to the surface with pointer focus.
    ///
    /// Coordinates for the button event are surface-local.
    ///
    /// Returns the serial.
    ///
    /// Compositors should use `Seat::notify_button` to
    /// send button events to respect pointer grabs.
    pub fn send_button(&mut self, time: Duration, button: u32, state: u32) -> u32 {
        unsafe { wlr_seat_pointer_send_button(self.seat, time.to_ms(), button, state) }
    }

    /// Send an axis event to the surface with pointer focus.
    ///
    /// Compositors should use `Seat::notify_axis` to
    /// send axis events to respect pointer grabs.
    pub fn send_axis(&mut self, time: Duration, orientation: wlr_axis_orientation, value: f64) {
        unsafe {
            wlr_seat_pointer_send_axis(self.seat, time.to_ms(), orientation, value);
        }
    }

    /// Start a grab of the pointer of this seat. The grabber is responsible for
    /// handling all pointer events until the grab ends.
    pub fn pointer_start_grab(&mut self, grab: PointerGrab) {
        unsafe { wlr_seat_pointer_start_grab(self.seat, grab.as_ptr()) }
    }

    /// End the grab of the pointer of this seat. This reverts the grab back to the
    /// default grab for the pointer.
    pub fn pointer_end_grab(&mut self) {
        unsafe { wlr_seat_pointer_end_grab(self.seat) }
    }

    /// Whether or not the pointer has a grab other than the default grab.
    pub fn pointer_has_grab(&self) -> bool {
        unsafe { wlr_seat_pointer_has_grab(self.seat) }
    }

    /// Clear the focused surface for the pointer and leave all entered
    /// surfaces.
    pub fn pointer_clear_focus(&mut self) {
        unsafe { wlr_seat_pointer_clear_focus(self.seat) }
    }

    /// Notify the seat of a pointer enter event to the given surface and request it
    /// to be the focused surface for the pointer.
    ///
    /// Pass surface-local coordinates where the enter occurred.
    pub fn pointer_notify_enter(&mut self, surface: Surface, sx: f64, sy: f64) {
        unsafe { wlr_seat_pointer_notify_enter(self.seat, surface.as_ptr(), sx, sy) }
    }

    /// Notify the seat of motion over the given surface.
    ///
    /// Pass surface-local coordinates where the pointer motion occurred.
    pub fn pointer_notify_motion(&mut self, time: Duration, sx: f64, sy: f64) {
        unsafe { wlr_seat_pointer_notify_motion(self.seat, time.to_ms(), sx, sy) }
    }

    // TODO Wrapper type around Button and State

    /// Notify the seat that a button has been pressed.
    ///
    /// Returns the serial of the button press or zero if no button press was sent.
    pub fn pointer_notify_button(&mut self, time: Duration, button: u32, state: u32) -> u32 {
        unsafe { wlr_seat_pointer_notify_button(self.seat, time.to_ms(), button, state) }
    }

    /// Notify the seat of an axis event.
    pub fn pointer_notify_axis(&mut self,
                               time: Duration,
                               orientation: wlr_axis_orientation,
                               value: f64) {
        unsafe { wlr_seat_pointer_notify_axis(self.seat, time.to_ms(), orientation, value) }
    }

    /// Set this keyboard as the active keyboard for the seat.
    pub fn set_keyboard(&mut self, dev: InputDevice) {
        unsafe { wlr_seat_set_keyboard(self.seat, dev.as_ptr()) }
    }

    // TODO Point to the correct function name in this documentation.

    /// Send the keyboard key to focused keyboard resources.
    ///
    /// Compositors should use `wlr_seat_notify_key()` to respect keyboard grabs.
    pub fn keyboard_send_key(&mut self, time: Duration, key: u32, state: u32) {
        unsafe { wlr_seat_keyboard_send_key(self.seat, time.to_ms(), key, state) }
    }

    /// Send the modifier state to focused keyboard resources.
    ///
    /// Compositors should use `Seat::keyboard_notify_modifiers()` to respect any keyboard grabs.
    pub fn keyboard_send_modifiers(&mut self, modifiers: &mut KeyboardModifiers) {
        unsafe { wlr_seat_keyboard_send_modifiers(self.seat, modifiers) }
    }

    /// Send a keyboard enter event to the given surface and consider it to be the
    /// focused surface for the keyboard.
    ///
    /// This will send a leave event to the last surface that was entered.
    ///
    /// Compositors should use `Seat::keyboard_notify_enter()` to
    /// change keyboard focus to respect keyboard grabs.
    pub fn keyboard_enter(&mut self,
                          surface: Surface,
                          keycodes: &mut [Keycode],
                          modifiers: &mut KeyboardModifiers) {
        let keycodes_length = keycodes.len();
        unsafe {
            wlr_seat_keyboard_enter(self.seat,
                                    surface.as_ptr(),
                                    keycodes.as_mut_ptr(),
                                    keycodes_length,
                                    modifiers)
        }
    }

    /// Start a grab of the keyboard of this seat. The grabber is responsible for
    /// handling all keyboard events until the grab ends.
    pub fn keyboard_start_grab(&mut self, grab: KeyboardGrab) {
        unsafe { wlr_seat_keyboard_start_grab(self.seat, grab.as_ptr()) }
    }

    /// End the grab of the keyboard of this seat. This reverts the grab back to the
    /// default grab for the keyboard.
    pub fn keyboard_end_grab(&mut self) {
        unsafe { wlr_seat_keyboard_end_grab(self.seat) }
    }

    /// Whether or not the keyboard has a grab other than the default grab
    pub fn keyboard_has_grab(&self) -> bool {
        unsafe { wlr_seat_keyboard_has_grab(self.seat) }
    }

    /// Clear the focused surface for the keyboard and leave all entered
    /// surfaces.
    pub fn keyboard_clear_focus(&mut self) {
        unsafe { wlr_seat_keyboard_clear_focus(self.seat) }
    }

    /// Notify the seat that the modifiers for the keyboard have changed.
    ///
    /// Defers to any keyboard grabs.
    pub fn keyboard_notify_modifiers(&mut self, modifiers: &mut KeyboardModifiers) {
        unsafe { wlr_seat_keyboard_notify_modifiers(self.seat, modifiers) }
    }

    /// Notify the seat that the keyboard focus has changed and request it to be the
    /// focused surface for this keyboard.
    ///
    /// Defers to any current grab of the seat's keyboard.
    pub fn keyboard_notify_enter(&mut self,
                                 surface: Surface,
                                 keycodes: &mut [Keycode],
                                 modifiers: &mut KeyboardModifiers) {
        let keycodes_length = keycodes.len();
        unsafe {
            wlr_seat_keyboard_notify_enter(self.seat,
                                           surface.as_ptr(),
                                           keycodes.as_mut_ptr(),
                                           keycodes_length,
                                           modifiers)
        }
    }

    // TODO Wrapper type for Key and State

    /// Notify the seat that a key has been pressed on the keyboard.
    ///
    /// Defers to any keyboard grabs.
    pub fn keyboard_notify_key(&mut self, time: Duration, key: u32, state: u32) {
        unsafe { wlr_seat_keyboard_notify_key(self.seat, time.to_ms(), key, state) }
    }

    /// How many touch ponits are currently down for the seat.
    pub fn touch_num_points(&mut self) -> i32 {
        unsafe { wlr_seat_touch_num_points(self.seat) }
    }

    /// Start a grab of the touch device of this seat. The grabber is responsible for
    /// handling all touch events until the grab ends.
    pub fn touch_start_grab(&mut self, grab: TouchGrab) {
        unsafe { wlr_seat_touch_start_grab(self.seat, grab.as_ptr()) }
    }

    /// End the grab of the touch device of this seat. This reverts the grab back to
    /// the default grab for the touch device.
    pub fn touch_end_grab(&mut self) {
        unsafe { wlr_seat_touch_end_grab(self.seat) }
    }

    /// Whether or not the seat has a touch grab other than the default grab.
    pub fn touch_has_grab(&self) -> bool {
        unsafe { wlr_seat_touch_has_grab(self.seat) }
    }

    // Get the active touch point with the given `touch_id`. If the touch point does
    // not exist or is no longer active, returns None.
    pub fn get_touch_point(&self, touch_id: TouchId) -> Option<TouchPoint> {
        unsafe {
            let touch_point = wlr_seat_touch_get_point(self.seat, touch_id.into());
            if touch_point.is_null() {
                return None
            } else {
                Some(TouchPoint::from_ptr(touch_point))
            }
        }
    }

    /// Notify the seat that the touch point given by `touch_id` has entered a new
    /// surface.
    ///
    /// The surface is required. To clear focus, use `Seat::touch_point_clear_focus()`.
    pub fn touch_point_focus(&mut self,
                             surface: Surface,
                             time: Duration,
                             touch_id: TouchId,
                             sx: f64,
                             sy: f64) {
        unsafe {
            wlr_seat_touch_point_focus(self.seat,
                                       surface.as_ptr(),
                                       time.to_ms(),
                                       touch_id.into(),
                                       sx,
                                       sy)
        }
    }

    //// Clear the focused surface for the touch point given by `touch_id`.
    pub fn touch_point_clear_focus(&mut self, time: Duration, touch_id: TouchId) {
        unsafe { wlr_seat_touch_point_clear_focus(self.seat, time.to_ms(), touch_id.into()) }
    }

    /// Send a touch down event to the client of the given surface.
    ///
    /// All future touch events for this point will go to this surface.
    ///
    /// If the touch down is valid, this will add a new touch point with the given `touch_id`.
    ///
    /// The touch down may not be valid if the surface seat client does not accept touch input.
    ///
    /// Coordinates are surface-local.
    ///
    /// Compositors should use `Seat::touch_notify_down()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_down(&mut self,
                           surface: Surface,
                           time: Duration,
                           touch_id: TouchId,
                           sx: f64,
                           sy: f64)
                           -> u32 {
        unsafe {
            wlr_seat_touch_send_down(self.seat,
                                     surface.as_ptr(),
                                     time.to_ms(),
                                     touch_id.into(),
                                     sx,
                                     sy)
        }
    }

    /// Send a touch up event for the touch point given by the `touch_id`.
    ///
    /// The event will go to the client for the surface given in the cooresponding touch down
    /// event.
    ///
    /// This will remove the touch point.
    ///
    /// Compositors should use `Seat::touch_notify_up()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_up(&mut self, time: Duration, touch_id: TouchId) {
        unsafe { wlr_seat_touch_send_up(self.seat, time.to_ms(), touch_id.into()) }
    }

    /// Send a touch motion event for the touch point given by the `touch_id`.
    ///
    /// The event will go to the client for the surface given in the corresponding touch
    /// down event.
    ///
    /// Compositors should use `Seat::touch_notify_motion()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_motion(&mut self, time: Duration, touch_id: TouchId, sx: f64, sy: f64) {
        unsafe {
            wlr_seat_touch_send_motion(self.seat, time.to_ms(), touch_id.into(), sx, sy)
        }
    }

    // TODO Should this be returning a u32? Should I wrap whatever that number is?

    /// Notify the seat of a touch down on the given surface. Defers to any grab of
    /// the touch device.
    pub fn touch_notify_down(&mut self,
                             surface: Surface,
                             time: Duration,
                             touch_id: TouchId,
                             sx: f64,
                             sy: f64)
                             -> u32 {
        unsafe {
            wlr_seat_touch_notify_down(self.seat,
                                       surface.as_ptr(),
                                       time.to_ms(),
                                       touch_id.into(),
                                       sx,
                                       sy)
        }
    }

    /// Notify the seat that the touch point given by `touch_id` is up. Defers to any
    /// grab of the touch device.
    pub fn touch_notify_up(&mut self, time: Duration, touch_id: TouchId) {
        unsafe { wlr_seat_touch_notify_up(self.seat, time.to_ms(), touch_id.into()) }
    }

    /// Notify the seat that the touch point given by `touch_id` has moved.
    ///
    /// Defers to any grab of the touch device.
    ///
    /// The seat should be notified of touch motion even if the surface is
    /// not the owner of the touch point for processing by grabs.
    pub fn touch_notify_motion(&mut self, time: Duration, touch_id: TouchId, sx: f64, sy: f64) {
        unsafe {
            wlr_seat_touch_notify_motion(self.seat, time.to_ms(), touch_id.into(), sx, sy)
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat {
        self.seat
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        unsafe { wlr_seat_destroy(self.seat) }
    }
}
