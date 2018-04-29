//! TODO Documentation

use std::{panic, ptr};
use std::cell::Cell;
use std::rc::{Rc, Weak};

use wlroots_sys::{wl_shell_surface_resize, wlr_wl_shell_surface, wlr_wl_shell_surface_configure,
                  wlr_wl_shell_surface_ping, wlr_wl_shell_surface_surface_at};

use SurfaceHandle;
use errors::{HandleErr, HandleResult};
use utils::c_to_rust_string;

struct WlShellSurfaceState {
    handle: Weak<Cell<bool>>
}

#[derive(Debug)]
pub struct WlShellSurface {
    liveliness: Option<Rc<Cell<bool>>>,
    shell_surface: *mut wlr_wl_shell_surface
}

#[derive(Debug, Clone)]
pub struct WlShellSurfaceHandle {
    handle: Weak<Cell<bool>>,
    shell_surface: *mut wlr_wl_shell_surface
}

impl WlShellSurface {
    pub(crate) unsafe fn new(shell_surface: *mut wlr_wl_shell_surface) -> Self {
        // TODO FIXME Free state in drop impl when Rc == 1
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(Cell::new(false));
        let state = Box::new(WlShellSurfaceState { handle: Rc::downgrade(&liveliness) });
        (*shell_surface).data = Box::into_raw(state) as *mut _;
        WlShellSurface { liveliness: Some(liveliness),
                         shell_surface }
    }

    unsafe fn from_handle(handle: &WlShellSurfaceHandle) -> Self {
        WlShellSurface { liveliness: None,
                         shell_surface: handle.as_ptr() }
    }

    /// Gets the surface used by this Wayland shell.
    pub fn surface(&mut self) -> SurfaceHandle {
        unsafe {
            let surface = (*self.shell_surface).surface;
            if surface.is_null() {
                panic!("wl_shell had a null surface!")
            }
            SurfaceHandle::from_ptr(surface)
        }
    }

    /// Determines if this Wayland shell surface has been configured or not.
    pub fn configured(&self) -> bool {
        unsafe { (*self.shell_surface).configured }
    }

    pub fn popup_mapped(&self) -> bool {
        unsafe { (*self.shell_surface).popup_mapped }
    }

    pub fn ping_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).ping_serial }
    }

    /// Get the title associated with this Wayland shell.
    pub fn title(&self) -> String {
        unsafe {
            c_to_rust_string((*self.shell_surface).title).expect("Could not parse class as UTF-8")
        }
    }

    /// Get the class associated with this Wayland shell.
    pub fn class(&self) -> String {
        unsafe {
            c_to_rust_string((*self.shell_surface).class).expect("Could not parse class as UTF-8")
        }
    }

    /// Send a ping to the surface.
    ///
    /// If the surface does not respond with a pong within a reasonable amount of time,
    /// the ping timeout event will be emitted.
    pub fn ping(&mut self) {
        unsafe {
            wlr_wl_shell_surface_ping(self.shell_surface);
        }
    }
    /// Request that the surface configure itself to be the given size.
    pub fn configure(&mut self, edges: wl_shell_surface_resize, width: i32, height: i32) {
        unsafe {
            wlr_wl_shell_surface_configure(self.shell_surface, edges, width, height);
        }
    }

    /// Find a surface within this surface at the surface-local coordinates.
    ///
    /// Returns the popup and coordinates in the topmost surface coordinate system
    /// or None if no popup is found at that location.
    pub fn surface_at(&mut self,
                      sx: f64,
                      sy: f64,
                      sub_sx: &mut f64,
                      sub_sy: &mut f64)
                      -> Option<SurfaceHandle> {
        unsafe {
            let sub_surface =
                wlr_wl_shell_surface_surface_at(self.shell_surface, sx, sy, sub_sx, sub_sy);
            if sub_surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(sub_surface))
            }
        }
    }

    /// Creates a weak reference to an `WlShellSurface`.
    ///
    /// # Panics
    /// If this `WlShellSurface` is a previously upgraded `WlShellSurfaceHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> WlShellSurfaceHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot dowgrade a previously upgraded WlShellSurfaceHandle");
        WlShellSurfaceHandle { handle: Rc::downgrade(arc),
                               shell_surface: self.shell_surface }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed WlShellSurface")
            .set(val);
    }
}

impl WlShellSurfaceHandle {
    /// Constructs a new WlShellSurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            WlShellSurfaceHandle { handle: Weak::new(),
                                   shell_surface: ptr::null_mut() }
        }
    }

    /// Creates a WlShellSurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_wl_shell_surface) -> Self {
        let data = (*shell_surface).data as *mut WlShellSurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        WlShellSurfaceHandle { handle,
                               shell_surface }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `WlShellSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `WlShellSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<WlShellSurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = WlShellSurface::from_handle(self);
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced WlShellSurface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the WlShellSurface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut WlShellSurface) -> R
    {
        let mut wl_shell_surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut wl_shell_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running WlShellSurface callback, \
                                                    mutable lock was false for: {:?}",
                                                   wl_shell_surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_wl_shell_surface {
        self.shell_surface
    }
}

impl Default for WlShellSurfaceHandle {
    fn default() -> Self {
        WlShellSurfaceHandle::new()
    }
}

impl PartialEq for WlShellSurfaceHandle {
    fn eq(&self, other: &WlShellSurfaceHandle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for WlShellSurfaceHandle {}
