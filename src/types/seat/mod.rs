mod seat_client;
mod seat;
mod grab;
mod touch_point;

pub use self::grab::*;
pub use self::seat::*;
pub use self::seat_client::*;
pub use self::touch_point::*;

pub use seat::Seat;
use std::collections::HashMap;
use wlroots_sys::wlr_seat;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SeatId(*mut wlr_seat);

impl SeatId {
    pub(crate) fn new(ptr: *mut wlr_seat) -> Self {
        SeatId(ptr)
    }
}

/// A wrapper around the mapping of id -> `Seat`.
///
/// This is to ensure you can't move the Seat out of the box,
/// or do other weird things to it.
///
/// To add a `Seat` to this, please use `Seat::create`
#[derive(Debug, Default)]
pub struct Seats(HashMap<SeatId, Box<Seat>>);

impl Seats {
    /// Gets a mutable reference to a seat by id.
    ///
    /// To add a Seat, please use `Seat::create`.
    ///
    /// A seat cannot be accessed while it is in a callback. To use it,
    /// you should instead use the Seat value that's passed in the callback.
    ///
    /// Returns `None` if the seat has been removed or the name is incorrect.
    pub fn get(&mut self, id: SeatId) -> Option<&mut Box<Seat>> {
        self.0.get_mut(&id)
    }

    /// Add a new seat to the mapping.
    pub(crate) fn insert(&mut self, seat: Box<Seat>) -> &mut Box<Seat> {
        let id = unsafe { SeatId::new(seat.as_ptr()) };
        self.0.insert(id, seat);
        self.0.get_mut(&id).unwrap()
    }

    /// Take the seat from the mapping.
    ///
    /// This is either done to destroy it (in the destroy callback)
    /// or to borrow it uniquely for a time (e.g in all other Seat callbacks).
    ///
    /// If the seat does not exist, then `None` is returned.
    pub(crate) fn remove(&mut self, id: SeatId) -> Option<Box<Seat>> {
        self.0.remove(&id)
    }
}
