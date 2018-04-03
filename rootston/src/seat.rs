use wlroots::SeatHandler;

pub struct Seat {}

impl Seat {
    pub fn new() -> Self {
        Seat {}
    }
}

impl SeatHandler for Seat {
    // TODO
}
