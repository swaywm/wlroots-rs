//! Wrapper for wlr_seat. For more information about what a seat is, please
//! consult the Wayland documentation ([libinput docs](https://wayland.freedesktop.org/libinput/doc/latest/seats.html), [wayland docs](https://wayland.freedesktop.org/docs/html/apa.html#protocol-spec-wl_seat))
//!
//! TODO This module could really use some examples, as the API surface is huge.

use std::cell::RefCell;
use std::time::Duration;

use xkbcommon::xkb::Keycode;
use libc;
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

use compositor::COMPOSITOR_PTR;
use utils::{c_to_rust_string, safe_as_cstring};
use utils::ToMS;
use {Compositor, InputDevice, KeyboardGrab, KeyboardModifiers, PointerGrab, Surface, TouchGrab,
     TouchId, TouchPoint};

pub trait SeatHandler {
    /// Callback triggered when a client has grabbed a pointer.
    fn pointer_grabbed(&mut self, &mut Compositor, &Seat, &mut PointerGrab) {}

    /// Callback triggered when a client has ended a pointer grab.
    fn pointer_released(&mut self, &mut Compositor, &Seat, &mut PointerGrab) {}

    /// Callback triggered when a client has grabbed a keyboard.
    fn keyboard_grabbed(&mut self, &mut Compositor, &Seat, &mut KeyboardGrab) {}

    /// Callback triggered when a client has ended a keyboard grab.
    fn keyboard_released(&mut self, &mut Compositor, &Seat, &mut KeyboardGrab) {}

    /// Callback triggered when a client has grabbed a touch.
    fn touch_grabbed(&mut self, &mut Compositor, &Seat, &mut TouchGrab) {}

    /// Callback triggered when a client has ended a touch grab.
    fn touch_released(&mut self, &mut Compositor, &Seat, &mut TouchGrab) {}

    /* TODO FIXME wlr_seat_pointer_request_set_cursor_event */
    fn cursor_set(&mut self, &mut Compositor, &Seat) {}

    /// The seat was provided with a selection by the client.
    fn received_selection(&mut self, &mut Compositor, &Seat) {}

    /// The seat was provided with a selection from the primary buffer
    /// by the client.
    fn primary_selection(&mut self, &mut Compositor, &Seat) {}

    /// The seat is being destroyed.
    fn destroy(&mut self, &mut Compositor, &Seat) {}
}

/// The structure that contains all actual seat pointers.
///
/// This is here so that we can ensure unique access.
pub struct SeatInner {
    handler: Box<SeatHandler>,
    seat: *mut wlr_seat,
}

wayland_listener!(Seat, RefCell<SeatInner>, [
    pointer_grab_begin_listener => pointer_grab_begin_notify: |this: &mut Seat,
                                                               event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        let pointer_grab = &mut *(event as *mut PointerGrab);
        inner.handler.pointer_grabbed(compositor, &*this, pointer_grab);
    };
    pointer_grab_end_listener => pointer_grab_end_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        let pointer_grab = &mut *(event as *mut PointerGrab);
        inner.handler.pointer_released(compositor, &*this, pointer_grab);
    };
    keyboard_grab_begin_listener => keyboard_grab_begin_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        let keyboard_grab = &mut *(event as *mut KeyboardGrab);
        inner.handler.keyboard_grabbed(compositor, &*this, keyboard_grab);
    };
    keyboard_grab_end_listener => keyboard_grab_end_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        let keyboard_grab = &mut *(event as *mut KeyboardGrab);
        inner.handler.keyboard_released(compositor, &*this, keyboard_grab);
    };
    touch_grab_begin_listener => touch_grab_begin_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        let touch_grab = &mut *(event as *mut TouchGrab);
        inner.handler.touch_grabbed(compositor, &*this, touch_grab);
    };
    touch_grab_end_listener => touch_grab_end_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        let touch_grab = &mut *(event as *mut TouchGrab);
        inner.handler.touch_released(compositor, &*this, touch_grab);
    };
    request_set_cursor_listener => request_set_cursor_notify: |this: &mut Seat,
    _event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        inner.handler.cursor_set(compositor, &*this);
    };
    selection_listener => selection_notify: |this: &mut Seat, _event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        inner.handler.received_selection(compositor, &*this);
    };
    primary_selection_listener => primary_selection_notify: |this: &mut Seat,
    _event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        inner.handler.primary_selection(compositor, &*this);
    };
    destroy_listener => destroy_notify: |this: &mut Seat, _event: *mut libc::c_void,|
    unsafe {
        let mut inner = this.data.borrow_mut();
        let compositor = &mut *COMPOSITOR_PTR;
        inner.handler.destroy(compositor, &*this);
    };
]);

