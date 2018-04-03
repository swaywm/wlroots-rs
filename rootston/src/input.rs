use super::seat;
use wlroots::{self, Capability, Compositor, InputDevice, InputManagerHandler};

const DEFAULT_SEAT_NAME: &str = "seat0";

pub struct InputManager {
    seats: Vec<wlroots::SeatHandle>
}

pub struct Input {}

impl InputManagerHandler for InputManager {
    fn input_added(&mut self, compositor: &mut Compositor, input: &mut InputDevice) {
        // TODO Get configuration from compositor state, use that name instead
        let seat_name = DEFAULT_SEAT_NAME;
        let seat = self.get_seat(compositor, seat_name);
        // TODO Should we move this device specific setup to the device specific
        // callbacks?
        self.add_device_to_seat(compositor, seat, input);
    }
}

impl InputManager {
    pub fn new() -> Self {
        InputManager { seats: vec![] }
    }

    /// Gets the seat by name. If it doesn't exist yet, create it.
    fn get_seat(&mut self, compositor: &mut Compositor, seat_name: &str) -> wlroots::SeatHandle {
        let found = &mut false;
        for seat in &self.seats {
            seat.clone().run(|mut seat| {
                         if seat.name().as_ref().map(|s| &**s) == Some(seat_name) {
                             *found = true;
                         }
                         Some(seat)
                     })
                .unwrap();
            if *found {
                return seat.clone()
            }
        }
        let seat = wlroots::Seat::create(compositor, seat_name.into(), Box::new(seat::Seat::new()));
        self.seats.push(seat);
        self.seats.last().unwrap().clone()
    }

    pub fn add_device_to_seat(&mut self,
                              compositor: &mut Compositor,
                              mut seat: wlroots::SeatHandle,
                              input: &mut InputDevice) {
        use wlroots::wlr_input_device_type::*;
        seat.run(|mut seat| {
                     match input.dev_type() {
                         WLR_INPUT_DEVICE_KEYBOARD => seat.set_keyboard(input),
                         WLR_INPUT_DEVICE_POINTER => {
                             // TODO Need cursor from the output layout from the thingie
                             // TODO Where is this cursor allocated though...?
                             // double check seat construction
                             //cursor.attach_input_device(input);
                             self.configure_cursor(compositor, &mut seat);
                             let mut capabilities = seat.capabilities();
                             capabilities.insert(Capability::Pointer);
                             seat.set_capabilities(capabilities);
                         }
                         WLR_INPUT_DEVICE_TOUCH | WLR_INPUT_DEVICE_TABLET_TOOL => {
                             // TODO Need cursor from the output layout from the thingie
                             // TODO Where is this cursor allocated though...?
                             // double check seat construction
                             //cursor.attach_input_device(input);
                             self.configure_cursor(compositor, &mut seat);
                             let mut capabilities = seat.capabilities();
                             capabilities.insert(Capability::Touch);
                             seat.set_capabilities(capabilities);
                         }
                         WLR_INPUT_DEVICE_TABLET_PAD => { /*TODO*/ }
                     }
                     Some(seat)
                 }).unwrap()
    }

    pub fn configure_cursor(&mut self, compositor: &mut Compositor, seat: &mut Box<wlroots::Seat>) {
        // reset mappings
        //state.layout.cursor(cursor_id)

        // configure device to output mappings
    }
}
