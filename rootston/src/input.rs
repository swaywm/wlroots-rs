use super::{seat, server::Server};
use wlroots::{self, Capability, Compositor, HandleResult, InputDevice, InputManagerHandler};

const DEFAULT_SEAT_NAME: &str = "seat0";

pub struct InputManager {
    seats: Vec<seat::Seat>
}

impl InputManagerHandler for InputManager {
    fn input_added(&mut self, compositor: &mut Compositor, input: &mut InputDevice) {
        let seat_name;
        {
            let state: &mut Server = compositor.into();
            seat_name = match state.config
                                   .devices
                                   .iter()
                                   .find(|dev_conf| dev_conf.name.as_str() == DEFAULT_SEAT_NAME)
            {
                None => DEFAULT_SEAT_NAME.into(),
                Some(dev) => dev.name.clone()
            };
        }
        let seat = self.get_seat(compositor, &*seat_name).expect("Could make a new seat");
        self.add_device_to_seat(compositor, seat, input).expect("Could not add a device to a seat");
    }
}

impl InputManager {
    pub fn new() -> Self {
        InputManager { seats: vec![] }
    }

    /// Gets the seat by name. If it doesn't exist yet, create it.
    fn get_seat(&mut self,
                compositor: &mut Compositor,
                seat_name: &str)
                -> HandleResult<wlroots::SeatHandle> {
        let found = &mut false;
        for seat in &mut self.seats {
            seat.seat.clone().run(|seat| {
                                       if seat.name().as_ref().map(|s| &**s) == Some(seat_name) {
                                           *found = true;
                                       }
                                   })?;
            if *found {
                return Ok(seat.seat.clone())
            }
        }
        let seat_handle = wlroots::Seat::create(compositor,
                                                seat_name.into(),
                                                Box::new(seat::SeatHandler::new()));
        let seat = seat::Seat::new(seat_handle);
        self.seats.push(seat);
        Ok(self.seats.last_mut().expect("impossible").seat.clone())
    }

    pub fn add_device_to_seat(&mut self,
                              compositor: &mut Compositor,
                              mut seat: wlroots::SeatHandle,
                              input: &mut InputDevice)
                              -> HandleResult<()> {
        use wlroots::wlr_input_device_type::*;
        match input.dev_type() {
            WLR_INPUT_DEVICE_KEYBOARD => {
                seat.run(|seat| {
                              seat.set_keyboard(input);
                          })?
            }
            WLR_INPUT_DEVICE_POINTER => {
                {
                    let roots_seat = self.roots_seat_from_handle(seat.clone());
                    if let Some(cursor) = roots_seat.cursor.as_mut() {
                        cursor.cursor.run(|cursor| {
                                               cursor.attach_input_device(input);
                                           })?;
                    }
                }
                self.configure_cursor(compositor, seat.clone())?;
                seat.run(|seat| {
                              let mut capabilities = seat.capabilities();
                              capabilities.insert(Capability::Pointer);
                              seat.set_capabilities(capabilities);
                          })?
            }
            WLR_INPUT_DEVICE_TOUCH | WLR_INPUT_DEVICE_TABLET_TOOL => {
                {
                    let roots_seat = self.roots_seat_from_handle(seat.clone());
                    if let Some(cursor) = roots_seat.cursor.as_mut() {
                        cursor.cursor.run(|cursor| {
                                               cursor.attach_input_device(input);
                                           })?;
                    }
                }
                self.configure_cursor(compositor, seat.clone())?;
                seat.run(|seat| {
                              let mut capabilities = seat.capabilities();
                              capabilities.insert(Capability::Touch);
                              seat.set_capabilities(capabilities);
                          })?
            }
            WLR_INPUT_DEVICE_TABLET_PAD => { /*TODO*/ }
        }
        Ok(())
    }

    pub fn configure_cursor(&mut self,
                            // TODO Remove?
                            compositor: &mut Compositor,
                            seat: wlroots::SeatHandle)
                            -> HandleResult<()> {
        let state: &mut Server = compositor.into();
        let roots_seat = self.roots_seat_from_handle(seat.clone());
        if let Some(cursor) = roots_seat.cursor.as_mut() {
            let pointers = &mut roots_seat.pointers;
            let touches = &mut roots_seat.touch;
            run_handles!([(seat: {seat}), (cursor: {cursor.cursor.clone()})] => {
                // reset mappings
                cursor.map_to_output(None);
                for pointer in pointers {
                    cursor.map_input_to_output(pointer.input_device(),
                                               None)
                }
                // TODO Also map input to region if part of config
                for touch in touches {
                    cursor.map_input_to_output(touch.input_device(), None)
                }
                // TODO table tool
                let outputs = &mut state.outputs;
                let seat_name = seat.name().unwrap_or_else(|| "".into());
                match state.config
                    .cursors
                    .iter()
                    .find(|cursor| cursor.seat == seat_name)
                {
                    None => {},
                    Some(cursor_config) => {
                        if let Some(mapped_output_name) =
                            cursor_config.mapped_output.as_ref()
                        {
                            for output in outputs {
                                output.run(|output| {
                                    if output.name() == *mapped_output_name
                                    {
                                        cursor.map_to_output(output)
                                    }
                                }).unwrap();
                            }
                        }
                    }
                }
            })?;
            //                                         Some(cursor)
            //                                     })
            //                                .ok()?;
            //                          Some(seat)
            //                      })?;
        }

        // configure device to output mappings
        Ok(())
    }

    fn roots_seat_from_handle(&mut self, handle: wlroots::SeatHandle) -> &mut seat::Seat {
        for seat in &mut self.seats {
            if seat.seat == handle {
                return seat
            }
        }
        panic!("{:?} not found", handle);
    }
}
