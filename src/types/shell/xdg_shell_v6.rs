//! TODO Documentation

use std::{panic, ptr};
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use wlroots_sys::{wlr_xdg_popup_v6, wlr_xdg_surface_v6, wlr_xdg_surface_v6_ping,
                  wlr_xdg_surface_v6_popup_at, wlr_xdg_surface_v6_popup_get_position,
                  wlr_xdg_surface_v6_role, wlr_xdg_toplevel_v6, wlr_xdg_toplevel_v6_send_close,
                  wlr_xdg_toplevel_v6_set_activated, wlr_xdg_toplevel_v6_set_fullscreen,
                  wlr_xdg_toplevel_v6_set_maximized, wlr_xdg_toplevel_v6_set_resizing,
                  wlr_xdg_toplevel_v6_set_size, wlr_xdg_toplevel_v6_state};

use {Area, SeatId, SurfaceHandle};
use errors::{UpgradeHandleErr, UpgradeHandleResult};
use utils::c_to_rust_string;

/// Used internally to reclaim a handle from just a *mut wlr_xdg_surface_v6.
struct XdgV6ShellSurfaceState {
    handle: Weak<AtomicBool>
}

pub struct XdgV6TopLevel<'surface> {
    toplevel: *mut wlr_xdg_toplevel_v6,
    phantom: PhantomData<&'surface XdgV6ShellSurface>
}

pub struct XdgV6Popup<'surface> {
    popup: *mut wlr_xdg_popup_v6,
    phantom: PhantomData<&'surface XdgV6ShellSurface>
}

