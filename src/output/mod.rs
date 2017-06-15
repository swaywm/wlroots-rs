//! Methods and structs that control the physical outputs and what they display.


mod output;

pub use self::output::Output;

use std::sync::Mutex;

lazy_static! {
    pub static ref OUTPUTS: Mutex<Vec<Output>> = Mutex::new(Vec::new());
}