impl Seat {
    /// Allocates a new `wlr_seat` and adds a wl_seat global to the display.
    ///
    /// Puts the seat in a `RefCell` so that it's safe to use both in your
    /// state wherever and in the callback provided by the handler.
    ///
    /// Puts the seat in an `Rc` so that the address is static for internal
    /// purposes.
    pub fn create(compositor: &mut Compositor,
               name: String,
               handler: Box<SeatHandler>)
               -> Option<Box<Self>> {
        unsafe {
            let name = safe_as_cstring(name);
            let seat = wlr_seat_create(compositor.display() as _, name.as_ptr());
            if seat.is_null() {
                None
            } else {
                Some(Seat::new(RefCell::new(SeatInner { seat, handler })))
            }
        }
    }

    /// Get the name of the seat.
    pub fn name(&self) -> Option<String> {
        let seat = self.data.borrow();
        unsafe {
            let name_ptr = (*seat.seat).name;
            if name_ptr.is_null() {
                return None
            }
            c_to_rust_string(name_ptr)
        }
    }

    /// Updates the name of this seat.
    /// Will automatically send it to all clients.
    pub fn set_name(&self, name: String) {
        let seat = self.data.borrow();
        let name = safe_as_cstring(name);
        unsafe {
            wlr_seat_set_name(seat.seat, name.as_ptr());
        }
    }

    /// Gets the capabilities of this seat.
    pub fn capabilities(&self) -> Capability {
        let seat = self.data.borrow();
        unsafe { Capability::from_raw((*seat.seat).capabilities).expect("Invalid capabilities") }
    }

