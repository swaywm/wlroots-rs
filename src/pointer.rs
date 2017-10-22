//! Pointers and their events

use wlroots_sys::{wlr_event_pointer_button, wlr_button_state};

pub struct ButtonEvent {
    event: *mut wlr_event_pointer_button
}

impl ButtonEvent {
    pub unsafe fn from_ptr(event: *mut wlr_event_pointer_button) -> ButtonEvent {
        ButtonEvent { event }
    }

    pub fn state(&self) -> wlr_button_state {
        unsafe {
            (*self.event).state
        }
    }

    pub fn button(&self) -> u32 {
        unsafe {
            (*self.event).button
        }
    }
}
