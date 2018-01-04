//! Wrapper for wlr_seat. For more information about what a seat is, please
//! consult the Wayland documentation ([libinput docs](https://wayland.freedesktop.org/libinput/doc/latest/seats.html), [wayland docs](https://wayland.freedesktop.org/docs/html/apa.html#protocol-spec-wl_seat))

use std::time::Duration;

use compositor::Compositor;
use utils::{c_to_rust_string, safe_as_cstring};
use wlroots_sys::{wlr_axis_orientation, wlr_seat, wlr_seat_create, wlr_seat_destroy,
                  wlr_seat_pointer_clear_focus, wlr_seat_pointer_send_axis,
                  wlr_seat_pointer_send_button, wlr_seat_pointer_send_motion,
                  wlr_seat_set_capabilities, wlr_seat_set_name};
use wlroots_sys::wayland_server::protocol::wl_seat::Capability;

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

    // TODO Need to wrap wlr_surface first
    //
    // // Determines if the surface has pointer focus.
    // pub fn pointer_surface_has_focus(&mut self, surface: Surface) -> bool {
    //    unsafe {
    //        wlr_seat_pointer_surface_has_focus(self.seat, surface.as_ptr())
    //    }
    // }
    //
    // // Sends a pointer enter event to the given surface and considers it to be
    // // the focused surface for the pointer.
    // //
    // // This will send a leave event to the last surface that was entered.
    // //
    // // Coordinates for the enter event are surface-local.
    // //
    // // Compositor should use `Seat::notify_enter` to
    // // change pointer focus to respect pointer grabs.
    // pub fn pointer_enter(&mut self, surface: Surface, sx: f64, sy: f64) {
    //    unsafe {
    //        wlr_seat_pointer_enter(surface.as_ptr(), sx, sy);
    //    }
    // }

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
        let seconds_delta = time.as_secs() as u32;
        let nano_delta = time.subsec_nanos();
        let ms = (seconds_delta * 1000) + nano_delta / 1000000;
        unsafe {
            // TODO FIXME what kind of time? ms? s?
            // I'm just guessing it's ms, there's no documentation on this.
            wlr_seat_pointer_send_motion(self.seat, ms, sx, sy)
        }
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
        let seconds_delta = time.as_secs() as u32;
        let nano_delta = time.subsec_nanos();
        let ms = (seconds_delta * 1000) + nano_delta / 1000000;
        unsafe { wlr_seat_pointer_send_button(self.seat, ms, button, state) }
    }

    /// Send an axis event to the surface with pointer focus.
    ///
    /// Compositors should use `Seat::notify_axis` to
    /// send axis events to respect pointer grabs.
    pub fn send_axis(&mut self, time: Duration, orientation: wlr_axis_orientation, value: f64) {
        let seconds_delta = time.as_secs() as u32;
        let nano_delta = time.subsec_nanos();
        let ms = (seconds_delta * 1000) + nano_delta / 1000000;
        unsafe {
            wlr_seat_pointer_send_axis(self.seat, ms, orientation, value);
        }
    }

    pub unsafe fn to_ptr(&self) -> *mut wlr_seat {
        self.seat
    }

    // TODO grab, notify, and some other specific input misc functions
}

impl Drop for Seat {
    fn drop(&mut self) {
        unsafe { wlr_seat_destroy(self.seat) }
    }
}
