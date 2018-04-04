use super::cursor;
use wlroots;

#[derive(Debug)]
pub struct Seat {
    pub seat: wlroots::SeatHandle,
    pub cursor: Option<cursor::Cursor>
}

#[derive(Debug)]
pub struct SeatHandler {
    pub cursor: Option<cursor::Cursor>
}

impl SeatHandler {
    pub fn new() -> Self {
        SeatHandler { cursor: None }
    }
}

impl wlroots::SeatHandler for SeatHandler {
    // TODO
}


impl Seat {
    pub fn new(seat: wlroots::SeatHandle) -> Self {
        Seat { seat, cursor: None }
    }
}
