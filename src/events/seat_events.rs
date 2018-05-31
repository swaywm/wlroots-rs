use wlroots_sys::wlr_seat_pointer_request_set_cursor_event;

use {SeatClient, SurfaceHandle};

#[derive(Debug)]
pub struct SetCursorEvent {
    event: *mut wlr_seat_pointer_request_set_cursor_event
}

impl SetCursorEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_seat_pointer_request_set_cursor_event) -> Self {
        SetCursorEvent { event }
    }
    /// Get the seat client associated with the seat where this
    /// event is occurring.
    pub fn seat_client<'seat>(&'seat self) -> SeatClient<'seat> {
        unsafe { SeatClient::from_ptr((*self.event).seat_client) }
    }

    /// Get the surface that is providing the cursor to the seat.
    pub fn surface(&self) -> Option<SurfaceHandle> {
        unsafe {
            let surface = (*self.event).surface;
            if surface.is_null() {
                None
            } else {
                SurfaceHandle::from_ptr(surface)
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
