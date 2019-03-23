use wlroots_sys::{wlr_event_switch_toggle, wlr_switch_state, wlr_switch_type};

use input;

pub struct Toggle {
    event: *mut wlr_event_switch_toggle,
    device: input::Device
}

impl Toggle {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_event_switch_toggle) -> Self {
        Toggle {
            event,
            device: input::Device::from_ptr((*event).device)
        }
    }

    pub fn device(&self) -> &input::Device {
        &self.device
    }

    /// Get the timestamp of this event.
    pub fn time_msec(&self) -> u32 {
        unsafe { (*self.event).time_msec }
    }

    /// Get the type of switch this is.
    pub fn switch_type(&self) -> wlr_switch_type {
        unsafe { (*self.event).switch_type }
    }

    /// Get the state the switch is in.
    pub fn switch_state(&self) -> wlr_switch_state {
        unsafe { (*self.event).switch_state }
    }
}
