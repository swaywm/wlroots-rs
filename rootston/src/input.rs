use super::seat;
use wlroots::{self, Capability, Compositor, InputDevice, InputManagerHandler};

const DEFAULT_SEAT_NAME: &str = "seat0";

pub struct InputManager {
    seats: Vec<seat::Seat>
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
        for seat in &mut self.seats {
            seat.seat.clone().run(|seat| {
                         if seat.name().as_ref().map(|s| &**s) == Some(seat_name) {
                             *found = true;
                         }
                         Some(seat)
                     })
                .unwrap();
            if *found {
                return seat.seat.clone()
            }
        }
        let seat_handle = wlroots::Seat::create(compositor,
                                         seat_name.into(),
                                         Box::new(seat::SeatHandler::new()));
        let seat = seat::Seat::new(seat_handle);
        self.seats.push(seat);
        self.seats.last_mut().unwrap().seat.clone()
    }

    pub fn add_device_to_seat(&mut self,
                              compositor: &mut Compositor,
                              mut seat: wlroots::SeatHandle,
                              input: &mut InputDevice) {
        use wlroots::wlr_input_device_type::*;
        match input.dev_type() {
            WLR_INPUT_DEVICE_KEYBOARD => seat.run(|mut seat| {
                seat.set_keyboard(input);
                Some(seat)
            }).unwrap(),
            WLR_INPUT_DEVICE_POINTER => {
                // TODO Need cursor from the output layout from the thingie
                // TODO Where is this cursor allocated though...?
                // double check seat construction
                //cursor.attach_input_device(input);
                self.configure_cursor(compositor, seat.clone());
                seat.run(|mut seat| {
                    let mut capabilities = seat.capabilities();
                    capabilities.insert(Capability::Pointer);
                    seat.set_capabilities(capabilities);
                    Some(seat)
                }).unwrap();
            }
            WLR_INPUT_DEVICE_TOUCH | WLR_INPUT_DEVICE_TABLET_TOOL => {
                // TODO Need cursor from the output layout from the thingie
                // TODO Where is this cursor allocated though...?
                // double check seat construction
                //cursor.attach_input_device(input);
                self.configure_cursor(compositor, seat.clone());
                seat.run(|mut seat| {
                    let mut capabilities = seat.capabilities();
                    capabilities.insert(Capability::Touch);
                    seat.set_capabilities(capabilities);
                    Some(seat)
                }).unwrap();
            }
            WLR_INPUT_DEVICE_TABLET_PAD => { /*TODO*/ }
        }
    }

    pub fn configure_cursor(&mut self, compositor: &mut Compositor, seat: wlroots::SeatHandle) {
        let roots_seat = self.roots_seat_from_handle(seat.clone());
        // reset mappings
        if let Some(cursor) = roots_seat.cursor.as_mut() {
            cursor.cursor.run(|mut cursor| {
                cursor.map_to_output(None);
                Some(cursor)
            }).unwrap();
        }
        //for pointer in
        //state.layout.cursor(cursor_id)

        // configure device to output mappings
    }

    fn roots_seat_from_handle(&mut self, mut handle: wlroots::SeatHandle) -> &mut seat::Seat {
        for seat in &mut self.seats {
            if seat.seat == handle {
                return seat
            }
        }
        panic!("{:?} not found", handle);
    }
}
