//! TODO Documentation

use std::{panic, ptr, rc::{Rc, Weak}, sync::atomic::{AtomicBool, Ordering}};

use errors::{UpgradeHandleErr, UpgradeHandleResult};
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
    liveliness: Option<Rc<AtomicBool>>,
    /// The device that refers to this pointer.
    device: InputDevice,
    /// The underlying pointer data.
    pointer: *mut wlr_pointer
}

#[derive(Debug)]
pub struct PointerHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<AtomicBool>,
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
                Some(Pointer { liveliness: Some(Rc::new(AtomicBool::new(false))),
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
    #[allow(dead_code)]
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_pointer {
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

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed Pointer")
            .store(val, Ordering::Release);
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
    /// Constructs a new PointerHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            PointerHandle { handle: Weak::new(),
                            // NOTE Rationale for null pointer here:
                            // It's never used, because you can never upgrade it,
                            // so no way to dereference it and trigger UB.
                            device: InputDevice::from_ptr(ptr::null_mut()),
                            pointer: ptr::null_mut() }
        }
    }

    /// Upgrades the pointer handle to a reference to the backing `Pointer`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Pointer`
    /// which may live forever..
    /// But no pointer lives forever and might be disconnected at any time.
    pub unsafe fn upgrade(&self) -> UpgradeHandleResult<Pointer> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let pointer = Pointer::from_handle(self);
                if check.load(Ordering::Acquire) {
                    wlr_log!(L_ERROR, "Double mutable borrows on {:?}", pointer);
                    panic!("Double mutable borrows detected");
                }
                check.store(true, Ordering::Release);
                Ok(pointer)
            })
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
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<R>
        where F: FnOnce(&Pointer) -> R
    {
        let mut pointer = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut pointer)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running pointer callback, mutable lock \
                                                    was false for: {:?}",
                                                   pointer);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this PointerHandle.
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    /// Gets the wlr_pointer associated with this PointerHandle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }
}

impl Clone for PointerHandle {
    fn clone(&self) -> Self {
        PointerHandle { pointer: self.pointer,
                        handle: self.handle.clone(),
                        /// NOTE Rationale for unsafe clone:
                        ///
                        /// You can only access it after a call to `upgrade`,
                        /// and that implicitly checks that it is valid.
                        device: unsafe { self.device.clone() } }
    }
}

impl PartialEq for PointerHandle {
    fn eq(&self, other: &PointerHandle) -> bool {
        self.pointer == other.pointer
    }
}

impl Eq for PointerHandle {}