/// A tagged enum of the different roles used by the xdg shell.
///
/// Uses the tag to disambiguate the union in `wlr_xdg_surface_v6`.
pub enum XdgV6ShellState<'surface> {
    TopLevel(XdgV6TopLevel<'surface>),
    Popup(XdgV6Popup<'surface>)
}

#[derive(Debug)]
pub struct XdgV6ShellSurface {
    liveliness: Option<Rc<AtomicBool>>,
    shell_surface: *mut wlr_xdg_surface_v6
}

#[derive(Debug, Clone)]
pub struct XdgV6ShellSurfaceHandle {
    handle: Weak<AtomicBool>,
    shell_surface: *mut wlr_xdg_surface_v6
}

impl XdgV6ShellSurface {
    pub(crate) unsafe fn new(shell_surface: *mut wlr_xdg_surface_v6) -> Self {
        // TODO FIXME Free state in drop impl when Rc == 1
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(AtomicBool::new(false));
        let state = Box::new(XdgV6ShellSurfaceState { handle: Rc::downgrade(&liveliness) });
        (*shell_surface).data = Box::into_raw(state) as *mut _;
        XdgV6ShellSurface { liveliness: Some(liveliness),
                            shell_surface }
    }

    unsafe fn from_handle(handle: &XdgV6ShellSurfaceHandle) -> Self {
        XdgV6ShellSurface { liveliness: None,
                            shell_surface: handle.as_ptr() }
    }

    /// Gets the surface used by this XDG shell.
    pub fn surface(&mut self) -> SurfaceHandle {
        unsafe {
            let surface = (*self.shell_surface).surface;
            if surface.is_null() {
                panic!("xdg shell had a null surface!")
            }
            SurfaceHandle::from_ptr(surface)
        }
    }

    /// Get the role of this XDG surface.
    pub fn role(&self) -> wlr_xdg_surface_v6_role {
        unsafe { (*self.shell_surface).role }
    }

    pub fn state<'surface>(&'surface mut self) -> Option<XdgV6ShellState<'surface>> {
        use self::wlr_xdg_surface_v6_role::*;
        use XdgV6ShellState::*;
        unsafe {
            match (*self.shell_surface).role {
                WLR_XDG_SURFACE_V6_ROLE_NONE => None,
                WLR_XDG_SURFACE_V6_ROLE_TOPLEVEL => {
                    let toplevel = (*self.shell_surface).__bindgen_anon_1.toplevel_state;
                    Some(TopLevel(XdgV6TopLevel::new(toplevel)))
                }
                WLR_XDG_SURFACE_V6_ROLE_POPUP => {
                    let popup = (*self.shell_surface).__bindgen_anon_1.popup_state;
                    Some(Popup(XdgV6Popup::new(popup)))
                }
            }
        }
    }

    /// Determines if this XDG shell surface has been configured or not.
    pub fn configured(&self) -> bool {
        unsafe { (*self.shell_surface).configured }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.shell_surface).added }
    }

    pub fn configure_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).configure_serial }
    }

    pub fn configure_next_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).configure_next_serial }
    }

    /// Get the title associated with this XDg shell.
    pub fn title(&self) -> String {
        unsafe {
            c_to_rust_string((*self.shell_surface).title).expect("Could not parse class as UTF-8")
        }
    }

    /// Get the app id associated with this XDG shell.
    pub fn app_id(&self) -> String {
        unsafe {
            c_to_rust_string((*self.shell_surface).app_id).expect("Could not parse class as UTF-8")
        }
    }

    pub fn has_next_geometry(&self) -> bool {
        unsafe { (*self.shell_surface).has_next_geometry }
    }

    pub fn next_geometry(&self) -> Option<Area> {
        unsafe {
            let next_geometry = (*self.shell_surface).next_geometry;
            if next_geometry.is_null() {
                None
            } else {
                Some(Area(*next_geometry))
            }
        }
    }

    pub fn geometry(&self) -> Option<Area> {
        unsafe {
            let geometry = (*self.shell_surface).geometry;
            if geometry.is_null() {
                None
            } else {
                Some(Area(*geometry))
            }
        }
    }

    /// Send a ping to the surface.
    ///
    /// If the surface does not respond with a pong within a reasonable amount of time,
    /// the ping timeout event will be emitted.
    pub fn ping(&mut self) {
        unsafe {
            wlr_xdg_surface_v6_ping(self.shell_surface);
        }
    }

    // TODO FIXME If it's not a toplevel, assert is thrown.
    // Lets control this either with a sexy enum (best option)
    // or with our own error reporting (worst option).

    /// Request that this toplevel surface be the given size.
    ///
    /// Returns the associated configure serial.
    pub fn set_size(&mut self, width: u32, height: u32) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_size(self.shell_surface, width, height) }
    }

    /// Request that this toplevel surface show itself in an activated or deactivated
    /// state.
    ///
    /// Returns the associated configure serial.
    pub fn set_activated(&mut self, activated: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_activated(self.shell_surface, activated) }
    }

    /// Request that this toplevel surface consider itself maximized or not
    /// maximized.
    ///
    /// Returns the associated configure serial.
    pub fn set_maximized(&mut self, maximized: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_maximized(self.shell_surface, maximized) }
    }

    /// Request that this toplevel surface consider itself fullscreen or not
    /// fullscreen.
    ///
    /// Returns the associated configure serial.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_fullscreen(self.shell_surface, fullscreen) }
    }

    /// Request that this toplevel surface consider itself to be resizing or not
    /// resizing.
    ///
    /// Returns the associated configure serial.
    pub fn set_resizing(&mut self, resizing: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_resizing(self.shell_surface, resizing) }
    }

    /// Request that this toplevel surface closes.
    pub fn close(&mut self) {
        unsafe { wlr_xdg_toplevel_v6_send_close(self.shell_surface) }
    }

    /// Compute the popup position in surface-local coordinates.
    ///
    /// Return value is in (x, y) format.
    pub fn position(&self) -> (f64, f64) {
        unsafe {
            let (mut x, mut y) = (0.0, 0.0);
            wlr_xdg_surface_v6_popup_get_position(self.shell_surface, &mut x, &mut y);
            (x, y)
        }
    }

    /// Find a popup within this surface at the surface-local coordinates.
    ///
    /// Returns the popup and coordinates in the topmost surface coordinate system
    /// or None if no popup is found at that location.
    pub fn popup_at(&mut self,
                    sx: f64,
                    sy: f64,
                    popup_sx: &mut f64,
                    popup_sy: &mut f64)
                    -> Option<XdgV6ShellSurfaceHandle> {
        unsafe {
            let popup_surface =
                wlr_xdg_surface_v6_popup_at(self.shell_surface, sx, sy, popup_sx, popup_sy);
            if popup_surface.is_null() {
                None
            } else {
                Some(XdgV6ShellSurfaceHandle::from_ptr(popup_surface))
            }
        }
    }

    /// Creates a weak reference to an `XdgV6ShellSurface`.
    ///
    /// # Panics
    /// If this `XdgV6ShellSurface` is a previously upgraded `XdgV6ShellSurfaceHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> XdgV6ShellSurfaceHandle {
        let arc = self.liveliness.as_ref()
                      .expect("Cannot dowgrade a previously upgraded XdgV6ShellSurfaceHandle");
        XdgV6ShellSurfaceHandle { handle: Rc::downgrade(arc),
                                  shell_surface: self.shell_surface }
    }

    /// Manually set the lock used to determine if a double-borrow is
    /// occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    pub(crate) unsafe fn set_lock(&self, val: bool) {
        self.liveliness.as_ref()
            .expect("Tried to set lock on borrowed XdgV6ShellSurface")
            .store(val, Ordering::Release);
    }
}

