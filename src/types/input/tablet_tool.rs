//! TODO Documentation
use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}};

use errors::{HandleErr, HandleResult};
use wlroots_sys::{wlr_input_device, wlr_tablet_tool};

use InputDevice;

#[derive(Debug)]
pub struct TabletTool {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `TabletToolHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `TabletToolHandle`.
    liveliness: Option<Rc<Cell<bool>>>,
    /// The device that refers to this tablet tool.
    device: InputDevice,
    /// Underlying tablet state
    tool: *mut wlr_tablet_tool
}

#[derive(Debug)]
pub struct TabletToolHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the tablet tool associated with this handle,
    handle: Weak<Cell<bool>>,
    /// The device that refers to this tablet_tool.
    device: InputDevice,
    /// The underlying tablet state
    tool: *mut wlr_tablet_tool
}

impl TabletTool {
    /// Tries to convert an input device to a TabletTool
    ///
    /// Returns None if it is of a different type of input variant.
    ///
    /// # Safety
    /// This creates a totally new TabletTool (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_TABLET_TOOL => {
                let tool = (*device).__bindgen_anon_1.tablet_tool;
                Some(TabletTool { liveliness: Some(Rc::new(Cell::new(false))),
                                  device: InputDevice::from_ptr(device),
                                  tool })
            }
            _ => None
        }
    }

    unsafe fn from_handle(handle: &TabletToolHandle) -> HandleResult<Self> {
        Ok(TabletTool { liveliness: None,
                        device: handle.input_device()?.clone(),
                        tool: handle.as_ptr() })
    }

    /// Gets the wlr_input_device associated with this TabletTool.
    pub fn input_device(&self) -> &InputDevice {
        &self.device
    }

    // TODO Real functions

    /// Creates a weak reference to a `TabletTool`.
    ///
    /// # Panics
    /// If this `TabletTool` is a previously upgraded `TabletTool`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> TabletToolHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade previously upgraded TabletToolHandle!");
        TabletToolHandle { handle: Rc::downgrade(arc),
                           // NOTE Rationale for cloning:
                           // We can't use the tablet tool handle unless the tablet tool is alive,
                           // which means the device pointer is still alive.
                           device: unsafe { self.device.clone() },
                           tool: self.tool }
    }
}

impl Drop for TabletTool {
    fn drop(&mut self) {
        if let Some(liveliness) = self.liveliness.as_ref() {
            if Rc::strong_count(liveliness) != 1 {
                return
            }
            wlr_log!(L_DEBUG, "Dropped TabletTool {:p}", self.tool);
            let weak_count = Rc::weak_count(liveliness);
            if weak_count > 0 {
                wlr_log!(L_DEBUG,
                         "Still {} weak pointers to TabletTool {:p}",
                         weak_count,
                         self.tool);
            }
        }
    }
}

impl TabletToolHandle {
    /// Constructs a new TabletToolHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            TabletToolHandle { handle: Weak::new(),
                               // NOTE Rationale for null pointer here:
                               // It's never used, because you can never upgrade it,
                               // so no way to dereference it and trigger UB.
                               device: InputDevice::from_ptr(ptr::null_mut()),
                               tool: ptr::null_mut() }
        }
    }

    /// Upgrades the tablet tool handle to a reference to the backing `TabletTool`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbounded `TabletTool`
    /// which may live forever..
    /// But no tablet tool lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<TabletTool> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let tool = TabletTool::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(tool)
            })
    }

    /// Run a function on the referenced TabletTool, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the tablet pad
    /// to a short lived scope of an anonymous function,
    /// this function ensures the TabletTool does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `TabletPad`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut TabletTool) -> R
    {
        let mut tool = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut tool)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running tablet tool callback, mutable \
                                                    lock was false for: {:?}",
                                                   tool);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Gets the wlr_input_device associated with this TabletToolHandle
    pub fn input_device(&self) -> HandleResult<&InputDevice> {
        match self.handle.upgrade() {
            Some(_) => Ok(&self.device),
            None => Err(HandleErr::AlreadyDropped)
        }
    }

    /// Gets the wlr_tablet_tool associated with this TabletToolHandle.
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_tablet_tool {
        self.tool
    }
}

impl Default for TabletToolHandle {
    fn default() -> Self {
        TabletToolHandle::new()
    }
}

impl Clone for TabletToolHandle {
    fn clone(&self) -> Self {
        TabletToolHandle { tool: self.tool,
                           handle: self.handle.clone(),
                           /// NOTE Rationale for unsafe clone:
                           ///
                           /// You can only access it after a call to `upgrade`,
                           /// and that implicitly checks that it is valid.
                           device: unsafe { self.device.clone() } }
    }
}

impl PartialEq for TabletToolHandle {
    fn eq(&self, other: &TabletToolHandle) -> bool {
        self.tool == other.tool
    }
}

impl Eq for TabletToolHandle {}
