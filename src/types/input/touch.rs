//! TODO Documentation

use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}};

use errors::{HandleErr, HandleResult};
use wlroots_sys::{wlr_input_device, wlr_touch};

use super::input_device::{InputDevice, InputState};

#[derive(Debug)]
pub struct Touch {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `TouchHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `TouchHandle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this touch.
    device: InputDevice,
    /// The underlying touch data.
    touch: *mut wlr_touch
}

#[derive(Debug)]
pub struct TouchHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the touch associated with this handle,
    /// this can no longer be used.
    handle: Weak<Cell<bool>>,
    /// The device that refers to this touch.
    device: InputDevice,
    /// The underlying touch data.
    touch: *mut wlr_touch
}

impl Touch {
    /// Tries to convert an input device to a Touch.
    ///
    /// Returns none if it is of a different input variant.
    ///
    /// # Safety
    /// This creates a totally new Touch (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_TOUCH => {
                let touch = (*device).__bindgen_anon_1.touch;
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState { handle,
                                                  device: InputDevice::from_ptr(device) });
                (*touch).data = Box::into_raw(state) as *mut _;
                Some(Touch { liveliness,
                             device: InputDevice::from_ptr(device),
                             touch })
            }
            _ => None
        }
    }

    /// Creates an unbound `Touch` from a `TouchHandle`
    unsafe fn from_handle(handle: &TouchHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Touch { liveliness,
                   device: handle.input_device()?.clone(),
                   touch: handle.as_ptr() })
    }

    /// Gets the wlr_input_device associated with this `Touch`.
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    /// Gets the wlr_touch associated with this `Touch`.
    #[allow(dead_code)]
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_touch {
        self.touch
    }

    /// Creates a weak reference to a `Touch`.
    ///
    /// # Panics
    /// If this `Touch` is a previously upgraded `TouchHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> TouchHandle {
        TouchHandle { handle: Rc::downgrade(&self.liveliness),
                      // NOTE Rationale for cloning:
                      // We can't use the keyboard handle unless the keyboard is alive,
                      // which means the device pointer is still alive.
                      device: unsafe { self.device.clone() },
                      touch: self.touch }
    }
}
impl Drop for Touch {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(L_DEBUG, "Dropped Touch {:p}", self.touch);
            unsafe {
                let _ = Box::from_raw((*self.touch).data as *mut InputDevice);
            }
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(L_DEBUG,
                         "Still {} weak pointers to Touch {:p}",
                         weak_count,
                         self.touch);
            }
        }
    }
}

impl TouchHandle {
    /// Constructs a new TouchHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            TouchHandle { handle: Weak::new(),
                          // NOTE Rationale for null pointer here:
                          // It's never used, because you can never upgrade it,
                          // so no way to dereference it and trigger UB.
                          device: InputDevice::from_ptr(ptr::null_mut()),
                          touch: ptr::null_mut() }
        }
    }

    /// Creates an TouchHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    ///
    /// # Panics
    /// Panics if the wlr_touch wasn't allocated using `new_from_input_device`.
    pub(crate) unsafe fn from_ptr(touch: *mut wlr_touch) -> Self {
        if (*touch).data.is_null() {
            panic!("Tried to get handle to keyboard that wasn't set up properly");
        }
        let data = Box::from_raw((*touch).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*touch).data = Box::into_raw(data) as *mut _;
        TouchHandle { handle,
                          touch,
                          device }
    }

    /// Upgrades the touch handle to a reference to the backing `Touch`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Touch`
    /// which may live forever..
    /// But no touch lives forever and might be disconnected at any time.
    pub unsafe fn upgrade(&self) -> HandleResult<Touch> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // touch to exist!
            .and_then(|check| {
                let touch = Touch::from_handle(self)?;
                if check.get() {
                    wlr_log!(L_ERROR, "Double mutable borrows on {:?}", touch);
                    panic!("Double mutable borrows detected");
                }
                check.set(true);
                Ok(touch)
            })
    }

    /// Run a function on the referenced `Touch`, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the `Touch` does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&Touch) -> R
    {
        let mut touch = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut touch)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running touch callback, mutable lock \
                                                    was false for: {:?}",
                                                   touch);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this TouchHandle.
    pub fn input_device(&self) -> HandleResult<&InputDevice> {
        match self.handle.upgrade() {
            Some(_) => Ok(&self.device),
            None => Err(HandleErr::AlreadyDropped)
        }
    }

    /// Gets the `wlr_touch` associated with this `TouchHandle`.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_touch {
        self.touch
    }
}

impl Default for TouchHandle {
    fn default() -> Self {
        TouchHandle::new()
    }
}

impl Clone for TouchHandle {
    fn clone(&self) -> Self {
        TouchHandle { touch: self.touch,
                      handle: self.handle.clone(),
                      /// NOTE Rationale for unsafe clone:
                      ///
                      /// You can only access it after a call to `upgrade`,
                      /// and that implicitly checks that it is valid.
                      device: unsafe { self.device.clone() } }
    }
}

impl PartialEq for TouchHandle {
    fn eq(&self, other: &TouchHandle) -> bool {
        self.touch == other.touch
    }
}

impl Eq for TouchHandle {}