impl XdgV6ShellSurfaceHandle {
    /// Creates a XdgV6ShellSurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_xdg_surface_v6) -> Self {
        let data = (*shell_surface).data as *mut XdgV6ShellSurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        XdgV6ShellSurfaceHandle { handle,
                                  shell_surface }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `XdgV6ShellSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `XdgV6ShellSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> UpgradeHandleResult<XdgV6ShellSurface> {
        self.handle.upgrade()
            .ok_or(UpgradeHandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = XdgV6ShellSurface::from_handle(self);
                if check.load(Ordering::Acquire) {
                    return Err(UpgradeHandleErr::AlreadyBorrowed)
                }
                check.store(true, Ordering::Release);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced XdgV6ShellSurface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the XdgV6ShellSurface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> UpgradeHandleResult<R>
        where F: FnOnce(&mut XdgV6ShellSurface) -> R
    {
        let mut xdg_surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut xdg_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.load(Ordering::Acquire) {
                                          wlr_log!(L_ERROR,
                                                   "After running XdgV6ShellSurface callback, \
                                                    mutable lock was false for: {:?}",
                                                   xdg_surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.store(false, Ordering::Release);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_xdg_surface_v6 {
        self.shell_surface
    }
}

impl PartialEq for XdgV6ShellSurfaceHandle {
    fn eq(&self, other: &XdgV6ShellSurfaceHandle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for XdgV6ShellSurfaceHandle {}

impl<'surface> XdgV6TopLevel<'surface> {
    fn new(toplevel: *mut wlr_xdg_toplevel_v6) -> XdgV6TopLevel<'surface> {
        XdgV6TopLevel { toplevel,
                        phantom: PhantomData }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.toplevel).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.toplevel).parent) }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.toplevel).added }
    }

    /// Get the client protocol request state.
    pub fn next_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).next }
    }

    /// Get the pending user configure request state.
    pub fn pending_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).pending }
    }

    pub fn current_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).current }
    }
}

impl<'surface> XdgV6Popup<'surface> {
    fn new(popup: *mut wlr_xdg_popup_v6) -> XdgV6Popup<'surface> {
        XdgV6Popup { popup,
                     phantom: PhantomData }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.popup).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.popup).parent) }
    }

    pub fn committed(&self) -> bool {
        unsafe { (*self.popup).committed }
    }

    /// Get the id of the seat associated with this popup.
    pub fn seat_id(&self) -> SeatId {
        unsafe { SeatId::new((*self.popup).seat) }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area((*self.popup).geometry) }
    }
}
