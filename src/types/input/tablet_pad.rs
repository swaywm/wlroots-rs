//! TODO Documentation
use std::{cell::Cell, ptr::NonNull, rc::Rc};

use wlroots_sys::{wlr_input_device, wlr_tablet_pad};

pub use crate::events::tablet_pad_events as event;
pub use crate::manager::tablet_pad_handler::*;
use crate::{
    input::{self, InputState},
    utils::{self, HandleErr, HandleResult, Handleable}
};

pub type Handle = utils::Handle<NonNull<wlr_input_device>, wlr_tablet_pad, TabletPad>;

#[derive(Debug)]
pub struct TabletPad {
    /// The structure that ensures weak handles to this structure are still
    /// alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `tablet_pad::Handle`,
    /// and the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `tablet_pad::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this tablet pad.
    device: input::Device,
    /// Underlying tablet state
    pad: NonNull<wlr_tablet_pad>
}

impl TabletPad {
    /// Tries to convert an input device to a TabletPad
    ///
    /// Returns None if it is of a different type of input variant.
    ///
    /// # Safety
    /// This creates a totally new TabletPad (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_TABLET_PAD => {
                let pad = NonNull::new((*device).__bindgen_anon_1.tablet_pad).expect(
                    "Pad pointer was \
                     null"
                );
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState {
                    handle,
                    device: input::Device::from_ptr(device)
                });
                (*pad.as_ptr()).data = Box::into_raw(state) as *mut _;
                Some(TabletPad {
                    liveliness,
                    device: input::Device::from_ptr(device),
                    pad
                })
            },
            _ => None
        }
    }

    /// Gets the wlr_input_device associated with this TabletPad.
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }
}

impl Drop for TabletPad {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) != 1 {
            return;
        }
        wlr_log!(WLR_DEBUG, "Dropped TabletPad {:p}", self.pad.as_ptr());
        unsafe {
            let _ = Box::from_raw((*self.pad.as_ptr()).data as *mut InputState);
        }
        let weak_count = Rc::weak_count(&self.liveliness);
        if weak_count > 0 {
            wlr_log!(
                WLR_DEBUG,
                "Still {} weak pointers to TabletPad {:p}",
                weak_count,
                self.pad.as_ptr()
            );
        }
    }
}

impl Handleable<NonNull<wlr_input_device>, wlr_tablet_pad> for TabletPad {
    #[doc(hidden)]
    unsafe fn from_ptr(pad: *mut wlr_tablet_pad) -> Option<Self> {
        let pad = NonNull::new(pad)?;
        let data = Box::from_raw((*pad.as_ptr()).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*pad.as_ptr()).data = Box::into_raw(data) as *mut _;
        Some(TabletPad {
            liveliness: handle.upgrade().unwrap(),
            device,
            pad
        })
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_tablet_pad {
        self.pad.as_ptr()
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle.upgrade().ok_or(HandleErr::AlreadyDropped)?;
        let device = handle.data.ok_or(HandleErr::AlreadyDropped)?;
        Ok(TabletPad {
            liveliness,
            // NOTE Rationale for cloning:
            // If we already dropped we don't reach this point.
            device: input::Device { device },
            pad: handle.as_non_null()
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle {
            ptr: self.pad,
            handle: Rc::downgrade(&self.liveliness),
            // NOTE Rationale for cloning:
            // Since we have a strong reference already,
            // the input must still be alive.
            data: unsafe { Some(self.device.as_non_null()) },
            _marker: std::marker::PhantomData
        }
    }
}
