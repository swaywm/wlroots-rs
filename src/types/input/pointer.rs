//! TODO Documentation

use std::{cell::Cell, rc::Rc};

use wlroots_sys::{wlr_input_device, wlr_pointer};

use {input::{self, InputState},
     utils::{self, Handleable, HandleErr, HandleResult}};
pub use manager::pointer_handler::*;
pub use events::pointer_events as event;

pub type Handle = utils::Handle<*mut wlr_input_device, wlr_pointer, Pointer>;

#[derive(Debug)]
pub struct Pointer {
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
    /// The underlying pointer data.
    pointer: *mut wlr_pointer
}

impl Pointer {
    /// Tries to convert an input device to a Pointer
    ///
    /// Returns none if it is of a different input variant.
    ///
    /// # Safety
    /// This creates a totally new Pointer (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_POINTER => {
                let pointer = (*device).__bindgen_anon_1.pointer;
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState { handle,
                                                  device: input::Device::from_ptr(device) });
                (*pointer).data = Box::into_raw(state) as *mut _;
                Some(Pointer { liveliness,
                               device: input::Device::from_ptr(device),
                               pointer })
            }
            _ => None
        }
    }

    /// Gets the wlr_input_device associated with this Pointer.
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }

    /// Gets the wlr_pointer associated with this Pointer.
    #[allow(dead_code)]
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }
}

impl Drop for Pointer {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped Pointer {:p}", self.pointer);
            unsafe {
                let _ = Box::from_raw((*self.pointer).data as *mut InputState);
            }
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(WLR_DEBUG,
                         "Still {} weak pointers to Pointer {:p}",
                         weak_count,
                         self.pointer);
            }
        }
    }
}

impl Handleable<*mut wlr_input_device, wlr_pointer> for Pointer {
    #[doc(hidden)]
    unsafe fn from_ptr(pointer: *mut wlr_pointer) -> Self {
        let data = Box::from_raw((*pointer).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*pointer).data = Box::into_raw(data) as *mut _;
        Pointer { liveliness: handle.upgrade().unwrap(),
                  device,
                  pointer }
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
            .upgrade()
            .ok_or(HandleErr::AlreadyDropped)?;
        Ok(Pointer { liveliness,
                     // NOTE Rationale for cloning:
                     // If we already dropped we don't reach this point.
                     device: input::Device { device: handle.data },
                     pointer: handle.as_ptr()
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle { ptr: self.pointer,
                 handle: Rc::downgrade(&self.liveliness),
                 // NOTE Rationale for cloning:
                 // Since we have a strong reference already,
                 // the input must still be alive.
                 data: unsafe { self.device.as_ptr() },
                 _marker: std::marker::PhantomData
        }
    }
}
