//! TODO Documentation

use std::{panic, ptr, rc::{Rc, Weak}, sync::atomic::{AtomicBool, Ordering}};

use wlroots_sys::wlr_subsurface;

use super::{SurfaceHandle, SurfaceState};
use errors::{HandleErr, HandleResult};

#[derive(Debug)]
pub struct Subsurface {
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
    subsurface: *mut wlr_subsurface
}

#[derive(Clone, Debug)]
pub struct SubsurfaceHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<AtomicBool>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    subsurface: *mut wlr_subsurface
}

impl Subsurface {
    pub(crate) unsafe fn new(subsurface: *mut wlr_subsurface) -> Self {
        let liveliness = Some(Rc::new(AtomicBool::new(false)));
        Subsurface { subsurface,
                     liveliness }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_subsurface {
        self.subsurface
    }

    /// Get a handle to the surface for this sub surface.
    pub fn surface(&self) -> SurfaceHandle {
        unsafe { SurfaceHandle::from_ptr((*self.subsurface).surface) }
    }

    /// Get a handle to the parent surface for this sub surface.
    pub fn parent_surface(&self) -> SurfaceHandle {
        unsafe { SurfaceHandle::from_ptr((*self.subsurface).parent) }
    }

    /// Get the cached state of the sub surface.
    pub fn cached_state<'surface>(&'surface self) -> Option<SurfaceState<'surface>> {
        unsafe {
            if (*self.subsurface).cached.is_null() {
                None
            } else {
                Some(SurfaceState::new((*self.subsurface).cached))
            }
        }
    }

    /// Determine if the sub surface has a cached state.
    pub fn has_cache(&self) -> bool {
        unsafe { (*self.subsurface).has_cache }
    }

    pub fn synchronized(&self) -> bool {
        unsafe { (*self.subsurface).synchronized }
    }

    pub fn reordered(&self) -> bool {
        unsafe { (*self.subsurface).reordered }
    }

    /// Creates a weak reference to a `Subsurface`.
    ///
    /// # Panics
    /// If this `Subsurface` is a previously upgraded `SubsurfaceHandle`
    /// then this function will panic.
    pub fn weak_reference(&self) -> SubsurfaceHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot downgrade a previously upgraded OutputHandle");
        SubsurfaceHandle { handle: Rc::downgrade(arc),
                           subsurface: self.subsurface }
    }

    unsafe fn from_handle(handle: &SubsurfaceHandle) -> Self {
        Subsurface { liveliness: None,
                     subsurface: handle.subsurface }
    }
}

impl SubsurfaceHandle {
    /// Constructs a new SubsurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            SubsurfaceHandle { handle: Weak::new(),
                               subsurface: ptr::null_mut() }
        }
    }
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_subsurface {
        self.subsurface
    }

    /// Upgrades the surface handle to a reference to the backing `Surface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Surface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Subsurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.load(Ordering::Acquire) {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(Subsurface::from_handle(self))
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
    pub fn run<F, R>(&mut self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut Subsurface) -> R
    {
        let mut subsurface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut subsurface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running subsurface callback, mutable \
                                                    lock was false for: {:?}",
                                                   subsurface);
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

impl Default for SubsurfaceHandle {
    fn default() -> Self {
        SubsurfaceHandle::new()
    }
}
