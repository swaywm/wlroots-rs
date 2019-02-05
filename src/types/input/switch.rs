//! TODO Documentation

use std::{cell::Cell, rc::Rc};

use {
    input::{self, InputState},
    utils::{self, Handleable, HandleErr, HandleResult}};
use wlroots_sys::{wlr_input_device, wlr_switch};
pub use manager::switch_handler::*;
pub use events::switch_events as event;

pub type Handle = utils::Handle<*mut wlr_input_device, wlr_switch, Switch>;

#[derive(Debug)]
pub struct Switch {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `pointer::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `pointer::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this pointer.
    device: input::Device,
    /// The underlying switch data.
    switch: *mut wlr_switch
}

impl Switch {
    /// Tries to convert an input device to a Switch
    ///
    /// Returns none if it is of a different input variant.
    ///
    /// # Safety
    /// This creates a totally new Switch (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_SWITCH => {
                let switch = (*device).__bindgen_anon_1.lid_switch;
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState { handle,
                                                  device: input::Device::from_ptr(device) });
                (*switch).data = Box::into_raw(state) as *mut _;
                Some(Switch { liveliness,
                              device: input::Device::from_ptr(device),
                              switch })
            }
            _ => None
        }
    }

    /// Gets the wlr_input_device associated with this switch.
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }
}

impl Drop for Switch {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped Switch {:p}", self.switch);
            unsafe {
                let _ = Box::from_raw((*self.switch).data as *mut InputState);
            }
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(WLR_DEBUG,
                         "Still {} weak pointers to Switch {:p}",
                         weak_count,
                         self.switch);
            }
        }
    }
}

impl Handleable<*mut wlr_input_device, wlr_switch> for Switch {
    #[doc(hidden)]
    unsafe fn from_ptr(switch: *mut wlr_switch) -> Self {
        let data = Box::from_raw((*switch).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*switch).data = Box::into_raw(data) as *mut _;
        Switch { liveliness: handle.upgrade().unwrap(),
                 device,
                 switch }
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_switch {
        self.switch
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
            .upgrade()
            .ok_or(HandleErr::AlreadyDropped)?;
        Ok(Switch { liveliness,
                    // NOTE Rationale for cloning:
                    // If we already dropped we don't reach this point.
                    device: input::Device { device: handle.data },
                    switch: handle.as_ptr()
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle { ptr: self.switch,
                 handle: Rc::downgrade(&self.liveliness),
                 // NOTE Rationale for cloning:
                 // Since we have a strong reference already,
                 // the input must still be alive.
                 data: unsafe { self.device.as_ptr() },
                 _marker: std::marker::PhantomData
        }
    }
}
