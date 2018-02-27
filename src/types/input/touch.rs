//! TODO Documentation

use std::panic;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use errors::{UpgradeHandleErr, UpgradeHandleResult};
use wlroots_sys::{wlr_input_device, wlr_touch};

use InputDevice;

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
    liveliness: Option<Rc<AtomicBool>>,
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
    handle: Weak<AtomicBool>,
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
                Some(Touch { liveliness: Some(Rc::new(AtomicBool::new(false))),
                             device: InputDevice::from_ptr(device),
                             touch })
            }
            _ => None
        }
    }

    /// Creates an unbound `Touch` from a `TouchHandle`
    unsafe fn from_handle(handle: &TouchHandle) -> Self {
        Touch { liveliness: None,
                device: handle.input_device().clone(),
                touch: handle.as_ptr() }
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
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade a previously upgraded TouchHandle!");
        TouchHandle { handle: Rc::downgrade(arc),
                      // NOTE Rationale for cloning:
                      // We can't use the keyboard handle unless the keyboard is alive,
                      // which means the device pointer is still alive.
                      device: unsafe { self.device.clone() },
                      touch: self.touch }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed Touch")
            .store(val, Ordering::Release);
    }
}
impl Drop for Touch {
    fn drop(&mut self) {
        match self.liveliness {
            None => {}
            Some(ref liveliness) => {
                if Rc::strong_count(liveliness) == 1 {
                    wlr_log!(L_DEBUG, "Dropped Touch {:p}", self.touch);
                    let weak_count = Rc::weak_count(liveliness);
                    if weak_count > 0 {
                        wlr_log!(L_DEBUG,
                                 "Still {} weak pointers to Touch {:p}",
                                 weak_count,
                                 self.touch);
                    }
                }
            }
        }
    }
}

impl TouchHandle {
    /// Upgrades the touch handle to a reference to the backing `Touch`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Touch`
    /// which may live forever..
    /// But no touch lives forever and might be disconnected at any time.
    pub unsafe fn upgrade(&self) -> UpgradeHandleResult<Touch> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // touch to exist!
            .and_then(|check| {
                let touch = Touch::from_handle(self);
                if check.load(Ordering::Acquire) {
                    wlr_log!(L_ERROR, "Double mutable borrows on {:?}", touch);
                    panic!("Double mutable borrows detected");
                }
                check.store(true, Ordering::Release);
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
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<R>
        where F: FnOnce(&Touch) -> R
    {
        let mut touch = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut touch)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running touch callback, mutable lock \
                                                    was false for: {:?}",
                                                   touch);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this TouchHandle.
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    /// Gets the `wlr_touch` associated with this `TouchHandle`.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_touch {
        self.touch
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
