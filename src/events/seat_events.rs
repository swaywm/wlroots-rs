use wlroots_sys::wlr_seat_pointer_request_set_cursor_event;

use crate::{seat, surface};

#[derive(Debug)]
pub struct SetCursor {
    event: *mut wlr_seat_pointer_request_set_cursor_event
}

impl SetCursor {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_seat_pointer_request_set_cursor_event) -> Self {
        SetCursor { event }
    }
    /// Get the seat client associated with the seat where this
    /// event is occurring.
    pub fn seat_client<'seat>(&'seat self) -> seat::Client<'seat> {
        unsafe { seat::Client::from_ptr((*self.event).seat_client) }
    }

    /// Get the surface that is providing the cursor to the seat.
    pub fn surface(&self) -> Option<surface::Handle> {
        unsafe {
            let surface = (*self.event).surface;
            if surface.is_null() {
                None
            } else {
                Some(surface::Handle::from_ptr(surface))
            }
        }
    }

    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }

    pub fn location(&self) -> (i32, i32) {
        unsafe { ((*self.event).hotspot_x, (*self.event).hotspot_y) }
    }
}