    /// Updates the capabilities available on this seat.
    /// Will automatically send it to all clients.
    pub fn set_capabilities(&self, capabilities: Capability) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_set_capabilities(seat.seat, capabilities.bits()) }
    }

    /// Determines if the surface has pointer focus.
    pub fn pointer_surface_has_focus(&self, surface: Surface) -> bool {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_surface_has_focus(seat.seat, surface.as_ptr()) }
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
    pub fn pointer_enter(&self, surface: Surface, sx: f64, sy: f64) {
        let seat = self.data.borrow();
        unsafe {
            wlr_seat_pointer_enter(seat.seat, surface.as_ptr(), sx, sy);
        }
    }

    /// Clears the focused surface for the pointer and leaves all entered
    /// surfaces.
    pub fn clear_focus(&self) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_clear_focus(seat.seat) }
    }

    /// Sends a motion event to the surface with pointer focus.
    ///
    /// Coordinates for the motion event are surface-local.
    ///
    /// Compositors should use `Seat::notify_motion` to
    /// send motion events to the respect pointer grabs.
    pub fn send_motion(&self, time: Duration, sx: f64, sy: f64) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_send_motion(seat.seat, time.to_ms(), sx, sy) }
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
    pub fn send_button(&self, time: Duration, button: u32, state: u32) -> u32 {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_send_button(seat.seat, time.to_ms(), button, state) }
    }

    /// Send an axis event to the surface with pointer focus.
    ///
    /// Compositors should use `Seat::notify_axis` to
    /// send axis events to respect pointer grabs.
    pub fn send_axis(&self, time: Duration, orientation: wlr_axis_orientation, value: f64) {
        let seat = self.data.borrow();
        unsafe {
            wlr_seat_pointer_send_axis(seat.seat, time.to_ms(), orientation, value);
        }
    }

    /// Start a grab of the pointer of this seat. The grabber is responsible for
    /// handling all pointer events until the grab ends.
    pub fn pointer_start_grab(&self, grab: PointerGrab) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_start_grab(seat.seat, grab.as_ptr()) }
    }

    /// End the grab of the pointer of this seat. This reverts the grab back to the
    /// default grab for the pointer.
    pub fn pointer_end_grab(&self) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_end_grab(seat.seat) }
    }

    /// Whether or not the pointer has a grab other than the default grab.
    pub fn pointer_has_grab(&self) -> bool {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_has_grab(seat.seat) }
    }

    /// Clear the focused surface for the pointer and leave all entered
    /// surfaces.
    pub fn pointer_clear_focus(&self) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_clear_focus(seat.seat) }
    }

    /// Notify the seat of a pointer enter event to the given surface and request it
    /// to be the focused surface for the pointer.
    ///
    /// Pass surface-local coordinates where the enter occurred.
    pub fn pointer_notify_enter(&self, surface: Surface, sx: f64, sy: f64) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_notify_enter(seat.seat, surface.as_ptr(), sx, sy) }
    }

    /// Notify the seat of motion over the given surface.
    ///
    /// Pass surface-local coordinates where the pointer motion occurred.
    pub fn pointer_notify_motion(&self, time: Duration, sx: f64, sy: f64) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_notify_motion(seat.seat, time.to_ms(), sx, sy) }
    }

    // TODO Wrapper type around Button and State

    /// Notify the seat that a button has been pressed.
    ///
    /// Returns the serial of the button press or zero if no button press was sent.
    pub fn pointer_notify_button(&self, time: Duration, button: u32, state: u32) -> u32 {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_notify_button(seat.seat, time.to_ms(), button, state) }
    }

    /// Notify the seat of an axis event.
    pub fn pointer_notify_axis(&self,
                               time: Duration,
                               orientation: wlr_axis_orientation,
                               value: f64) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_pointer_notify_axis(seat.seat, time.to_ms(), orientation, value) }
    }

    /// Set this keyboard as the active keyboard for the seat.
    pub fn set_keyboard(&self, dev: InputDevice) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_set_keyboard(seat.seat, dev.as_ptr()) }
    }

    // TODO Point to the correct function name in this documentation.

    /// Send the keyboard key to focused keyboard resources.
    ///
    /// Compositors should use `wlr_seat_notify_key()` to respect keyboard grabs.
    pub fn keyboard_send_key(&self, time: Duration, key: u32, state: u32) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_send_key(seat.seat, time.to_ms(), key, state) }
    }

    /// Send the modifier state to focused keyboard resources.
    ///
    /// Compositors should use `Seat::keyboard_notify_modifiers()` to respect any keyboard grabs.
    pub fn keyboard_send_modifiers(&self, modifiers: &mut KeyboardModifiers) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_send_modifiers(seat.seat, modifiers) }
    }

    /// Send a keyboard enter event to the given surface and consider it to be the
    /// focused surface for the keyboard.
    ///
    /// This will send a leave event to the last surface that was entered.
    ///
    /// Compositors should use `Seat::keyboard_notify_enter()` to
    /// change keyboard focus to respect keyboard grabs.
    pub fn keyboard_enter(&self,
                          surface: Surface,
                          keycodes: &mut [Keycode],
                          modifiers: &mut KeyboardModifiers) {
        let seat = self.data.borrow();
        let keycodes_length = keycodes.len();
        unsafe {
            wlr_seat_keyboard_enter(seat.seat,
                                    surface.as_ptr(),
                                    keycodes.as_mut_ptr(),
                                    keycodes_length,
                                    modifiers)
        }
    }

    /// Start a grab of the keyboard of this seat. The grabber is responsible for
    /// handling all keyboard events until the grab ends.
    pub fn keyboard_start_grab(&self, grab: KeyboardGrab) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_start_grab(seat.seat, grab.as_ptr()) }
    }

    /// End the grab of the keyboard of this seat. This reverts the grab back to the
    /// default grab for the keyboard.
    pub fn keyboard_end_grab(&self) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_end_grab(seat.seat) }
    }

    /// Whether or not the keyboard has a grab other than the default grab
    pub fn keyboard_has_grab(&self) -> bool {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_has_grab(seat.seat) }
    }

    /// Clear the focused surface for the keyboard and leave all entered
    /// surfaces.
    pub fn keyboard_clear_focus(&self) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_clear_focus(seat.seat) }
    }

    /// Notify the seat that the modifiers for the keyboard have changed.
    ///
    /// Defers to any keyboard grabs.
    pub fn keyboard_notify_modifiers(&self, modifiers: &mut KeyboardModifiers) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_notify_modifiers(seat.seat, modifiers) }
    }

    /// Notify the seat that the keyboard focus has changed and request it to be the
    /// focused surface for this keyboard.
    ///
    /// Defers to any current grab of the seat's keyboard.
    pub fn keyboard_notify_enter(&self,
                                 surface: Surface,
                                 keycodes: &mut [Keycode],
                                 modifiers: &mut KeyboardModifiers) {
        let seat = self.data.borrow();
        let keycodes_length = keycodes.len();
        unsafe {
            wlr_seat_keyboard_notify_enter(seat.seat,
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
    pub fn keyboard_notify_key(&self, time: Duration, key: u32, state: u32) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_keyboard_notify_key(seat.seat, time.to_ms(), key, state) }
    }

    /// How many touch ponits are currently down for the seat.
    pub fn touch_num_points(&self) -> i32 {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_num_points(seat.seat) }
    }

    /// Start a grab of the touch device of this seat. The grabber is responsible for
    /// handling all touch events until the grab ends.
    pub fn touch_start_grab(&self, grab: TouchGrab) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_start_grab(seat.seat, grab.as_ptr()) }
    }

    /// End the grab of the touch device of this seat. This reverts the grab back to
    /// the default grab for the touch device.
    pub fn touch_end_grab(&self) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_end_grab(seat.seat) }
    }

    /// Whether or not the seat has a touch grab other than the default grab.
    pub fn touch_has_grab(&self) -> bool {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_has_grab(seat.seat) }
    }

    // Get the active touch point with the given `touch_id`. If the touch point does
    // not exist or is no longer active, returns None.
    pub fn get_touch_point(&self, touch_id: TouchId) -> Option<TouchPoint> {
        let seat = self.data.borrow();
        unsafe {
            let touch_point = wlr_seat_touch_get_point(seat.seat, touch_id.into());
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
    pub fn touch_point_focus(&self,
                             surface: Surface,
                             time: Duration,
                             touch_id: TouchId,
                             sx: f64,
                             sy: f64) {
        let seat = self.data.borrow();
        unsafe {
            wlr_seat_touch_point_focus(seat.seat,
                                       surface.as_ptr(),
                                       time.to_ms(),
                                       touch_id.into(),
                                       sx,
                                       sy)
        }
    }

    //// Clear the focused surface for the touch point given by `touch_id`.
    pub fn touch_point_clear_focus(&self, time: Duration, touch_id: TouchId) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_point_clear_focus(seat.seat, time.to_ms(), touch_id.into()) }
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
    pub fn touch_send_down(&self,
                           surface: Surface,
                           time: Duration,
                           touch_id: TouchId,
                           sx: f64,
                           sy: f64)
                           -> u32 {
        let seat = self.data.borrow();
        unsafe {
            wlr_seat_touch_send_down(seat.seat,
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
    pub fn touch_send_up(&self, time: Duration, touch_id: TouchId) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_send_up(seat.seat, time.to_ms(), touch_id.into()) }
    }

    /// Send a touch motion event for the touch point given by the `touch_id`.
    ///
    /// The event will go to the client for the surface given in the corresponding touch
    /// down event.
    ///
    /// Compositors should use `Seat::touch_notify_motion()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_motion(&self, time: Duration, touch_id: TouchId, sx: f64, sy: f64) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_send_motion(seat.seat, time.to_ms(), touch_id.into(), sx, sy) }
    }

    // TODO Should this be returning a u32? Should I wrap whatever that number is?

    /// Notify the seat of a touch down on the given surface. Defers to any grab of
    /// the touch device.
    pub fn touch_notify_down(&self,
                             surface: Surface,
                             time: Duration,
                             touch_id: TouchId,
                             sx: f64,
                             sy: f64)
                             -> u32 {
        let seat = self.data.borrow();
        unsafe {
            wlr_seat_touch_notify_down(seat.seat,
                                       surface.as_ptr(),
                                       time.to_ms(),
                                       touch_id.into(),
                                       sx,
                                       sy)
        }
    }

    /// Notify the seat that the touch point given by `touch_id` is up. Defers to any
    /// grab of the touch device.
    pub fn touch_notify_up(&self, time: Duration, touch_id: TouchId) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_notify_up(seat.seat, time.to_ms(), touch_id.into()) }
    }

    /// Notify the seat that the touch point given by `touch_id` has moved.
    ///
    /// Defers to any grab of the touch device.
    ///
    /// The seat should be notified of touch motion even if the surface is
    /// not the owner of the touch point for processing by grabs.
    pub fn touch_notify_motion(&self, time: Duration, touch_id: TouchId, sx: f64, sy: f64) {
        let seat = self.data.borrow();
        unsafe { wlr_seat_touch_notify_motion(seat.seat, time.to_ms(), touch_id.into(), sx, sy) }
    }

    pub(crate) unsafe fn as_ptr(&mut self) -> *mut wlr_seat {
        let inner = self.data.get_mut();
        inner.seat
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        let inner = self.data.get_mut();
        unsafe { wlr_seat_destroy(inner.seat) }
    }
}
