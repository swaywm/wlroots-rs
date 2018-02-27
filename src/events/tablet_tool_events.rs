//! TODO Documentation

use wlroots_sys::{wlr_button_state, wlr_event_tablet_tool_axis, wlr_event_tablet_tool_button,
                  wlr_event_tablet_tool_proximity, wlr_event_tablet_tool_tip,
                  wlr_tablet_tool_proximity_state, wlr_tablet_tool_tip_state};

#[derive(Debug)]
/// Event that is triggered when a tablet tool axis event occurs.
pub struct AxisEvent {
    event: *mut wlr_event_tablet_tool_axis
}

#[derive(Debug)]
/// Event that is triggered when a tablet tool proximity event occurs.
pub struct ProximityEvent {
    event: *mut wlr_event_tablet_tool_proximity
}

/// Event that is triggered when a tablet tool tip event occurs.
pub struct TipEvent {
    event: *mut wlr_event_tablet_tool_tip
}

/// Event that is triggered when a tablet tool button event occurs.
pub struct ButtonEvent {
    event: *mut wlr_event_tablet_tool_button
}

impl AxisEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_tool_axis) -> Self {
        AxisEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    pub fn updated_axes(&self) -> u32 {
        unsafe { (*self.event).updated_axes }
    }

    /// Gets the position of the event in mm.
    ///
    /// Return value is in (x, y) format.
    pub fn position(&self) -> (f64, f64) {
        unsafe { ((*self.event).x_mm, (*self.event).y_mm) }
    }

    /// Gets the size of the touch event in mm.
    ///
    /// Return value is in (w, h) format.
    pub fn size(&self) -> (f64, f64) {
        unsafe { ((*self.event).width_mm, (*self.event).height_mm) }
    }

    pub fn pressure(&self) -> f64 {
        unsafe { (*self.event).pressure }
    }

    pub fn distance(&self) -> f64 {
        unsafe { (*self.event).distance }
    }

    /// Gets the tilt of the event.
    ///
    /// Return value is in (x, y) format.
    pub fn tilt(&self) -> (f64, f64) {
        unsafe { ((*self.event).tilt_x, (*self.event).tilt_y) }
    }

    pub fn slider(&self) -> f64 {
        unsafe { (*self.event).slider }
    }

    pub fn wheel_delta(&self) -> f64 {
        unsafe { (*self.event).wheel_delta }
    }
}

impl ProximityEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_tool_proximity) -> Self {
        ProximityEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the position of the event in mm.
    ///
    /// Return value is in (x, y) format.
    pub fn position(&self) -> (f64, f64) {
        unsafe { ((*self.event).x_mm, (*self.event).y_mm) }
    }

    /// Gets the size of the touch event in mm.
    ///
    /// Return value is in (w, h) format.
    pub fn size(&self) -> (f64, f64) {
        unsafe { ((*self.event).width_mm, (*self.event).height_mm) }
    }

    pub fn state(&self) -> wlr_tablet_tool_proximity_state {
        unsafe { (*self.event).state }
    }
}

impl TipEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_tool_tip) -> Self {
        TipEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Gets the position of the event in mm.
    ///
    /// Return value is in (x, y) format.
    pub fn position(&self) -> (f64, f64) {
        unsafe { ((*self.event).x_mm, (*self.event).y_mm) }
    }

    /// Gets the size of the touch event in mm.
    ///
    /// Return value is in (w, h) format.
    pub fn size(&self) -> (f64, f64) {
        unsafe { ((*self.event).width_mm, (*self.event).height_mm) }
    }

    pub fn state(&self) -> wlr_tablet_tool_tip_state {
        unsafe { (*self.event).state }
    }
}

impl ButtonEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_tool_button) -> Self {
        ButtonEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    pub fn button(&self) -> u32 {
        unsafe { (*self.event).button }
    }

    pub fn state(&self) -> wlr_button_state {
        unsafe { (*self.event).state }
    }
}
