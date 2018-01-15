//! Pointers and their events

use types::input_device::InputDevice;

use wlroots_sys::{wlr_button_state, wlr_event_pointer_axis, wlr_event_pointer_button,
                  wlr_event_pointer_motion, wlr_event_pointer_motion_absolute};

// TODO FIXME Document this shit man

#[derive(Debug)]
pub struct AxisEvent {
    event: *mut wlr_event_pointer_axis,
    device: InputDevice
}

#[derive(Debug)]
pub struct ButtonEvent {
    event: *mut wlr_event_pointer_button,
    device: InputDevice
}

#[derive(Debug)]
pub struct MotionEvent {
    event: *mut wlr_event_pointer_motion,
    device: InputDevice
}

#[derive(Debug)]
pub struct AbsoluteMotionEvent {
    event: *mut wlr_event_pointer_motion_absolute,
    device: InputDevice
}

impl ButtonEvent {
    pub unsafe fn from_ptr(event: *mut wlr_event_pointer_button) -> Self {
        ButtonEvent { device: InputDevice::from_ptr((*event).device), event }
    }

    pub fn device(&self) -> &InputDevice {
        &self.device
    }

    pub fn state(&self) -> wlr_button_state {
        unsafe { (*self.event).state }
    }

    pub fn button(&self) -> u32 {
        unsafe { (*self.event).button }
    }
}

impl AxisEvent {
    pub unsafe fn from_ptr(event: *mut wlr_event_pointer_axis) -> Self {
        AxisEvent { device: InputDevice::from_ptr((*event).device), event }
    }

    pub fn device(&self) -> &InputDevice {
        &self.device
    }

    pub fn delta(&self) -> f64 {
        unsafe { (*self.event).delta }
    }
}

impl MotionEvent {
    pub unsafe fn from_ptr(event: *mut wlr_event_pointer_motion) -> Self {
        MotionEvent { device: InputDevice::from_ptr((*event).device), event }
    }

    pub fn device(&self) -> &InputDevice {
        &self.device
    }

    pub fn delta(&self) -> (f64, f64) {
        unsafe { ((*self.event).delta_x, (*self.event).delta_y) }
    }
}

impl AbsoluteMotionEvent {
    pub unsafe fn from_ptr(event: *mut wlr_event_pointer_motion_absolute) -> Self {
        AbsoluteMotionEvent { device: InputDevice::from_ptr((*event).device), event }
    }

    pub fn device(&self) -> &InputDevice {
        &self.device
    }
}
