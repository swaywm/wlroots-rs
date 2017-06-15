//! Methods and structs that control the physical outputs and what they display.


mod output;

pub use self::output::Output;
use ::Session;

use std::sync::Mutex;

lazy_static! {
    pub static ref OUTPUTS: Mutex<Vec<Output>> = Mutex::new(Vec::new());
}

/// Sets up the session so that it listens for and automatically manages adding
/// and removing outputs.
pub unsafe fn init(session: &mut Session) {
    //wl_signal_add(&mut (*session.backend).events.output_add,
    //              &mut output_add)
}
