//! TODO Documentation

use std::rc::{Rc, Weak};

use wlroots_sys::{wlr_input_device, wlr_pointer};

use InputDevice;

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
    device: InputDevice,
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
    device: InputDevice,
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
                Some(Pointer { liveliness: Some(Rc::new(())),
                               device: InputDevice::from_ptr(device),
                               pointer })
            }
            _ => None
        }
    }

    /// Creates an unbound Pointer from a `PointerHandle`
    unsafe fn from_handle(handle: &PointerHandle) -> Self {
        Pointer { liveliness: None,
                  device: handle.input_device().clone(),
                  pointer: handle.as_ptr() }
    }

    /// Gets the wlr_input_device associated with this Pointer.
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    /// Gets the wlr_pointer associated with this Pointer.
    pub unsafe fn as_ptr(&self) -> *mut wlr_pointer {
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
                        // NOTE Rationale for cloning:
                        // We can't use the keyboard handle unless the keyboard is alive,
                        // which means the device pointer is still alive.
                        device: unsafe { self.device.clone() },
                        pointer: self.pointer }
    }
}

impl Drop for Pointer {
    fn drop(&mut self) {
        match self.liveliness {
            None => {}
            Some(ref liveliness) => {
                if Rc::strong_count(liveliness) == 1 {
                    wlr_log!(L_DEBUG, "Dropped Pointer {:p}", self.pointer);
                    let weak_count = Rc::weak_count(liveliness);
                    if weak_count > 0 {
                        wlr_log!(L_DEBUG,
                                 "Still {} weak pointers to Pointer {:p}",
                                 weak_count,
                                 self.pointer);
                    }
                }
            }
        }
    }
}

impl PointerHandle {
    /// Upgrades the pointer handle to a reference to the backing `Pointer`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Pointer`
    /// which may live forever..
    /// But no pointer lives forever and might be disconnected at any time.
    pub unsafe fn upgrade(&self) -> Option<Pointer> {
        self.handle.upgrade()
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .map(|_| Pointer::from_handle(self))
    }

    /// Run a function on the referenced Pointer, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Pointer does not live longer
    /// than it exists.
    pub fn run<F, R>(&self, runner: F) -> Option<R>
        where F: FnOnce(&Pointer) -> R
    {
        let pointer = unsafe { self.upgrade() };
        match pointer {
            None => None,
            Some(pointer) => Some(runner(&pointer))
        }
    }

    /// Gets the wlr_input_device associated with this PointerHandle.
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    /// Gets the wlr_pointer associated with this PointerHandle.
    pub unsafe fn as_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }
}
