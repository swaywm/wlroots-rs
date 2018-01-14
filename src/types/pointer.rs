//! TODO Documentation

use std::rc::{Rc, Weak};

use wlroots_sys::{wlr_input_device, wlr_pointer};

#[derive(Debug)]
pub struct Pointer {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `PointerHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `PointerHandle`.
    liveliness: Option<Rc<()>>,
    /// The device that refers to this pointer.
    device: *mut wlr_input_device,
    /// The underlying pointer data.
    pointer: *mut wlr_pointer
}

/// A wlr_input_device that is guaranteed to be a pointer.
#[derive(Debug)]
pub struct PointerHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<()>,
    /// The device that refers to this pointer.
    device: *mut wlr_input_device,
    /// The underlying pointer data.
    pointer: *mut wlr_pointer
}

impl Pointer {
    /// Tries to convert an input device to a pointer
    ///
    /// Returns none if it is of a different input varient.
    pub(crate) unsafe fn from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_POINTER => {
                let pointer = (*device).__bindgen_anon_1.pointer;
                Some(Pointer { liveliness: Some(Rc::new(())),
                               device,
                               pointer })
            }
            _ => None
        }
    }

    /// Creates an unbound Pointer from a `PointerHandle`
    unsafe fn from_handle(handle: &PointerHandle) -> Self {
        Pointer { liveliness: None,
                  device: handle.input_device(),
                  pointer: handle.pointer_ptr() }
    }

    /// Gets the wlr_input_device associated with this Pointer.
    pub unsafe fn input_device(&self) -> *mut wlr_input_device {
        self.device
    }

    /// Gets the wlr_pointer associated with this Pointer.
    pub unsafe fn pointer_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }

    /// Creates a weak reference to a `Pointer`.
    ///
    /// # Panics
    /// If this `Pointer` is a previously upgraded `PointerHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> PointerHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade a previously upgraded PointerHandle!");
        PointerHandle { handle: Rc::downgrade(arc),
                        device: self.device,
                        pointer: self.pointer }
    }
}

impl PointerHandle {
    /// Upgrades the pointer handle to a reference to the backing `Pointer`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates a lifetime bound to
    /// PointerHandle, which may live forever..
    /// But no output lives forever and might be disconnected at any time.
    pub unsafe fn upgrade(&self) -> Option<Pointer> {
        self.handle.upgrade()
            // NOTE
            // We drop the upgrade here because we don't want to cause a memory leak!
            .map(|_| Pointer::from_handle(self))
    }

    /// Gets the wlr_input_device associated with this PointerHandle.
    pub unsafe fn input_device(&self) -> *mut wlr_input_device {
        self.device
    }

    /// Gets the wlr_pointer associated with this PointerHandle.
    pub unsafe fn pointer_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }
}
