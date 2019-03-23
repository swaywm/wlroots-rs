//! Timing is important for compositors and clients to know when to render
//! frames. Most of these functions will be used for that purpose.

use std::time::Duration;

use crate::libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

/// Trait to convert something to milliseconds.
///
/// Used primarily to convert a `std::time::Duration` into
/// something usable by wlroots
pub trait ToMs {
    /// Convert the time to a millisecond representation.
    ///
    /// This conversion should be lossless.
    fn to_ms(self) -> u32;
}

impl ToMs for Duration {
    fn to_ms(self) -> u32 {
        let seconds_delta = self.as_secs() as u32;
        let mili_delta = self.subsec_millis();
        (seconds_delta * 1000) + mili_delta
    }
}

/// Get the current time as a duration suitable for functions such as
/// `surface.send_frame_done()`.
pub fn current_time() -> Duration {
    unsafe {
        let mut ts = timespec {
            tv_sec: 0,
            tv_nsec: 0
        };
        clock_gettime(CLOCK_MONOTONIC, &mut ts);
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}
