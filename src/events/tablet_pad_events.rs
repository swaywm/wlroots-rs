//! TODO Documentation

use libc::{c_double, c_uint};

use wlroots_sys::{wlr_event_tablet_pad_button, wlr_event_tablet_pad_ring,
                  wlr_event_tablet_pad_strip};

pub use wlroots_sys::{wlr_button_state, wlr_tablet_pad_ring_source, wlr_tablet_pad_strip_source};

#[derive(Debug)]
/// Event that is triggered when a tablet pad button event occurs.
pub struct ButtonEvent {
    event: *mut wlr_event_tablet_pad_button
}

#[derive(Debug)]
/// Event that is triggered when a ring event occurs.
pub struct RingEvent {
    event: *mut wlr_event_tablet_pad_ring
}

#[derive(Debug)]
/// Event that is triggered wen a strip event occurs
pub struct StripEvent {
    event: *mut wlr_event_tablet_pad_strip
}

impl ButtonEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_pad_button) -> Self {
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

    pub fn mode(&self) -> c_uint {
        unsafe { (*self.event).mode }
    }
}

impl RingEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_pad_ring) -> Self {
        RingEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    pub fn source(&self) -> wlr_tablet_pad_ring_source {
        unsafe { (*self.event).source }
    }

    pub fn ring(&self) -> u32 {
        unsafe { (*self.event).ring }
    }

    pub fn position(&self) -> c_double {
        unsafe { (*self.event).position }
    }

    pub fn mode(&self) -> c_uint {
        unsafe { (*self.event).mode }
    }
}

impl StripEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_tablet_pad_strip) -> Self {
        StripEvent { event }
    }

    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    pub fn source(&self) -> wlr_tablet_pad_strip_source {
        unsafe { (*self.event).source }
    }

    pub fn strip(&self) -> u32 {
        unsafe { (*self.event).strip }
    }

    pub fn position(&self) -> c_double {
        unsafe { (*self.event).position }
    }

    pub fn mode(&self) -> c_uint {
        unsafe { (*self.event).mode }
    }
}
