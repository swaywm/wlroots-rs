use super::{seat, server::Server};
use wlroots::{self, Capability, Compositor, InputDevice, InputManagerHandler};

const DEFAULT_SEAT_NAME: &str = "seat0";

pub struct InputManager {
    seats: Vec<wlroots::SeatId>
}

pub struct Input {}

impl InputManagerHandler for InputManager {
    fn input_added(&mut self, compositor: &mut Compositor, input: &mut InputDevice) {
        // TODO Get configuration from compositor state, use that name instead
        let seat_name = DEFAULT_SEAT_NAME;
        let seat_id = self.get_seat(compositor, seat_name);
        let seat = compositor.seats.get(seat_id).expect("seat id was invalid");
        let server: &mut Server = compositor.data.downcast_mut().unwrap();
        // TODO Should we move this device specific setup to the device specific
        // callbacks?
        self.add_device_to_seat(server, seat, input);
    }
}

impl InputManager {
    pub fn new() -> Self {
        InputManager { seats: vec![] }
    }

    /// Gets the seat by name. If it doesn't exist yet, create it.
    fn get_seat(&self, compositor: &mut Compositor, seat_name: &str) -> wlroots::SeatId {
        for seat_id in &self.seats {
            if let Some(seat) = compositor.seats.get(*seat_id) {
                // TODO Avoid allocation
                if seat.name() == Some(seat_name.into()) {
                    return *seat_id
                }
            }
        }
        wlroots::Seat::create(compositor, seat_name.into(), Box::new(seat::Seat::new()))
                .expect("Could not create a seat").id()
    }

    pub fn add_device_to_seat(&mut self,
                              server: &mut Server,
                              seat: &mut Box<wlroots::Seat>,
                              input: &mut InputDevice) {
        use wlroots::wlr_input_device_type::*;
        match input.dev_type() {
            WLR_INPUT_DEVICE_KEYBOARD => seat.set_keyboard(input),
            WLR_INPUT_DEVICE_POINTER => {
                // TODO Need cursor from the output layout from the thingie
                // TODO Where is this cursor allocated though...?
                // double check seat construction
                //cursor.attach_input_device(input);
                self.configure_cursor(server, seat);
                let mut capabilities = seat.capabilities();
                capabilities.insert(Capability::Pointer);
                seat.set_capabilities(capabilities);
            }
            WLR_INPUT_DEVICE_TOUCH | WLR_INPUT_DEVICE_TABLET_TOOL => {
                // TODO Need cursor from the output layout from the thingie
                // TODO Where is this cursor allocated though...?
                // double check seat construction
                //cursor.attach_input_device(input);
                self.configure_cursor(server, seat);
                let mut capabilities = seat.capabilities();
                capabilities.insert(Capability::Touch);
                seat.set_capabilities(capabilities);
            }
            WLR_INPUT_DEVICE_TABLET_PAD => { /*TODO*/ }
        }
    }

    pub fn configure_cursor(&mut self, server: &mut Server, seat: &mut Box<wlroots::Seat>) {
        // reset mappings
        //state.layout.cursor(cursor_id)

        // configure device to output mappings
    }
}
