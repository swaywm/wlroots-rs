//! Pointers and their events

use libc::c_double;
use wlroots_sys::{
    wlr_axis_orientation, wlr_axis_source, wlr_button_state, wlr_event_pointer_axis,
    wlr_event_pointer_button, wlr_event_pointer_motion, wlr_event_pointer_motion_absolute
};

use input;

// NOTE Taken from linux/input-event-codes.h
// TODO Find a way to automatically parse and fetch from there.
pub const BTN_MOUSE: u32 = 0x110;
pub const BTN_LEFT: u32 = 0x110;
pub const BTN_RIGHT: u32 = 0x111;
pub const BTN_MIDDLE: u32 = 0x112;
pub const BTN_SIDE: u32 = 0x113;
pub const BTN_EXTRA: u32 = 0x114;
pub const BTN_FORWARD: u32 = 0x115;
pub const BTN_BACK: u32 = 0x116;
pub const BTN_TASK: u32 = 0x117;

/// Event that triggers when the pointer device scrolls (e.g using a wheel
// or in the case of a touchpad when you use two fingers to scroll).
#[derive(Debug)]
pub struct Axis {
    event: *mut wlr_event_pointer_axis,
    device: input::Device
}

/// Event that triggers when a button is pressed (e.g left click, right click,
/// a gaming mouse button, etc.).
#[derive(Debug)]
pub struct Button {
    event: *mut wlr_event_pointer_button,
    device: input::Device
}

/// Event that triggers when the pointer moves.
#[derive(Debug)]
pub struct Motion {
    event: *mut wlr_event_pointer_motion,
    device: input::Device
}

/// Event that triggers when data from a device that supports absolute motion
/// sends data to the compositor.
///
/// For more information on absolute motion, [see this link](https://wayland.freedesktop.org/libinput/doc/latest/absolute_axes.html).
#[derive(Debug)]
pub struct AbsoluteMotion {
    event: *mut wlr_event_pointer_motion_absolute,
    device: input::Device
}

impl Button {
    /// Constructs a `Button` from the raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_pointer_button) -> Self {
        Button {
            device: input::Device::from_ptr((*event).device),
            event
        }
    }

    /// Get the device this event refers to.
    pub fn device(&self) -> &input::Device {
        &self.device
    }

    /// Get the state of the button (e.g pressed or released).
    pub fn state(&self) -> wlr_button_state {
        unsafe { (*self.event).state }
    }

    /// Get the timestamp of this event.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Get the value of the button pressed. This will generally be an
    /// atomically increasing value, with e.g left click being 1 and right
    /// click being 2...
    ///
    /// We make no guarantees that 1 always maps to left click, as this is
    /// device driver specific.
    pub fn button(&self) -> u32 {
        unsafe { (*self.event).button }
    }
}

impl Axis {
    /// Constructs a `Axis` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_pointer_axis) -> Self {
        Axis {
            device: input::Device::from_ptr((*event).device),
            event
        }
    }

    /// Get the device this event refers to.
    pub fn device(&self) -> &input::Device {
        &self.device
    }

    /// Get the timestamp of this event.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    pub fn source(&self) -> wlr_axis_source {
        unsafe { (*self.event).source }
    }

    pub fn orientation(&self) -> wlr_axis_orientation {
        unsafe { (*self.event).orientation }
    }

    /// Get the change from the last axis value.
    ///
    /// Useful to determine e.g how much to scroll.
    pub fn delta(&self) -> f64 {
        unsafe { (*self.event).delta }
    }
}

impl Motion {
    /// Constructs a `Motion` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_pointer_motion) -> Self {
        Motion {
            device: input::Device::from_ptr((*event).device),
            event
        }
    }

    /// Get the device this event refers to.
    pub fn device(&self) -> &input::Device {
        &self.device
    }

    /// Get the timestamp of this event.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Get the change from the last positional value.
    ///
    /// Returned in (x, y) form.
    ///
    /// Note you should not cast this to a type with less precision,
    /// otherwise you'll lose important motion data which can cause bugs
    /// (e.g see [this fun wlc bug](https://github.com/Cloudef/wlc/issues/181)).
    pub fn delta(&self) -> (f64, f64) {
        unsafe { ((*self.event).delta_x, (*self.event).delta_y) }
    }
}

impl AbsoluteMotion {
    /// Construct an `AbsoluteMotion` from a raw event pointer.
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_pointer_motion_absolute) -> Self {
        AbsoluteMotion {
            device: input::Device::from_ptr((*event).device),
            event
        }
    }

    /// Get the timestamp of this event.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Get the absolute position of the pointer from this event.
    pub fn pos(&self) -> (c_double, c_double) {
        unsafe { ((*self.event).x, (*self.event).y) }
    }

    /// Get the device this event refers to.
    pub fn device(&self) -> &input::Device {
        &self.device
    }
}
