//! TODO Documentation
use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}};

use errors::{HandleErr, HandleResult};
use wlroots_sys::{wlr_input_device, wlr_tablet_pad};

use input::{self, InputState};
pub use manager::tablet_pad_handler::*;
pub use events::tablet_pad_events as event;

#[derive(Debug)]
pub struct TabletPad {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `tablet_pad::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `tablet_pad::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this tablet pad.
    device: input::Device,
    /// Underlying tablet state
    pad: *mut wlr_tablet_pad
}

#[derive(Debug)]
pub struct Handle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the tablet tool associated with this handle,
    handle: Weak<Cell<bool>>,
    /// The device that refers to this tablet_pad.
    device: input::Device,
    /// The underlying tablet state
    pad: *mut wlr_tablet_pad
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
                let pad = (*device).__bindgen_anon_1.tablet_pad;
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState { handle,
                                                  device: input::Device::from_ptr(device) });
                (*pad).data = Box::into_raw(state) as *mut _;
                Some(TabletPad { liveliness,
                                 device: input::Device::from_ptr(device),
                                 pad })
            }
            _ => None
        }
    }

    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(TabletPad { liveliness,
                       device: handle.input_device()?.clone(),
                       pad: handle.as_ptr() })
    }

    /// Gets the wlr_input_device associated with this TabletPad.
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }

    // TODO Real functions

    /// Creates a weak reference to a `TabletPad`.
    ///
    /// # Panics
    /// If this `TabletPad` is a previously upgraded `TabletPad`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> Handle {
        Handle { handle: Rc::downgrade(&self.liveliness),
                          // NOTE Rationale for cloning:
                          // We can't use the tablet tool handle unless the tablet tool is alive,
                          // which means the device pointer is still alive.
                          device: unsafe { self.device.clone() },
                          pad: self.pad }
    }
}

impl Drop for TabletPad {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) != 1 {
            return
        }
        wlr_log!(WLR_DEBUG, "Dropped TabletPad {:p}", self.pad);
        unsafe {
            let _ = Box::from_raw((*self.pad).data as *mut InputState);
        }
        let weak_count = Rc::weak_count(&self.liveliness);
        if weak_count > 0 {
            wlr_log!(WLR_DEBUG,
                     "Still {} weak pointers to TabletPad {:p}",
                     weak_count,
                     self.pad);
        }
    }
}

impl Handle {
    /// Constructs a new tablet_pad::Handle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            Handle { handle: Weak::new(),
                              // NOTE Rationale for null pointer here:
                              // It's never used, because you can never upgrade it,
                              // so no way to dereference it and trigger UB.
                              device: input::Device::from_ptr(ptr::null_mut()),
                              pad: ptr::null_mut() }
        }
    }

    /// Creates an tablet_pad::Handle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    ///
    /// # Panics
    /// Panics if the wlr_tablet_pad wasn't allocated using `new_from_input_device`.
    pub(crate) unsafe fn from_ptr(pad: *mut wlr_tablet_pad) -> Self {
        if (*pad).data.is_null() {
            panic!("Tried to get handle to keyboard that wasn't set up properly");
        }
        let data = Box::from_raw((*pad).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*pad).data = Box::into_raw(data) as *mut _;
        Handle { handle,
                         pad,
                         device }
    }

    /// Upgrades the tablet tool handle to a reference to the backing `TabletPad`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbounded `TabletPad`
    /// which may live forever..
    /// But no tablet tool lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<TabletPad> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let pad = TabletPad::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(pad)
            })
    }

    /// Run a function on the referenced TabletPad, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the TabletPad does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `TabletPad`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut TabletPad) -> R
    {
        let mut pad = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut pad)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(WLR_ERROR,
                                                   "After running tablet tool callback, mutable \
                                                    lock was false for: {:?}",
                                                   pad);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this tablet_pad::Handle
    pub fn input_device(&self) -> HandleResult<&input::Device> {
        match self.handle.upgrade() {
            Some(_) => Ok(&self.device),
            None => Err(HandleErr::AlreadyDropped)
        }
    }

    /// Gets the wlr_tablet_tool associated with this tablet_pad::Handle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_tablet_pad {
        self.pad
    }
}

impl Default for Handle {
    fn default() -> Self {
        Handle::new()
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Handle { pad: self.pad,
                          handle: self.handle.clone(),
                          /// NOTE Rationale for unsafe clone:
                          ///
                          /// You can only access it after a call to `upgrade`,
                          /// and that implicitly checks that it is valid.
                          device: unsafe { self.device.clone() } }
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Handle) -> bool {
        self.pad == other.pad
    }
}

impl Eq for Handle {}
