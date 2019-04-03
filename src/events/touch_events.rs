//! TODO Documentation

use wlroots_sys::{wlr_event_touch_cancel, wlr_event_touch_down, wlr_event_touch_motion, wlr_event_touch_up};

#[derive(Debug)]
/// Event that is triggered when a touch down event occurs.
pub struct Down {
    event: *mut wlr_event_touch_down
}

#[derive(Debug)]
/// Event that is triggered when a touch up event occurs.
pub struct Up {
    event: *mut wlr_event_touch_up
}

#[derive(Debug)]
/// Event that is triggered when a touch motion event occurs.
pub struct Motion {
    event: *mut wlr_event_touch_motion
}

#[derive(Debug)]
/// Event that is triggered when a touch cancel event occurs.
pub struct Cancel {
    event: *mut wlr_event_touch_cancel
}

impl Down {
    /// Constructs a `Down` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_down) -> Self {
        Down { event }
    }

    /// Gets how long the touch event has been going on for.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the touch id associated with this event.
    pub fn touch_id(&self) -> i32 {
        unsafe { (*self.event).touch_id }
    }

    /// Gets the location of the touch event in mm.
    ///
    /// Return value is in (x, y) format.
    pub fn location(&self) -> (f64, f64) {
        unsafe { ((*self.event).x, (*self.event).y) }
    }
}

impl Up {
    /// Constructs a `Up` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_up) -> Self {
        Up { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the touch id associated with this event.
    pub fn touch_id(&self) -> i32 {
        unsafe { (*self.event).touch_id }
    }
}

impl Motion {
    /// Constructs a `Motion` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_motion) -> Self {
        Motion { event }
    }

    /// Gets how long the touch event has been going on for.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the touch id associated with this event.
    pub fn touch_id(&self) -> i32 {
        unsafe { (*self.event).touch_id }
    }

    /// Gets the location of the touch event in mm.
    ///
    /// Return value is in (x, y) format.
    pub fn location(&self) -> (f64, f64) {
        unsafe { ((*self.event).x, (*self.event).y) }
    }
}

impl Cancel {
    /// Constructs a `Cancel` from a raw event pointe
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_cancel) -> Self {
        Cancel { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the touch id associated with this event.
    pub fn touch_id(&self) -> i32 {
        unsafe { (*self.event).touch_id }
    }
}
