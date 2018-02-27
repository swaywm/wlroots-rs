//! TODO Documentation

use wlroots_sys::{wlr_event_touch_cancel, wlr_event_touch_down, wlr_event_touch_motion,
                  wlr_event_touch_up};

#[derive(Debug)]
/// Event that is triggered when a touch down event occurs.
pub struct DownEvent {
    event: *mut wlr_event_touch_down
}

#[derive(Debug)]
/// Event that is triggered when a touch up event occurs.
pub struct UpEvent {
    event: *mut wlr_event_touch_up
}

#[derive(Debug)]
/// Event that is triggered when a touch motion event occurs.
pub struct MotionEvent {
    event: *mut wlr_event_touch_motion
}

#[derive(Debug)]
/// Event that is triggered when a touch cancel event occurs.
pub struct CancelEvent {
    event: *mut wlr_event_touch_cancel
}

impl DownEvent {
    /// Constructs a `DownEvent` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_down) -> Self {
        DownEvent { event }
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
        unsafe { ((*self.event).x_mm, (*self.event).y_mm) }
    }

    /// Gets the size of the touch event in mm.
    ///
    /// Return value is in (w, h) format.
    pub fn size(&self) -> (f64, f64) {
        unsafe { ((*self.event).width_mm, (*self.event).height_mm) }
    }
}

impl UpEvent {
    /// Constructs a `UpEvent` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_up) -> Self {
        UpEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the touch id associated with this event.
    pub fn touch_id(&self) -> i32 {
        unsafe { (*self.event).touch_id }
    }
}

impl MotionEvent {
    /// Constructs a `MotionEvent` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_motion) -> Self {
        MotionEvent { event }
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
        unsafe { ((*self.event).x_mm, (*self.event).y_mm) }
    }

    /// Gets the size of the touch event in mm.
    ///
    /// Return value is in (w, h) format.
    pub fn size(&self) -> (f64, f64) {
        unsafe { ((*self.event).width_mm, (*self.event).height_mm) }
    }
}

impl CancelEvent {
    /// Constructs a `CancelEvent` from a raw event pointe
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_touch_cancel) -> Self {
        CancelEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the touch id associated with this event.
    pub fn touch_id(&self) -> i32 {
        unsafe { (*self.event).touch_id }
    }
}
