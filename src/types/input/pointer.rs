//! TODO Documentation

use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}};

use errors::{HandleErr, HandleResult};
use wlroots_sys::{wlr_input_device, wlr_pointer};

use super::input_device::{InputDevice, InputState};

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
    liveliness: Rc<Cell<bool>>,
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
    handle: Weak<Cell<bool>>,
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
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState { handle,
                                                  device: InputDevice::from_ptr(device) });
                (*pointer).data = Box::into_raw(state) as *mut _;
                Some(Pointer { liveliness,
                               device: InputDevice::from_ptr(device),
                               pointer })
            }
            _ => None
        }
    }

    /// Creates an unbound Pointer from a `PointerHandle`
    unsafe fn from_handle(handle: &PointerHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Pointer { liveliness,
                     device: handle.input_device()?.clone(),
                     pointer: handle.as_ptr() })
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
        PointerHandle { handle: Rc::downgrade(&self.liveliness),
                        // NOTE Rationale for cloning:
                        // We can't use the keyboard handle unless the keyboard is alive,
                        // which means the device pointer is still alive.
                        device: unsafe { self.device.clone() },
                        pointer: self.pointer }
    }
}

impl Drop for Pointer {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(L_DEBUG, "Dropped Pointer {:p}", self.pointer);
            unsafe {
                let _ = Box::from_raw((*self.pointer).data as *mut InputState);
            }
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(L_DEBUG,
                         "Still {} weak pointers to Pointer {:p}",
                         weak_count,
                         self.pointer);
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

    /// Creates a PointerHandle from the raw pointer, using the saved user
    /// data to recreate the memory model.
    ///
    /// # Panics
    /// Panics if the wlr_pointer wasn't allocated using `new_from_input_device`.
    pub(crate) unsafe fn from_ptr(pointer: *mut wlr_pointer) -> Self {
        if (*pointer).data.is_null() {
            panic!("Tried to get handle to keyboard that wasn't set up properly");
        }
        let data = Box::from_raw((*pointer).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*pointer).data = Box::into_raw(data) as *mut _;
        PointerHandle { handle,
                        device,
                        pointer }
    }

    /// Upgrades the pointer handle to a reference to the backing `Pointer`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Pointer`
    /// which may live forever..
    /// But no pointer lives forever and might be disconnected at any time.
    pub unsafe fn upgrade(&self) -> HandleResult<Pointer> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let pointer = Pointer::from_handle(self)?;
                if check.get() {
                    wlr_log!(L_ERROR, "Double mutable borrows on {:?}", pointer);
                    panic!("Double mutable borrows detected");
                }
                check.set(true);
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
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&Pointer) -> R
    {
        let mut pointer = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut pointer)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running pointer callback, mutable lock \
                                                    was false for: {:?}",
                                                   pointer);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this PointerHandle.
    pub fn input_device(&self) -> HandleResult<&InputDevice> {
        match self.handle.upgrade() {
            Some(_) => Ok(&self.device),
            None => Err(HandleErr::AlreadyDropped)
        }
    }

    /// Gets the wlr_pointer associated with this PointerHandle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_pointer {
        self.pointer
    }
}

impl Default for PointerHandle {
    fn default() -> Self {
        PointerHandle::new()
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
