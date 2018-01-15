//! Wrapper for wlr_seat_client, a manager for handling seats to an individual
//! client.
//!
//! This struct is very unsafe, and probably will not be used directly by the
//! compositor author. Instead, this is used internally by various wlr seat
//! state structs (e.g `wlr_seat_keyboard_state`, `wlr_seat_pointer_state`)

use std::marker::PhantomData;

use super::seat::Seat;

use wlroots_sys::{wl_client, wlr_seat_client, wlr_seat_client_for_wl_client};

/// Contains state for a single client's bound wl_seat resource.
/// It can be used to issue input events to the client.
///
/// The lifetime of this object is managed by `Seat`.
pub struct SeatClient<'wlr_seat> {
    client: *mut wlr_seat_client,
    _phantom: PhantomData<&'wlr_seat Seat>
}

impl<'wlr_seat> SeatClient<'wlr_seat> {
    /// Gets a SeatClient for the specified client,
    /// if there is one bound for that client.
    ///
    /// # Unsafety
    /// Since this just is a wrapper for checking if the wlr_seat pointer matches
    /// the provided wl_client pointer, this function is unsafe.
    ///
    /// Please only pass a valid pointer to a wl_client to this function.
    pub unsafe fn client_for_wl_client(seat: &'wlr_seat mut Seat,
                                       client: *mut wl_client)
                                       -> Option<SeatClient<'wlr_seat>> {
        let client = wlr_seat_client_for_wl_client(seat.as_ptr(), client);
        if client.is_null() {
            None
        } else {
            Some(SeatClient { client,
                              _phantom: PhantomData })
        }
    }

    /// Recreates a `SeatClient` from a raw `wlr_seat_client`.
    ///
    /// # Unsafety
    /// The pointer must point to a valid `wlr_seat_client`.
    ///
    /// Note also that the struct has an *boundless lifetime*. You _must_ ensure
    /// this struct does not live longer than the `Seat` that manages it.
    pub unsafe fn from_ptr<'unbound_seat>(client: *mut wlr_seat_client)
                                          -> SeatClient<'unbound_seat> {
        SeatClient { client,
                     _phantom: PhantomData }
    }

    pub unsafe fn as_ptr(&self) -> *mut wlr_seat_client {
        self.client
    }
}
