//! TODO Documentation

use std::panic;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use wlroots_sys::wlr_surface;

use errors::{UpgradeHandleErr, UpgradeHandleResult};

/// The state stored in the wlr_surface user data.
struct SurfaceState {
    /// Used to reconstruct a SurfaceHandle from just an *mut wlr_surface.
    handle: Weak<AtomicBool>
}

/// A Wayland object that represents the data that we display on the screen.
///
/// Most surfaces come from Wayland clients, though they can also be created
/// by the compositor directly.
#[derive(Debug)]
pub struct Surface {
    /// The structe that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `SurfaceHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `SurfaceHandle`.
    liveliness: Option<Rc<AtomicBool>>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    surface: *mut wlr_surface
}

/// See `Surface` for more information on how to use this structure.
#[derive(Clone, Debug)]
pub struct SurfaceHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<AtomicBool>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    surface: *mut wlr_surface
}

impl Surface {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_surface {
        self.surface
    }

    pub(crate) unsafe fn from_ptr(surface: *mut wlr_surface) -> Self {
        if !(*surface).data.is_null() {
            panic!("Tried to construct a Surface from an already initialized wlr_surface");
        }
        let liveliness = Rc::new(AtomicBool::new(false));
        let handle = Rc::downgrade(&liveliness);
        (*surface).data = Box::into_raw(Box::new(SurfaceState { handle })) as _;
        let liveliness = Some(liveliness);
        Surface { liveliness,
                  surface }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed Surface")
            .store(val, Ordering::Release);
    }

    unsafe fn from_handle(handle: &SurfaceHandle) -> Self {
        Surface { liveliness: None,
                  surface: handle.surface }
    }
}

impl SurfaceHandle {
    /// Creates an SurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(surface: *mut wlr_surface) -> Self {
        let data = (*surface).data as *mut SurfaceState;
        let handle = (*data).handle.clone();
        SurfaceHandle { handle, surface }
    }

    /// Upgrades the surface handle to a reference to the backing `Surface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Surface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> UpgradeHandleResult<Surface> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.load(Ordering::Acquire) {
                    return Err(UpgradeHandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(Surface::from_handle(self))
            })
    }

    /// Run a function on the referenced Surface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the surface
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Surface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Surface`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<R>
        where F: FnOnce(&mut Surface) -> R
    {
        let mut surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running surface callback, mutable lock \
                                                    was false for: {:?}",
                                                   surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }
}
