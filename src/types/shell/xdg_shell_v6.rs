//! TODO Documentation

use std::{panic, ptr};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use wlroots_sys::{wlr_xdg_popup_v6, wlr_xdg_surface_v6, wlr_xdg_surface_v6_ping,
                  wlr_xdg_surface_v6_popup_get_position, wlr_xdg_surface_v6_role,
                  wlr_xdg_surface_v6_send_close, wlr_xdg_surface_v6_surface_at,
                  wlr_xdg_toplevel_v6, wlr_xdg_toplevel_v6_set_activated,
                  wlr_xdg_toplevel_v6_set_fullscreen, wlr_xdg_toplevel_v6_set_maximized,
                  wlr_xdg_toplevel_v6_set_resizing, wlr_xdg_toplevel_v6_set_size,
                  wlr_xdg_toplevel_v6_state};

use {Area, SeatHandle, SurfaceHandle};
use errors::{HandleErr, HandleResult};
use utils::c_to_rust_string;

/// Used internally to reclaim a handle from just a *mut wlr_xdg_surface_v6.
struct XdgV6ShellSurfaceState {
    handle: Weak<AtomicBool>,
    shell_state: Option<XdgV6ShellState>
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct XdgV6TopLevel {
    shell_surface: *mut wlr_xdg_surface_v6,
    toplevel: *mut wlr_xdg_toplevel_v6
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct XdgV6Popup {
    shell_surface: *mut wlr_xdg_surface_v6,
    popup: *mut wlr_xdg_popup_v6
}

/// A tagged enum of the different roles used by the xdg shell.
///
/// Uses the tag to disambiguate the union in `wlr_xdg_surface_v6`.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum XdgV6ShellState {
    TopLevel(XdgV6TopLevel),
    Popup(XdgV6Popup)
}

#[derive(Debug)]
pub struct XdgV6ShellSurface {
    liveliness: Option<Rc<AtomicBool>>,
    state: Option<XdgV6ShellState>,
    shell_surface: *mut wlr_xdg_surface_v6
}

#[derive(Debug)]
pub struct XdgV6ShellSurfaceHandle {
    state: Option<XdgV6ShellState>,
    handle: Weak<AtomicBool>,
    shell_surface: *mut wlr_xdg_surface_v6
}

impl Clone for XdgV6ShellSurfaceHandle {
    fn clone(&self) -> Self {
        let state = match self.state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        XdgV6ShellSurfaceHandle { state,
                                  handle: self.handle.clone(),
                                  shell_surface: self.shell_surface }
    }
}

impl XdgV6ShellSurface {
    pub(crate) unsafe fn new<T: Into<Option<XdgV6ShellState>>>
                             (shell_surface: *mut wlr_xdg_surface_v6, state: T)
                             -> Self {
        let state = state.into();
        // TODO FIXME Free state in drop impl when Rc == 1
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(AtomicBool::new(false));
        let shell_state =
            Box::new(XdgV6ShellSurfaceState { handle: Rc::downgrade(&liveliness),
                                              shell_state: match state {
                                                  None => None,
                                                  Some(ref state) => Some(state.clone())
                                              } });
        (*shell_surface).data = Box::into_raw(shell_state) as *mut _;
        XdgV6ShellSurface { liveliness: Some(liveliness),
                            state: state,
                            shell_surface }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xdg_surface_v6 {
        self.shell_surface
    }

    unsafe fn from_handle(handle: &XdgV6ShellSurfaceHandle) -> Self {
        XdgV6ShellSurface { liveliness: None,
                            state: handle.clone().state,
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

    pub fn state(&mut self) -> Option<&mut XdgV6ShellState> {
        self.state.as_mut()
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

    pub fn has_next_geometry(&self) -> bool {
        unsafe { (*self.shell_surface).has_next_geometry }
    }

    pub fn next_geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface).next_geometry) }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface).geometry) }
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
                wlr_xdg_surface_v6_surface_at(self.shell_surface, sx, sy, sub_sx, sub_sy);
            if sub_surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(sub_surface))
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
                                  state: match self.state {
                                      None => None,
                                      Some(ref state) => unsafe { Some(state.clone()) }
                                  },
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
    /// Constructs a new XdgV6ShellSurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            XdgV6ShellSurfaceHandle { handle: Weak::new(),
                                      state: None,
                                      shell_surface: ptr::null_mut() }
        }
    }

    /// Creates a XdgV6ShellSurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_xdg_surface_v6) -> Self {
        let data = (*shell_surface).data as *mut XdgV6ShellSurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        let state = match (*data).shell_state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        XdgV6ShellSurfaceHandle { handle,
                                  state,
                                  shell_surface }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `XdgV6ShellSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `XdgV6ShellSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<XdgV6ShellSurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = XdgV6ShellSurface::from_handle(self);
                if check.load(Ordering::Acquire) {
                    return Err(HandleErr::AlreadyBorrowed)
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
    pub fn run<F, R>(&mut self, runner: F) -> HandleResult<R>
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

impl Default for XdgV6ShellSurfaceHandle {
    fn default() -> Self {
        XdgV6ShellSurfaceHandle::new()
    }
}

impl PartialEq for XdgV6ShellSurfaceHandle {
    fn eq(&self, other: &XdgV6ShellSurfaceHandle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for XdgV6ShellSurfaceHandle {}

impl XdgV6TopLevel {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface_v6,
                                    toplevel: *mut wlr_xdg_toplevel_v6)
                                    -> XdgV6TopLevel {
        XdgV6TopLevel { shell_surface,
                        toplevel }
    }

    /// Get the title associated with this XDG shell toplevel.
    pub fn title(&self) -> String {
        unsafe { c_to_rust_string((*self.toplevel).title).expect("Could not parse class as UTF-8") }
    }

    /// Get the app id associated with this XDG shell toplevel.
    pub fn app_id(&self) -> String {
        unsafe {
            c_to_rust_string((*self.toplevel).app_id).expect("Could not parse class as UTF-8")
        }
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

    /// Get the pending client state.
    pub fn client_pending_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).client_pending }
    }

    /// Get the pending server state.
    pub fn server_pending_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).server_pending }
    }

    /// Get the current configure state.
    pub fn current_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).current }
    }

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
        unsafe { wlr_xdg_surface_v6_send_close(self.shell_surface) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xdg_toplevel_v6 {
        self.toplevel
    }
}

impl XdgV6Popup {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface_v6,
                                    popup: *mut wlr_xdg_popup_v6)
                                    -> XdgV6Popup {
        XdgV6Popup { shell_surface,
                     popup }
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

    /// Get a handle to the seat associated with this popup.
    pub fn seat_handle(&self) -> Option<SeatHandle> {
        unsafe {
            let seat = (*self.popup).seat;
            if seat.is_null() {
                None
            } else {
                Some(SeatHandle::from_ptr(seat))
            }
        }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.popup).geometry) }
    }
}

impl XdgV6ShellState {
    /// Unsafe copy of the pointer
    unsafe fn clone(&self) -> Self {
        use XdgV6ShellState::*;
        match *self {
            TopLevel(XdgV6TopLevel { shell_surface,
                                     toplevel }) => {
                TopLevel(XdgV6TopLevel { shell_surface,
                                         toplevel })
            }
            Popup(XdgV6Popup { shell_surface,
                               popup }) => {
                Popup(XdgV6Popup { shell_surface,
                                   popup })
            }
        }
    }
}
